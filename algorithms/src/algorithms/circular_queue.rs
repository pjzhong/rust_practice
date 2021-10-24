/**
 * Your MyCircularQueue object will be instantiated and called as such:
 * let obj = MyCircularQueue::new(k);
 * let ret_1: bool = obj.en_queue(value);
 * let ret_2: bool = obj.de_queue();
 * let ret_3: i32 = obj.front();
 * let ret_4: i32 = obj.rear();
 * let ret_5: bool = obj.is_empty();~
 * let ret_6: bool = obj.is_full();
 */

/// click [`here`] to leetcode
///
/// [`here`]: https://leetcode.com/problems/design-circular-queue/
struct MyCircularQueue {
    front: usize,
    back: usize,
    elements: Vec<i32>,
}

/**
 * `&self` means the method takes an immutable reference.
 * If you need a mutable reference, change it to `&mut self` instead.
 */
#[allow(dead_code)]
impl MyCircularQueue {
    fn new(k: i32) -> Self {
        Self {
            front: 0usize,
            back: 0usize,
            elements: vec![-1; k as usize],
        }
    }

    fn en_queue(&mut self, value: i32) -> bool {
        let back = self.back;
        let front = self.front;

        let len = back.wrapping_sub(front);

        if len >= self.elements.capacity() {
            return false;
        }

        let idx = back % self.elements.capacity();

        self.elements[idx] = value;
        self.back = back.wrapping_add(1);

        true
    }

    fn de_queue(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        let idx = self.front % self.elements.capacity();
        self.elements[idx] = -1;
        self.front = self.front.wrapping_add(1);

        true
    }

    fn front(&self) -> i32 {
        if self.is_empty() {
            -1
        } else {
            let idx = self.front % self.elements.capacity();
            return self.elements[idx];
        }
    }

    fn rear(&self) -> i32 {
        if self.is_empty() {
            -1
        } else {
            let idx = (self.back - 1) % self.elements.capacity();
            return self.elements[idx];
        }
    }

    fn is_empty(&self) -> bool {
        self.back.wrapping_sub(self.front) == 0
    }

    fn is_full(&self) -> bool {
        let back = self.back;
        let front = self.front;

        let len = back.wrapping_sub(front);

        len >= self.elements.capacity()
    }
}

#[test]
fn next_power_of_two_test() {
    let two: usize = 3;
    assert_eq!(4, two.next_power_of_two());
}

#[test]
fn wrapping_sub_test() {
    let b: usize = 0;
    let t: usize = 100;

    assert_eq!(100, t.wrapping_sub(b));

    let b1: usize = 0;
    let t1: usize = 0;

    assert_eq!(0, t1.wrapping_sub(b1));

    let b2: usize = 1;
    let t2: usize = 129;
    assert_eq!(128, t2.wrapping_sub(b2))
}

#[test]
fn option_replace_test() {
    let mut i32s = vec![None; 3];

    i32s[0].replace(1);

    assert_eq!(vec![Some(1), None, None], i32s);
    assert_ne!(vec![Some(3), None, None], i32s);
}

#[test]
fn option_take_test() {
    let mut i32s = vec![Some(1), Some(2), Some(3)];

    i32s[1].take();

    assert_eq!(vec![Some(1), None, Some(3)], i32s);
}

#[test]
fn my_queue_test() {}
