use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread;
use std::thread::JoinHandle;

use rand::Rng;

use algorithms::deque::new;
use algorithms::deque::Stolen::{Data, Empty};

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
