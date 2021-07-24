//! 步幅调度算法的调度器 [`StrideScheduler`]

use super::Scheduler;
use alloc::collections::LinkedList;

/// 将线程和调度信息打包
struct StrideThread<ThreadType: Clone + Eq> {
    /// 线程当前步数
    pub step: usize,
    /// 每个线程的步幅，步幅大小为ticket/优先级+1
    pub stride: usize,
    /// 线程数据
    pub thread: ThreadType,
}

/// 采用 Stride Scheduling（步幅调度算法）的调度器
pub struct StrideScheduler<ThreadType: Clone + Eq> {
    ticket: usize,
    /// 带有调度信息的线程池
    pool: LinkedList<StrideThread<ThreadType>>,
}
impl<ThreadType: Clone + Eq> Default for StrideScheduler<ThreadType> {
    fn default() -> Self {
        Self {
            ticket: 100,
            pool: LinkedList::new(),
        }
    }
}
impl<ThreadType: Clone + Eq> Scheduler<ThreadType> for StrideScheduler<ThreadType> {
    type Priority = usize;

    fn add_thread(&mut self, thread: ThreadType) {
        self.pool.push_back(StrideThread {
            step: 0,
            /// 默认优先级为1
            stride: self.ticket / 1,
            thread,
        });
    }

    fn get_next(&mut self) -> Option<ThreadType> {
        if let Some(best) = self.pool.iter_mut().min_by(|x, y| x.step.cmp(&(y.step))) {
            best.step += best.stride;
            Some(best.thread.clone())
        } else {
            None
        }
    }

    fn remove_thread(&mut self, thread: &ThreadType) {
        let mut removed = self.pool.drain_filter(|t| t.thread == *thread);
        assert!(removed.next().is_some() && removed.next().is_none());
    }

    fn set_priority(&mut self, thread: ThreadType, priority: Self::Priority) {
        let mut iter = self.pool.iter_mut().filter(|t| t.thread == thread);
        if let Some(thread) = iter.next() {
            if !iter.next().is_none() {
                panic!("set_priority error")
            }
            thread.stride = self.ticket / priority + 1;
        } else {
            panic!("set_priority error")
        }
    }
}
