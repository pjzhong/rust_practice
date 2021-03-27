use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/number-of-good-pairs/
impl Solution {
    pub fn num_identical_pairs(nums: Vec<i32>) -> i32 {
        let mut result = 0;
        let size = nums.len();
        for i in 0..size {
            for j in i + 1..size {
                if nums[i] == nums[j] {
                    result += 1;
                }
            }
        }

        result
    }
}

#[test]
fn num_identical_pairs_test() {
    println!("{}", Solution::num_identical_pairs(vec![1, 2, 3, 1, 1, 3]));
}
