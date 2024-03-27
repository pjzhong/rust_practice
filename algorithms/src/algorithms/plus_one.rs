use super::Solution;

impl Solution {
    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.cn/problems/plus-one/description/
    #[allow(unused)]
    pub fn plus_one(digits: Vec<i32>) -> Vec<i32> {
        let mut digits = digits;

        let mut plus = 1;
        for i in (0..digits.len()).rev() {
            digits[i] += plus;
            plus = digits[i] / 10;
            if 0 < plus {
                digits[i] %= 10;
            } else {
                break;
            }
        }

        if 0 < plus {
            digits.insert(0, plus);
        }

        digits
    }
}

#[test]
fn test() {
    assert_eq!(vec![1, 2, 2], Solution::plus_one(vec![1, 2, 1]));
    assert_eq!(vec![1], Solution::plus_one(vec![0]));
    assert_eq!(vec![1, 0, 0], Solution::plus_one(vec![9, 9]));
}
