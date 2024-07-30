# 功能实现

在`ProcessControlBlock`中添加`mutex`和`semphore`的`avalible`数组，`request`矩阵，`allocation`矩阵等数据结构。

`thread`，`mutex`和`semphore`创建时初始化这些数据结构，线程执行`mutex lock`和`semphore down`操作时运行安全性检查算法。

# 问答作业

## 1

需要回收所有线程的`USER RES`，进程地址空间，`fd_table`。

计时器`timer`中的`TaskControlBlock`引用，需要手动回收
`TaskManager`中的`TaskControlBlock`引用，需要手动回收
`ProcessControlBlock`中的处于blocking状态的`TaskControlBlock`引用随PCB被回收，无需手动回收

## 2

`mutex_inner.locked = true`执行位置不同。

当`wait_queue`中有2个以上blocking的线程时，第一种可能会意外将锁释放，无法保证互斥性。

# 参考文献
[计算机操作系统 - 死锁](https://github.com/CyC2018/CS-Notes/blob/master/notes/%E8%AE%A1%E7%AE%97%E6%9C%BA%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%20-%20%E6%AD%BB%E9%94%81.md#3-%E5%A4%9A%E4%B8%AA%E8%B5%84%E6%BA%90%E7%9A%84%E9%93%B6%E8%A1%8C%E5%AE%B6%E7%AE%97%E6%B3%95)