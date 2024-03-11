use super::Solution;

impl Solution {
    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.cn/problems/reverse-integer/description/
    #[allow(unused)]
    pub fn reverse(x: i32) -> i32 {
        let sign = if x < 0 { -1 } else { 1 };

        let mut res: i32 = 0;
        let mut x = match x.checked_abs() {
            Some(x) => x,
            None => return 0,
        };
        while x != 0 {
            let n = x % 10;
            x /= 10;

            res = if let Some(res) = res.checked_mul(10).and_then(|res| res.checked_add(n)) {
                res
            } else {
                return 0;
            }
        }

        res.checked_mul(sign).unwrap_or_default()
    }
}

#[test]
fn sign_test() {
    assert!(0 < Solution::reverse(10));
    assert!(Solution::reverse(-10) < 0);
    assert!(Solution::reverse(0) == 0)
}

#[test]
fn result_checked() {
    assert_eq!(Solution::reverse(0), 0);
    assert_eq!(Solution::reverse(321), 123);
    assert_eq!(Solution::reverse(-123), -321);
    assert_eq!(Solution::reverse(120), 21);
    assert_eq!(Solution::reverse(dbg!(i32::MAX)), 0);
    assert_eq!(Solution::reverse(dbg!(i32::MIN)), 0);
    assert_eq!(Solution::reverse(dbg!(2147483641)), 1463847412);
    assert_eq!(Solution::reverse(dbg!(-2147483641)), -1463847412);
    assert_eq!(Solution::reverse(dbg!(00000000001)), 1);
}
