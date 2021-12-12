use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread;
use std::thread::JoinHandle;

use rand::Rng;

use algorithms::deque::Stolen::{Data, Empty};
use algorithms::deque::{new, Stealer, Worker};

#[test]
fn smoke() {
    let (w, s) = new::<isize>();
    assert_eq!(w.pop(), None);
    assert_eq!(s.steal(), Empty);
    w.push(1);
    assert_eq!(w.pop(), Some(1));
    w.push(1);
    assert_eq!(s.steal(), Data(1));
    w.push(1);
    assert_eq!(s.clone().steal(), Data(1));

    assert_eq!(w.pop(), None);
    assert_eq!(s.steal(), Empty);
}

#[test]
fn steal_push() {
    static AMT: isize = 100000;
    let (w, s) = new::<isize>();
    let t = thread::spawn(move || {
        let mut left = AMT;
        while left > 0 {
            match s.steal() {
                Data(i) => {
                    assert_eq!(i, 1);
                    left -= 1;
                }
                _ => {}
            }
        }
    });

    for _ in 0..AMT {
        w.push(1);
    }

    t.join().unwrap();
}

#[test]
fn steal_push_large() {
    static AMT: isize = 100000;
    let (w, s) = new::<(isize, isize)>();
    let t = thread::spawn(move || {
        let mut left = AMT;
        while left > 0 {
            match s.steal() {
                Data(i) => {
                    assert_eq!(i, (1, 10));
                    left -= 1;
                }
                _ => {}
            }
        }
    });

    for _ in 0..AMT {
        w.push((1, 10));
    }

    t.join().unwrap();
}

#[test]
fn stress() {
    static AMT: isize = 10000000;
    static NTHREADS: isize = 8;
    static DONE: AtomicBool = AtomicBool::new(false);
    static HITS: AtomicUsize = AtomicUsize::new(0);
    let (w, s) = new::<isize>();

    let threads = (0..NTHREADS)
        .map(|_| {
            let s = s.clone();
            thread::spawn(move || loop {
                match s.steal() {
                    Data(2) => {
                        HITS.fetch_add(1, SeqCst);
                    }
                    _ if DONE.load(SeqCst) => break,
                    _ => {}
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    let mut rng = rand::thread_rng();
    let mut expected = 0;
    while expected < AMT {
        if rng.gen_range(0, 3) == 2 {
            match w.pop() {
                None => {}
                Some(2) => {
                    HITS.fetch_add(1, SeqCst);
                }
                _ => panic!(),
            }
        } else {
            expected += 1;
            w.push(2);
        }
    }

    while HITS.load(SeqCst) < AMT as usize {
        match w.pop() {
            None => {}
            Some(2) => {
                HITS.fetch_add(1, SeqCst);
            }
            Some(_) => panic!(),
        }
    }

    DONE.store(true, SeqCst);

    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(HITS.load(SeqCst), expected as usize);
}

#[derive(Clone, Copy)]
struct UnsafeAtomicUsize(*mut AtomicUsize);

unsafe impl Send for UnsafeAtomicUsize {}

fn stampede(w: Worker<Box<isize>>, s: Stealer<Box<isize>>, nthreads: isize, amt: usize) {
    for _ in 0..amt {
        w.push(Box::new(20));
    }

    let mut remaining = AtomicUsize::new(amt);
    let mut count = AtomicUsize::new(0);
    let unsafe_remaing = UnsafeAtomicUsize(&mut remaining);
    let unsafe_count = UnsafeAtomicUsize(&mut count);

    let threads = (0..nthreads)
        .map(|_| {
            let s = s.clone();
            thread::spawn(move || unsafe {
                let UnsafeAtomicUsize(unsafe_remaing) = unsafe_remaing;
                let UnsafeAtomicUsize(unsafe_count) = unsafe_count;
                while (*unsafe_remaing).load(SeqCst) > 0 {
                    match s.steal() {
                        Data(ref i) if **i == 20 => {
                            (*unsafe_remaing).fetch_sub(1, SeqCst);
                            (*unsafe_count).fetch_add(1, SeqCst);
                        }
                        Data(..) => panic!(),
                        _ => {}
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    while remaining.load(SeqCst) > 0 {
        match w.pop() {
            Some(ref i) if **i == 20 => {
                remaining.fetch_sub(1, SeqCst);
                count.fetch_add(1, SeqCst);
            }
            Some(..) => panic!(),
            None => {}
        }
    }

    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(amt, count.load(SeqCst));
}

#[test]
fn run_stampede() {
    let (w, s) = new::<Box<isize>>();
    stampede(w, s, 8, 10000)
}

#[test]
fn many_stampede() {
    static AMT: usize = 4;
    let threads = (0..AMT)
        .map(|_| {
            let (w, s) = new::<Box<isize>>();
            thread::spawn(move || {
                stampede(w, s, 4, 100000);
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for thread in threads {
        thread.join().unwrap();
    }
}
