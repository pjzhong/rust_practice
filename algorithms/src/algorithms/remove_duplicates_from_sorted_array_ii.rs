use crate::algorithms::Solution;

#[allow(dead_code)]
impl Solution {
    /// click [`here`] to leetcode。
    /// leetcode上面有更简洁的，完美的呈现如何运用【大小】规律
    ///
    /// [`here`]: https://leetcode.com/problems/remove-duplicates-from-sorted-array-ii/
    pub fn remove_duplicates_remove_duplicates_from_sorted(nums: &mut Vec<i32>) -> i32 {
        let len = nums.len();
        let mut read = 0;
        let mut write = 0;
        let mut pre = i32::MIN;
        let mut count = 0;
        while read < len {
            if pre != nums[read] {
                count = 0;
            }

            pre = nums[read];
            count += 1;
            if count <= 2 {
                nums[write] = nums[read];
                write += 1;
            }
            read += 1;
        }

        write.max(1) as i32
    }

    /// click [`here`] to leetcode。
    /// leetcode上面有更简洁的，完美的呈现如何运用【大小】规律
    ///
    /// [`here`]: https://leetcode.com/problems/remove-duplicates-from-sorted-array/
    pub fn remove_duplicates_remove_duplicates_i_from_sorted(nums: &mut Vec<i32>) -> i32 {
        let mut write = 0;
        let mut pre = i32::MIN;
        for i in 0..nums.len() {
            if pre != nums[i] {
                nums[write] = nums[i];
                write += 1;
            }

            pre = nums[i];
        }

        write as i32
    }
}

#[test]
fn remove_duplicates_from_sorted_test() {
    {
        let mut example = vec![0, 0, 1, 1, 1, 1, 2, 3, 3];
        assert_eq!(
            7,
            Solution::remove_duplicates_remove_duplicates_from_sorted(&mut example)
        );
        assert_eq!(vec![0, 0, 1, 1, 2, 3, 3, 3, 3], example);
    }

    {
        let mut example = vec![1, 1, 1, 2, 2, 3];
        assert_eq!(
            5,
            Solution::remove_duplicates_remove_duplicates_from_sorted(&mut example)
        );
        assert_eq!(vec![1, 1, 2, 2, 3, 3], example);
    }

    {
        let mut example = vec![1, 2, 3, 3, 3, 5, 4, 4, 4];
        assert_eq!(
            7,
            Solution::remove_duplicates_remove_duplicates_from_sorted(&mut example)
        );
        assert_eq!(vec![1, 2, 3, 3, 5, 4, 4, 4, 4], example)
    }
}
