use std::iter::FromIterator;

use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/reorganize-string/
#[allow(dead_code)]
impl Solution {

    // 借鉴了 https://leetcode.com/problems/reorganize-string/discuss/232469/Java-No-Sort-O(N)-0ms-beat-100
    // 想是想到了，但没实现出来...
    pub fn reorganize_string(s: String) -> String {
        let diff = 'a' as u32;
        let mut counts = vec![(0, '.'); 26];
        let mut max_idx = 0;
        for c in s.chars() {
            let idx = (c as u32 - diff) as usize;
            counts[idx].0 += 1;
            counts[idx].1 = c;
            if counts[max_idx].0 < counts[idx].0 {
                max_idx = idx;
            }
        }

        let mut res = vec!['0'; s.chars().count()];
        let mut idx = 0;
        let size = res.len();
        // fill the most characters first
        let (mut x, y) = counts[max_idx];
        if x > ((size + 1) / 2) as u32 {
            return String::new();
        }

        while 0 < x {
            res[idx] = y;
            idx += 2;
            x -= 1;
        }

        counts[max_idx].0 = 0;

        for (mut x, y) in counts {
            while x > 0 {
                if idx >= size {
                    //idx has loop for a while in previous
                    idx = 1;
                }
                res[idx] = y;
                idx += 2;
                x -= 1;
            }
        }

        String::from_iter(res.iter())
    }
}

#[test]
fn reorganize_string_test() {
    println!("{}", Solution::reorganize_string("abcd".to_string()));
    println!("{}", Solution::reorganize_string("aab".to_string()));
    println!("{}", Solution::reorganize_string("abc".to_string()));
    println!("{}", Solution::reorganize_string("aacc".to_string()));
    println!("{}", Solution::reorganize_string("acacd".to_string()));
    println!("{}", Solution::reorganize_string("aaccdd".to_string()));
    println!("{}", Solution::reorganize_string("aaab".to_string()));
    println!("{}", Solution::reorganize_string("vvvlo".to_string()));
}
