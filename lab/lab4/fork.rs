pub fn fork(&self, context: &Context) -> MemoryResult<Arc<Thread>> {
    let process = self.process.clone();
    let stack = process.alloc_page_range(STACK_SIZE, Flags::READABLE | Flags::WRITABLE)?;
    let mut new_context = context.clone();
    new_context.set_sp(new_context.sp() + stack.start.0 - self.stack.start.0);
    let thread = Arc::new(Thread {
        id: unsafe {
            THREAD_COUNTER += 1;
            THREAD_COUNTER
        },
        stack,
        process,
        inner: Mutex::new(ThreadInner {
            context: Some(new_context),
            sleeping: false,
            dead: false,
        }),
    });

    Ok(thread)
}
