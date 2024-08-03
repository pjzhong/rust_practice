use super::Solution;

#[allow(dead_code)]
impl Solution {
    /// 果然是递归的简单题目，只要构建出了1-3的解法，然后任意N的解法就完成了
    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.cn/problems/hanota-lcci/description/
    pub fn hanota(a: &mut Vec<i32>, b: &mut Vec<i32>, c: &mut Vec<i32>) {
        Self::hanota_inner(a.len(), a, b, c)
    }

    fn hanota_inner(n: usize, a: &mut Vec<i32>, b: &mut Vec<i32>, c: &mut Vec<i32>) {
        if n == 0 {
            if let Some(num) = a.pop() {
                c.push(num);
            }
            return;
        }

        //把N-1个盘从A移动到B
        Self::hanota_inner(n - 1, a, c, b);
        //把最大的盘从A移动到C
        Self::hanota_inner(0, a, b, c);
        //把N-1个盘从B移动到C
        Self::hanota_inner(n - 1, b, a, c)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn basic() {
        Solution::hanota(&mut vec![], &mut vec![], &mut vec![]);

        let mut a = vec![2];
        let mut b = vec![];
        let mut c = vec![];

        Solution::hanota(&mut a, &mut b, &mut c);

        assert!(a.is_empty());
        assert!(b.is_empty());
        assert_eq!(c, vec![2]);
    }

    #[test]
    fn two() {
        let mut a = vec![2, 1];
        let mut b = vec![];
        let mut c = vec![];

        Solution::hanota(&mut a, &mut b, &mut c);

        assert!(a.is_empty());
        assert!(b.is_empty());
        assert_eq!(c, vec![2, 1]);
    }

    #[test]
    fn three() {
        let mut a = vec![2, 1, 0];
        let mut b = vec![];
        let mut c = vec![];

        Solution::hanota(&mut a, &mut b, &mut c);

        assert!(a.is_empty());
        assert!(b.is_empty());
        assert_eq!(c, vec![2, 1, 0]);
    }

    #[test]
    fn n() {
        let mut a = (1..10).rev().collect();
        let mut b = vec![];
        let mut c = vec![];

        Solution::hanota(&mut a, &mut b, &mut c);

        assert!(a.is_empty());
        assert!(b.is_empty());
        assert_eq!(c, (1..10).rev().collect::<Vec<_>>());
    }
}
