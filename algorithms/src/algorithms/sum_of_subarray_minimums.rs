use std::collections::LinkedList;

use crate::algorithms::Solution;

#[allow(dead_code)]
impl Solution {
    /// click [`here`] to leetcode
    /// [here]: https://leetcode.com/problems/sum-of-subarray-minimums/
    pub fn sum_subarray_mins(arr: Vec<i32>) -> i32 {
        let mut res: usize = 0;
        let mut stack = LinkedList::new();

        stack.push_front((arr[0], 0usize));
        for i in 1..arr.len() {
            let num = arr[i];
            if let Some((val, _)) = stack.front() {
                if num <= *val {
                    res += Solution::pop_out(num, i, &mut stack);
                }
                stack.push_front((num, i));
            }
        }
        res += Solution::pop_out(0, arr.len(), &mut stack);

        (res % 1_000_000_007) as i32
    }

    fn pop_out(val: i32, idx: usize, stack: &mut LinkedList<(i32, usize)>) -> usize {
        let mut res = 0;
        while let Some((num, i)) = stack.pop_front() {
            if val <= num {
                let add = {
                    let mut add = idx - i;
                    let prev = if let Some((_, prev)) = stack.front() {
                        i - *prev - 1
                    } else {
                        i
                    };
                    add += add * prev;
                    add
                };
                res += num as usize * add;
            } else {
                stack.push_front((num, i));
                break;
            }
        }

        res
    }
}

/// click [`here`] to leetcode
/// [here]: https://leetcode.com/problems/sum-of-subarray-minimums/
fn brute_force(arr: Vec<i32>) -> i32 {
    let mut res: usize = 0;

    for i in 0..arr.len() {
        let mut min = arr[i];
        for j in i..arr.len() {
            min = min.min(arr[j]);
            res += min as usize;
        }
    }
    (res % 1_000_000_007) as i32
}

#[test]
fn sum_subarray_minimums_test() {
    assert_eq!(
        brute_force(vec![1, 2, 3, 4]),
        Solution::sum_subarray_mins(vec![1, 2, 3, 4]),
        "1, 2, 3, 4"
    );
    assert_eq!(
        brute_force(vec![3, 1, 2, 4]),
        Solution::sum_subarray_mins(vec![3, 1, 2, 4]),
        "3, 1, 2, 4"
    );
    assert_eq!(
        brute_force(vec![2, 3, 1, 2, 4]),
        Solution::sum_subarray_mins(vec![2, 3, 1, 2, 4]),
        "2, 3, 1, 2, 4"
    );
    assert_eq!(
        brute_force(vec![3, 1, 2, 4, 7, 5]),
        Solution::sum_subarray_mins(vec![3, 1, 2, 4, 7, 5]),
        "3, 1, 2, 4, 7,5"
    );
    assert_eq!(
        brute_force(vec![1, 1, 3, 4]),
        Solution::sum_subarray_mins(vec![1, 1, 3, 4]),
        "three"
    );
    assert_eq!(
        brute_force(vec![11, 81, 94, 43, 3]),
        Solution::sum_subarray_mins(vec![11, 81, 94, 43, 3]),
        "four"
    );
    assert_eq!(
        brute_force(vec![81, 94, 43, 45, 46]),
        Solution::sum_subarray_mins(vec![81, 94, 43, 45, 46]),
        "81, 94, 43, 45, 46"
    );
    assert_eq!(
        brute_force(vec![11, 81, 94, 43, 45, 46, 3]),
        Solution::sum_subarray_mins(vec![11, 81, 94, 43, 45, 46, 3]),
        "11, 81, 94, 43, 45, 46, 3"
    );

    let vec: Vec<i32> = (1000..2000).collect();
    assert_eq!(
        brute_force(vec.clone()),
        Solution::sum_subarray_mins(vec.clone()),
        "{:?}",
        vec
    );
}
