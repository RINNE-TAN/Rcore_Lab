pub struct ClockSwapper {
    queue: Vec<(VirtualPageNumber, FrameTracker, *mut PageTableEntry)>,
    /// 映射数量上限
    quota: usize,
    /// 时钟指针指向的位置
    postion: usize,
}
unsafe impl Send for ClockSwapper {}
impl Swapper for ClockSwapper {
    fn new(quota: usize) -> Self {
        Self {
            queue: Vec::new(),
            quota,
            postion: 0,
        }
    }

    fn full(&self) -> bool {
        self.queue.len() == self.quota
    }

    fn pop(&mut self) -> Option<(VirtualPageNumber, FrameTracker)> {
        let mut step = 0;
        loop {
            let entry = self.queue[self.postion].2;
            let flags = unsafe { entry.as_ref().unwrap().flags() };
            // 两位同时为0则返回
            if !flags.contains(Flags::ACCESSED) {
                let res = self.queue.remove(self.postion);
                return Some((res.0, res.1));
            }
            // 将ACCESSED位置为0
            unsafe {
                entry.as_mut().unwrap().set_flags(flags & !Flags::ACCESSED);
            }
            // 移至下一位
            self.postion = (self.postion + 1) % self.quota;
            step = step + 1;
            if step == self.quota + 1 {
                break;
            }
        }
        None
    }

    fn push(&mut self, vpn: VirtualPageNumber, frame: FrameTracker, entry: *mut PageTableEntry) {
        self.queue.insert(self.postion, (vpn, frame, entry));
    }

    fn retain(&mut self, predicate: impl Fn(&VirtualPageNumber) -> bool) {
        self.queue.retain(|(vpn, _, _)| predicate(vpn));
    }
}
