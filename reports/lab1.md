# 功能总结

我在`TaskInfo`结构体中添加了`syscall_times`和`time`2个字段。前者为一个长度为`MAX_SYSCALL_NUM`的u32数组，记录每个程序每个系统调用次数，后者记录应用启动的时间。当应用发起系统调用时，内核将`syscall_times[syscall_id] += 1`。用户发起`sys_task_info`返回`syscall_times`数组，`get_time_ms() - time`和`TaskStatus::Running`。

我认为可以使用哈希表来做系统调用接口计数，`syscall_id`做为key，调用次数为value。

# 问答作业
## 1
SBI版本：RustSBI-QEMU Version 0.2.0-alpha.2

错误输出：
- ch2b_bad_address.rs: PageFault in application, bad addr = 0x0, bad instruction = 0x804003ac, kernel killed it.
- ch2b_bad_instructions.rs：IllegalInstruction in application, kernel killed it.
- ch2b_bad_register.rs：IllegalInstruction in application, kernel killed it.
## 2
### 1

刚进入`__restore`时，`a0`代表了`trap_handle()`的返回值。`__restore`主要在应用程序初始化和系统调用完成后返回用户态

### 2

L43-L48特殊处理了`sstatus`，`sepc`和`sscratch`

`sstatus`中保存了各种状态信息，如`SPP`字段指定`sret`后的CPU特权级

`sepc`保存了切换回用户态时PC寄存器的值

`sscratch`保存了应用栈栈顶位置

### 3

`x2`为栈指针（`sp`）,`__restore`后面还要使用且需要用来与`sscratch`切换内核栈和用户栈

`x4`为线程指针（`tp`），目前rcore还没有使用线程

### 4

`sp`指向用户栈栈顶，`sscratch`指向内核栈栈顶

### 5

L61: `sret`，`sret`会根据`sstatus`的`SPP`字段来决定进入用户态还是内核态

### 6

`sscratch`和`sp`的值发生交换，`sp`指向内核栈栈顶，`sscratch`指向用户栈栈顶

### 7
`stvec`指向的指令即`__alltraps`（L13: `csrrw sp, sscratch, sp`）

# 参考文献
- RISC-V 开放架构设计之道 1.0.0
- 一些网页资料