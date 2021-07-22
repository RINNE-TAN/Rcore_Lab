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

同时维护另一个变量，每次进程运行，计数器将会自增（名为step）

调度方法：

每次挑step最小的线程，自增量为stride。

算法的核心就是保证线程位移的尽可能接近

每次进程执行时在轴上移动一定的步长，由于每个进程的步长不一样，每次都选择位移（step）最少的线程去执行，

关于算法的几个问题：
- 在 Stride Scheduling 算法下，如果一个线程进入了一段时间的等待（例如等待输入，此时它不会被运行），会发生什么？

    这个比较难理解，如果线程进入了休眠，则不会对调度队列中的线程调度造成影响，在线程被唤醒后，因为这个线程的step相对于别的线程是很低的，故在被唤醒后（输入），会更优先得执行。
- 对于两个优先级分别为 9 和 1 的线程，连续 10 个时间片中，前者的运行次数一定更多吗？

    不一定。决定线程调度的直接因素是step，而不是优先级。
- 你认为 Stride Scheduling 算法有什么不合理之处？可以怎样改进？
    第一个问题是在线程休眠后和新假如线程时难以设置优先级。怎么改进暂时没想好。
    第二个问题是在短时间内的吞吐量会带来巨大误差，参考于[论文](http://web.eecs.umich.edu/~mosharaf/Readings/Stride.pdf)

    里面提到了一种极端的情况：
    
    For example, consider a set of 101 clients with a
    100 : 1 : 1 : 1... ticket allocation. A schedule that minimizes absolute error and response time variability would
    alternate the 100-ticket client with each of the singleticket clients. However, the standard stride algorithm
    schedules the clients in order, with the 100-ticket client
    receiving 100 quanta before any other client receives
    a single quantum. Thus, after 100 allocations, the intended allocation for the 100-ticket client is 50, while
    its actual allocation is 100, yielding a large absolute
    error of 50. This behavior is also exhibited by similar rate-based flow control algorithms for networks.

    这里提到一种假设，假如有100张票，一个优先级为1的线程和100个优先级为1的线程共101个线程一起工作，那在前100个时间周期里面优先级为1的线程分到的份额是100%，但是实际上应该是50%，这就使其他线程陷入长时间的饥饿状态。

    文章的后面提到了Hierarchical stride scheduling算法（分层步幅调度算法），可以改进这种情况，这个算法比较难理解，有时间在回头看了

