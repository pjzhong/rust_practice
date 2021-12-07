use core::ptr;
use std::mem::forget;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst};
use std::sync::atomic::{fence, AtomicIsize, AtomicPtr};
use std::sync::Arc;

static MIN_SIZE: usize = 32;

struct Deque<T: Send> {
    bottom: AtomicIsize,
    top: AtomicIsize,
    array: AtomicPtr<Buffer<T>>,
}

pub struct Worker<T: Send> {
    deque: Arc<Deque<T>>,
}

pub struct Stealer<T: Send> {
    deque: Arc<Deque<T>>,
}

struct Buffer<T: Send> {
    storage: *mut T,
    size: usize,
    prev: Option<Box<Buffer<T>>>,
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

        return if self
            .top
            .compare_exchange(t, t.wrapping_add(1), SeqCst, Relaxed)
            == Ok(t)
        {
            self.bottom.store(t.wrapping_add(1), Relaxed);
            Some(data)
        } else {
            self.bottom.store(t.wrapping_add(1), Relaxed);
            forget(data);
            None
        };
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
            storage: vec![].as_mut_ptr(),
            size,
            prev: None,
        }
    }

    fn size(&self) -> isize {
        self.size() as isize
    }

    fn mask(&self) -> isize {
        self.size as isize - 1
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
        return buf;
    }
}

impl<T: Send> Worker<T> {
    pub fn pop(&self) -> Option<T> {
        self.deque.pop()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let (w, s) = new::<isize>();
        assert_eq!(w.pop(), None);
    }

    #[test]
    fn wrapping_test() {
        let max_one = add(isize::MAX, 1);
        let max_two = isize::MAX.wrapping_add(1);
        assert_eq!(max_one, max_two);
    }

    fn add(one: isize, two:isize) -> isize {
        one + two
    }
}
