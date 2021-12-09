use core::ptr;
use std::mem::forget;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, AtomicPtr, fence};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst};

use crate::deque::Stolen::{Abort, Data, Empty};

static MIN_SIZE: usize = 32;

struct Deque<T: Send> {
    bottom: AtomicIsize,
    top: AtomicIsize,
    array: AtomicPtr<Buffer<T>>,
}

pub struct Worker<T: Send> {
    deque: Arc<Deque<T>>,
}

/// The stealing half of the work-stealing deque. Stealers have access to the
/// opposite end of the deque from the worker, and they only have access to the
/// `steal` method.
pub struct Stealer<T: Send> {
    deque: Arc<Deque<T>>,
}

impl<T: Send> Clone for Stealer<T> {
    fn clone(&self) -> Self {
        Self {
            deque: self.deque.clone(),
        }
    }
}

/// When stealing some data, this is an enumeration of the possible outcomes.
#[derive(PartialEq, Debug)]
pub enum Stolen<T> {
    /// The deque was empty at the time of stealing
    Empty,
    /// The stealer lost the race for stealing data, and a retry may return more
    /// data.
    Abort,
    /// The stealer has successfully stolen some data.
    Data(T),
}

struct Buffer<T: Send> {
    storage: *mut T,
    size: usize,
    prev: Option<Box<Buffer<T>>>,
}

impl<T: Send> Stealer<T> {
    pub fn steal(&self) -> Stolen<T> {
        self.deque.steal()
    }
}

impl<T: Send> Deque<T> {
    fn new() -> Deque<T> {
        let buf = Box::new(Buffer::new(MIN_SIZE));
        Self {
            bottom: AtomicIsize::new(0),
            top: AtomicIsize::new(0),
            array: AtomicPtr::new(Box::into_raw(buf)),
        }
    }

    fn pop(&self) -> Option<T> {
        let b = self.bottom.load(Relaxed);
        let t = self.top.load(Relaxed);

        // Early exit if the deque is empty. This avoids the need for a SeqCst
        // fence in this case.
        if b.wrapping_sub(t) <= 0 {
            return None;
        }

        // Make sure bottom is stored before top is read.
        let b = b.wrapping_sub(1);
        self.bottom.store(b, Relaxed);
        fence(SeqCst);
        let t = self.top.load(Relaxed);

        let size = b.wrapping_sub(t);
        if size < 0 {
            self.bottom.store(b.wrapping_add(1), Relaxed);
            return None;
        }

        let a = self.array.load(Relaxed);
        let data = unsafe { (*a).get(b) };

        if size != 0 {
            return Some(data);
        }

        self.bottom.store(t.wrapping_add(1), Relaxed);
        if self
            .top
            .compare_exchange(t, t.wrapping_add(1), SeqCst, Relaxed)
            == Ok(t)
        {
            Some(data)
        } else {
            forget(data);
            None
        }
    }

    fn steal(&self) -> Stolen<T> {
        let t = self.top.load(Acquire);
        fence(SeqCst);
        let b = self.bottom.load(Acquire);

        let size = b.wrapping_sub(t);
        if size <= 0 {
            return Empty;
        }

        let a = self.array.load(Acquire);
        let data = unsafe { (*a).get(t) };

        if self
            .top
            .compare_exchange(t, t.wrapping_add(1), SeqCst, Relaxed)
            == Ok(t)
        {
            Data(data)
        } else {
            forget(data);
            Abort
        }
    }

    fn push(&self, data: T) {
        let b = self.bottom.load(Relaxed);
        let t = self.top.load(Acquire);
        let mut a = self.array.load(Relaxed);

        let size = b.wrapping_sub(t);
        unsafe {
            if size == (*a).size() {
                a = Box::into_raw(Box::from_raw(a).grow(b, t));
                self.array.store(a, Release);
            }

            (*a).put(b, data);
        }

        fence(Release);
        self.bottom.store(b.wrapping_add(1), Relaxed);
    }
}

impl<T: Send> Buffer<T> {
    fn new(size: usize) -> Buffer<T> {
        Self {
            storage: allocate(size),
            size,
            prev: None,
        }
    }

    fn size(&self) -> isize {
        self.size as isize
    }

    fn mask(&self) -> isize {
        (self.size - 1) as isize
    }

    fn elem(&self, i: isize) -> *mut T {
        unsafe { self.storage.offset(i & self.mask()) }
    }

    fn get(&self, i: isize) -> T {
        unsafe { ptr::read(self.elem(i)) }
    }

    fn put(&self, i: isize, t: T) {
        unsafe {
            ptr::write(self.elem(i), t);
        }
    }

    fn grow(self: Box<Buffer<T>>, b: isize, t: isize) -> Box<Buffer<T>> {
        let mut buf = Box::new(Buffer::new(self.size * 2));
        let mut i = t;
        while i != b {
            buf.put(i, self.get(i));
            i = i.wrapping_add(1);
        }
        buf.prev = Some(self);
        buf
    }
}

impl<T: Send> Worker<T> {
    pub fn pop(&self) -> Option<T> {
        self.deque.pop()
    }

    pub fn push(&self, t: T) {
        self.deque.push(t);
    }
}

pub fn new<T: Send>() -> (Worker<T>, Stealer<T>) {
    let deque = Arc::new(Deque::new());
    (
        Worker {
            deque: deque.clone(),
        },
        Stealer { deque },
    )
}

fn allocate<T>(number: usize) -> *mut T {
    let v = Vec::with_capacity(number);
    take_ptr_from_vec(v)
}

fn take_ptr_from_vec<T>(mut buf: Vec<T>) -> *mut T {
    let ptr = buf.as_mut_ptr();
    forget(buf);
    ptr
}