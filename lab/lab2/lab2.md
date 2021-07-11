# Lab 2

目前阶段对lab-2做了一部分调试，先记录一下调试结果和理解

## 物理页分配的调试
``` rust
for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
        println!("{} and {}", frame_0.address(), frame_1.address());
    }
```
最后的输出结果：

PhysicalAddress(0x80a20000) and PhysicalAddress(0x80a21000)

PhysicalAddress(0x80a20000) and PhysicalAddress(0x80a21000)

这里发生了物理页的分配和回收的过程，在for循环里面第一次分配和第二次分配得到的物理地址相同，因为第一次循环结束以后就出了局部变量的作用域，发生了回收，故第二次分配也分配到了第一次的地址上面。~~虽然我还是没搞懂物理页分配~~

关于教程中提到的问题，以下这段代码的执行

``` rust
match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
```
最终会无法执行到后面的panic!("end of rust_main"),然后一直触发时钟中断（陷入死循环的表现），这里是因为申请的内存没有对应的消费者，由于FRAME_ALLOCATOR用了自旋锁包裹起来，因为并没有drop所以自旋锁会一直循环等待锁的释放，这里没有释放的机会，故陷入死循环。

## 堆空间分配的调试

如果用heap.rs中引用的buddy_system_allocator算法，由于要引入外部的lib，相对起来调试没那么简单，我先把heap2.rs中的bitmap算法拿来进行堆的分配，同时println!调试，在每次alloc和dealloc的时候打印状态和申请（释放）内存的大小，观察了一下程序运行的逻辑。

比较神奇的是在前面Box申请的内存和Vec申请的内存都是正常地释放了
而且Vec的申请释放大致如下:

alloc 32

alloc 64

dealloc 32

alloc 128

dealloc 64

...

和之前了解到的切片扩容策略一致，也证明了算法的正确性，不过bitmap这里限制的大小比较小，所以很快就溢出了。

意料之外的是会有段莫名其妙申请的内存并且还没有被释放，出现在分配页表那部分，而且for循环当中两次分配的大小还不一样。定位了以后，发现是因为在分配页表时用到了StackedAllocator，这个数据结构包含了Vec，在分配页表中list发生的扩容。

## 对页分配的理解
对物理页用了带生命周期的变量来绑定，在变量生命周期结束的时候会调用drop方法，如下
``` rust
impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}
```
所以在实现上还是比较自由的，不是限定好的接口，可以进行去分配多个物理页的尝试