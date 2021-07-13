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

## 线段树分配物理页总结
**实现的功能**

1. 实现连续任意大小的物理页分配，在原有接口进行扩展，不止是单页物理页的分配。
2. 实现懒修改，不用在申请/释放内存的时候对所有结点进行操作。
3. 实现线段树不同区间内空余内存的合并，减少内存碎片

**数据结构设计**

``` rust
struct Node {
    left: usize,
    right: usize,
    sum: usize,
    pre: usize,
    suf: usize,
    lazy: usize,
}
```
Node是线段树的单个结点，由于线段树是一颗平衡树，这里采用数组的方式来实现，每个结点的left和right来确定该结点控制的内存范围，sum代表这片内存区域中连续的空余内存的大小，pre和suf分别代表前缀连续内存大小和后缀连续内存大小，并且通过控制pre和suf来进行内存块的合并，lazy是线段树的lazy标记，0代表无标记，1代表该区间内存全部被占用，2代表该区间内存全部被释放

**懒标记**

懒标记是线段树对区间操作时的一个重要思想，在线段树进行修改操作时，如果要对线段树的所有结点进行修改，会大大增加时间复杂度，而懒标记的作用是，在对一段区间进行修改时，不对这个区间下面所有结点修改，而只是对父结点进行标记，当涉及到对父结点的操作，再把标记下放到子结点，而像内存的申请/释放都可以直接对标记进行修改

具体的算法涉及**push_down()**，**pull_up()**两个函数，

**pull_up()**是在子结点数据发送改变时，通过子节点数据来更新父节点数据，内存块的合并也发生在这个时候，当子节点改变后同时满足合并内存块的条件会依次更新父节点。

**push_down()**是在查询可用内存块的位置和分配/释放内存时进行把标记下放给子结点，将父结点的标记清楚，并且修改子节点的数据（如在下放分配内存的标记时，会把子结点的sum、pre、suf全部置为0），并且在最后通过**pull_up()**来更新父节点。

**内存分配和释放**

目前的内存分配策略暂时采用了相对简单的做法，即每次分配都找可分配的连续内存空间地址最小的那一块来进行分配，在搜索到之后，对区间对应的两个父节点或者一个父节点进行标记，释放同理

**测试结果**

![](..\..\image\lab2-result.png)

顺利完成了页表的分配和释放