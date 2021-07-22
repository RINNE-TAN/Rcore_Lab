# Lab 4
## 几个对进程和线程的主要概念:
1. 进程是资源的分配单位，进程之间不共享资源，如果进程之间要共享资源要通过通信的手段
2. 线程是cpu的最小调度单位，不同进程中的线程通过同一个调度器调度。
3. 系统对进程分配资源，不对线程分配资源
## Ctrl+C终止当前线程的实验
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

## 7月22日补充
今天调试时候意外地发现这个实验还存在bug

1. 在运行用户线程的时候，用了提供的notebook程序，此时按下Ctrl+C就会panic，因为此时正在运行的current_thread不是notebook，因为它正在休眠中，等待外设的输入，此时正在运行的实际上是**IDLE_THREAD**，是全局的空闲线程，但是因为这个空闲线程只会等待下一次时钟中断，也没有放进去调度器的pool里面，故在调用kill_current_thread的时候就会出现panic，因为队列中找不到想要删除的线程
2. 基于1的改进还算比较简单，先取出current_thread，如果其和IDLE_THREAD相同的话，就从等待STDIN的休眠线程中取出一个，并移出记录休眠线程的队列中。但是随即遇到新问题，在按下Ctrl+C后，程序没有任何输入输出，顿时把我整懵了，后来仔细定位了一下是我不常遇到的死锁问题
```rust
if c == 3 {
        let mut processor = PROCESSOR.lock();
        let current_thread = processor.current_thread();
        if current_thread == IDLE_THREAD.clone() {
            // 移出当前休眠中等待STDIN的进程
            STDIN.remove_one();
            return processor.prepare_next_thread();
        }
        println!("thread {} has been killed", current_thread.id);
        processor.kill_current_thread();
        return processor.prepare_next_thread();
}
```
乍一看好像没什么毛病

但是因为let mut processor = PROCESSOR.lock();这里获得了PROCESSOR的带锁引用，然而 STDIN.remove_one();也用到了PROCESSOR.lock()的带锁引用，于是STDIN一直在等processor释放锁，而processor又一直在等STDIN执行完，就发生死锁了

**解决方案**：把所有需要processor的地方换成PROCESSOR.lock()，用完立即释放，即可达到目的

**改进后代码**
```rust
if c == 3 {
        let current_thread = PROCESSOR.lock().current_thread();
        if current_thread == IDLE_THREAD.clone() {
            // 移出当前休眠中等待STDIN的进程
            STDIN.remove_one();
            return PROCESSOR.lock().prepare_next_thread();
        }
        println!("thread {} has been killed", current_thread.id);
        PROCESSOR.lock().kill_current_thread();
        return PROCESSOR.lock().prepare_next_thread();
}
```
成功地退出了正在休眠中的notebook进程

## 7月22日再补充
还存在一点疑惑的地方，在实现了fork功能去fork了notebook线程，结果在Ctrl+C的时候别的线程都是正常被kill掉的，只有当只剩下一个notebook线程的时候才会调用remove_sleep_thread()从休眠队列中移除，也就是说自始至终只有最开始的那个线程是进入了休眠等待外设备输入的，别的线程都没进入休眠？不过由于不同线程是具有相同的静态资源的（因为都属于同一个进程，而进程是资源的基本单位），除了线程栈不一样以外的都一样，**意味着它们获取寻找到相同的代码段执行相同的代码的（形象一点理解它们都是去main.rs的93行去执行同一个函数，当然实际上是编译后的字节码）**，这也是为什么用static自增的时候要用unsafe包裹，因为这是线程不安全的，可能与这也有关系，需要再测试理解一下。

后面的测试证实了我的猜想，每次外设有输入的时候唤醒的都是同一个线程（最开始的那个），而且休眠队列和等待外设输入的watch队列也只有一个。

## fork进程的实验

fork进程相对要复杂得多，目前先简单实现了更简单的fork线程

按f键触发比较简单，只要在外设中断的时候判断并且调用fork函数就行
```rust
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
```
由于被fork后的线程也属于同一个进程，故先要clone process，然后再为它分配对应的线程栈，但是栈指针sp也要重新设置，其他部分则直接复制父线程的上下文context。

## Stride Scheduling 调度算法

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

