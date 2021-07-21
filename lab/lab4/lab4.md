# Lab 4
lab-4中有几个对进程和线程的主要概念需要理解:
1. 进程是资源的分配单位，进程之间不共享资源，如果进程之间要共享资源要通过通信的手段
2. 线程是cpu的最小调度单位，不同进程中的线程通过同一个调度器调度。
3. 系统对进程分配资源，不对线程分配资源

关于第一个实验，在键入Ctrl+C的时候要能结束当前进程，这个还是比较容易的，在中断处理中有处理外部中断，在此基础上加个判断即可
```rust
fn supervisor_external(context: &mut Context) -> *mut Context {
    let mut c = console_getchar();
    if c == 3 {
        let mut processor = PROCESSOR.lock();
        let current_thread = processor.current_thread();
        println!("thread {} has been killed", current_thread.id);
        processor.kill_current_thread();
        return processor.prepare_next_thread();
    }
    if c <= 255 {
        if c == '\r' as usize {
            c = '\n' as usize;
        }
        STDIN.push(c as u8);
    }
    context
}
```

Ctrl+C对应的Acell码是3

用户程序的带返回值退出，会触发系统调用，从而带来中断，然后带来sys_exit的回调，输出类似thread 3 exit with code 0的结果

**Stride Scheduling 调度算法**

假设一共100张ticket，ABC分别为10/5/25,那么stride步长为10/20/4（名为stride）

同时维护另一个变量，每次进程运行，计数器将会自增（名为pass）

调度方法：

每次挑pass最小的线程，自增量为stride。

算法的核心就是保证线程位移的尽可能接近

每次进程执行时在轴上移动一定的步长，由于每个进程的步长不一样，每次都选择位移（step）最少的线程去执行，

关于算法的几个问题：