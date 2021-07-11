use crate::algorithms::Solution;

impl Solution {
    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.com/problems/majority-element/
    #[allow(dead_code)]
    pub fn majority_element(nums: Vec<i32>) -> i32 {
        let mut cnt = 0;
        let mut maj = 0;

        for n in nums {
            if cnt == 0 {
                maj = n;
                cnt = 1;
            } else if maj == n {
                cnt += 1;
            } else {
                cnt -= 1;
            }
        }

        maj
    }
}

#[test]
fn majority_element_test() {
    assert_eq!(3, Solution::majority_element(vec![3, 2, 3]));
    assert_eq!(2, Solution::majority_element(vec![2, 2, 1, 1, 1, 2, 2]));
}
