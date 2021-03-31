use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/backspace-string-compare/
#[allow(dead_code)]
impl Solution {
    pub fn backspace_compare(s: String, t: String) -> bool {
        let fun = |str: &String| {
            let mut vec = Vec::new();
            for c in str.chars() {
                if c == '#' {
                    vec.pop();
                } else {
                    vec.push(c);
                }
            }
            vec
        };

        let s_vec = fun(&s);
        let t_vec = fun(&t);
        s_vec.eq(&t_vec)
    }
}

#[test]
fn backspace_string_compare_test() {
    assert!(Solution::backspace_compare(
        "ab#c".to_string(),
        "ad#c".to_string()
    ));
    assert!(!Solution::backspace_compare(
        "a#c".to_string(),
        "b".to_string()
    ));
}
