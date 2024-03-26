use std::collections::VecDeque;

use super::Solution;

impl Solution {

    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.cn/problems/palindrome-number/description/
    #[allow(unused)]
    pub fn is_palindrome(x: i32) -> bool {
        if x.is_negative() {
            return false;
        }

        let mut a = VecDeque::<i32>::new();
        let mut b = VecDeque::<i32>::new();
        let mut x = x;
        while x != 0 {
            let n = x % 10;
            x /= 10;
            a.push_front(n);
            b.push_back(n);
        }

        for (a, b) in a.iter().zip(b.iter()) {
            if a != b {
                return false;
            }
        }

        true
    }
}

#[test]
fn test() {
    assert_eq!(true, Solution::is_palindrome(121));
    assert_eq!(false, Solution::is_palindrome(-10));
    assert_eq!(false, Solution::is_palindrome(10));
    assert_eq!(false, Solution::is_palindrome(i32::MIN));
    assert_eq!(true, Solution::is_palindrome(123321));
}
