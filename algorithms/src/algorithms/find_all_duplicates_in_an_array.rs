use super::Solution;

impl Solution {
    pub fn find_duplicates(mut nums: Vec<i32>) -> Vec<i32> {
        // n == nums.length
        // 1 <= n <= 10^ 5
        // 1 <= nums[i] <= n 这个条件很重要！！！！！！！！数组里面的每个数可以直接作为下标使用！！！！！

        // 桶排序，也是O(n)。不够简洁，写操作变多了
        let mut res = vec![];
        let len = nums.len();
        let mut idx = 0;
        while idx < len {
            let i = nums[idx].abs();
            let target_idx = (i as usize).saturating_sub(1);
            let target = nums[target_idx];
            if idx == target_idx || i == 0 {
                idx += 1;
                continue;
            }

            if i == target {
                res.push(i);
                nums[idx] = 0;
                idx += 1;
            } else {
                //这里把target和idx交换之后，新的[idx]没有处理。会漏处理
                nums.swap(idx, target_idx);
            }
        }

        res

        //这道题主要突破点就是找出数字是否出现过，就是true和false的状态。可以参考leetcode上的答案
        //这个答案更简洁，用负数表示对应数字出现过
        // for idx in 0..num.len() {
        //     let num = nums[idx].abs();
        //     let num_idx = (num - 1) as usize;
        //     if nums[num_idx] < 0 {
        //         res.push(num);
        //     } else {
        //         nums[idx] *= -1;
        //     }
        // }
    }
}

#[test]
fn remove_duplicates_test() {
    use rand::seq::SliceRandom;
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    for _ in 0..100 {
        let max = 10i32.pow(5);
        let n = rng.gen_range(1..=max);
        let rate = rng.gen::<f64>();
        let mut nums: Vec<i32> = vec![];
        let mut result: Vec<i32> = Vec::new();

        for i in 1..=n {
            let i = i as i32;
            if rng.gen_bool(rate) {
                result.push(i);
                nums.push(i);
            }
            nums.push(i);
        }

        nums.shuffle(&mut rng);
        let mut res = Solution::find_duplicates(nums);
        res.sort();
        assert_eq!(result, res);
    }
}
