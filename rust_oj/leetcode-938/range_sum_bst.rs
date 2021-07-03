#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}
use std::cell::RefCell;
use std::rc::Rc;
impl Solution {
    pub fn range_sum_bst(root: Option<Rc<RefCell<TreeNode>>>, low: i32, high: i32) -> i32 {
        let mut ans = 0;
        if let Some(root) = root {
            let root = root.borrow_mut();
            if root.val >= low && root.val <= high {
                ans += root.val;
            }
            if root.val <= high {
                ans += Self::range_sum_bst(root.right.to_owned(), low, high);
            }
            if root.val >= low {
                ans += Self::range_sum_bst(root.left.to_owned(), low, high);
            }
        }
        ans
    }
}
