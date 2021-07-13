//! 用线段树实现的分配器 [`StackedAllocator`]
use super::Allocator;
use alloc::{vec, vec::Vec};
use core::clone::Clone;
pub struct SegmentTreeAllocator {
    tree: Vec<Node>,
}
#[derive(Clone, Copy)]
struct Node {
    left: usize,
    right: usize,
    sum: usize,
    pre: usize,
    suf: usize,
    lazy: usize,
}
impl Allocator for SegmentTreeAllocator {
    fn new(capacity: usize) -> Self {
        let tree = vec![
            Node {
                left: 0,
                right: 0,
                sum: 0,
                pre: 0,
                suf: 0,
                lazy: 0
            };
            5 * capacity / 2
        ];
        let mut allocator = SegmentTreeAllocator { tree };
        allocator.build(1, 1, capacity);
        allocator
    }

    fn alloc(&mut self) -> Option<usize> {
        self.alloc_with_size(1)
    }

    fn dealloc(&mut self, index: usize) {
        self.dealloc_with_size(index, 1)
    }
}
impl SegmentTreeAllocator {
    fn alloc_with_size(&mut self, size: usize) -> Option<usize> {
        if self.tree[1].sum < size {
            return None;
        }
        if let Some(index) = self.search(1, size) {
            self.lazy_change(1, index, index + size - 1, 1);
            Some(index)
        } else {
            None
        }
    }
    fn dealloc_with_size(&mut self, index: usize, size: usize) {
        self.lazy_change(1, index, index + size - 1, 2);
    }
    fn build(&mut self, i: usize, l: usize, r: usize) {
        self.tree[i] = Node {
            left: l,
            right: r,
            pre: r - l + 1,
            suf: r - l + 1,
            sum: r - l + 1,
            lazy: 0,
        };
        if l == r {
            return;
        }
        let mid = (r + l) / 2;
        self.build(left_node(i), l, mid);
        self.build(right_node(i), mid + 1, r);
    }
    fn search(&mut self, i: usize, size: usize) -> Option<usize> {
        self.push_down(i);
        if self.tree[i].left == self.tree[i].right {
            return Some(self.tree[i].left);
        }
        if self.tree[left_node(i)].sum >= size {
            return self.search(left_node(i), size);
        }
        if self.tree[left_node(i)].suf + self.tree[right_node(i)].pre >= size {
            return Some(self.tree[left_node(i)].right - self.tree[left_node(i)].suf + 1);
        }
        if self.tree[right_node(i)].sum >= size {
            return self.search(right_node(i), size);
        }
        None
    }
    fn len(&self, i: usize) -> usize {
        self.tree[i].right - self.tree[i].left + 1
    }
    fn push_down(&mut self, i: usize) {
        let lazy = self.tree[i].lazy;
        if lazy == 0 {
            return;
        }
        if self.tree[i].left != self.tree[i].right {
            //标记下放左结点
            let l = if self.tree[i].lazy == 1 {
                0
            } else {
                self.len(left_node(i))
            };
            self.tree[left_node(i)].pre = l;
            self.tree[left_node(i)].suf = l;
            self.tree[left_node(i)].sum = l;
            self.tree[left_node(i)].lazy = lazy;
            //标记下放右结点
            let r = if self.tree[i].lazy == 1 {
                0
            } else {
                self.len(right_node(i))
            };
            self.tree[right_node(i)].pre = r;
            self.tree[right_node(i)].suf = r;
            self.tree[right_node(i)].sum = r;
            self.tree[right_node(i)].lazy = lazy;
        }
        //删除标记
        self.tree[i].lazy = 0;
        self.pull_up(i);
    }
    fn pull_up(&mut self, i: usize) {
        if self.tree[i].left == self.tree[i].right {
            return;
        }
        //通过子节点更新pre
        self.tree[i].pre = if self.len(left_node(i)) == self.tree[left_node(i)].sum {
            self.tree[left_node(i)].sum + self.tree[right_node(i)].pre
        } else {
            self.tree[left_node(i)].pre
        };
        //通过子节点更新suf
        self.tree[i].suf = if self.len(right_node(i)) == self.tree[right_node(i)].sum {
            self.tree[right_node(i)].sum + self.tree[left_node(i)].suf
        } else {
            self.tree[right_node(i)].suf
        };
        //通过子节点sum
        self.tree[i].sum = max(
            self.tree[left_node(i)].suf + self.tree[right_node(i)].pre,
            max(self.tree[right_node(i)].sum, self.tree[left_node(i)].sum),
        );
    }
    fn lazy_change(&mut self, i: usize, l: usize, r: usize, lazy: usize) {
        self.push_down(i);
        if l <= self.tree[i].left && r >= self.tree[i].right {
            self.tree[i].lazy = lazy;
            let j = if lazy == 1 { 0 } else { self.len(left_node(i)) };
            self.tree[i].pre = j;
            self.tree[i].suf = j;
            self.tree[i].sum = j;
            return;
        }
        let mid = (self.tree[i].left + self.tree[i].right) / 2;
        if l <= mid {
            self.lazy_change(left_node(i), l, min(mid, r), lazy);
        }
        if r > mid {
            self.lazy_change(right_node(i), max(mid, l), r, lazy);
        }
        self.pull_up(i);
    }
}
fn left_node(i: usize) -> usize {
    i * 2
}
fn right_node(i: usize) -> usize {
    i * 2 + 1
}
fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}
fn min(a: usize, b: usize) -> usize {
    if a > b {
        b
    } else {
        a
    }
}
