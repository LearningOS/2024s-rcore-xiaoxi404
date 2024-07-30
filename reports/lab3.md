# 功能总结

spawn的实现思路是在原有的`TaskControlBlock`的`new`方法的基础上增加一个向父进程添加子进程的流程，`TaskControlBlock`的`spawn`方法返回参数与`fork`一样为Arc<Self>。


stride调度算法实现思路是将`TaskManager`的`ready_queue`改成一个按stride从小到大排列的队列，这样每次`ready_queue`添加任务后都要重新排列一番。另外，`TaskControlBlockInner`也需要添加`priority`和`stride`字段。

# 问答作业
## 1
不会轮到`p1`执行，`p1.stride = 255 + 10`会溢出

## 2
### 2.1
pass的最大值为`BigStride / 2`，如果严格按照算法执行，队列中`STRIDE_MAX`和`STRIDE_MIN`相差最大的时候这个差小于等于它们2个进程的`pass`中的最大值，pass所有可能值中的最大值为`BigStride / 2`，故`STRIDE_MAX – STRIDE_MIN <= BigStride / 2`

### 2.2

~~~
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 == other.0 {
            return Some(Ordering::Equal)
        }
        
        if self.0 < other.0 {
            if other.0 - self.0 > BIG_STRIDE / 2 {
                return Some(Ordering::Greater)
            }else {
                return Some(Ordering::Less)
            }
        }
        
        if self.0 > other.0 {
            if self.0 - other.0 > BIG_STRIDE / 2 {
                return Some(Ordering::Less)
            }else {
                return Some(Ordering::Greater)
            }
        }
        
        return None;
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
~~~

# 参考文献
[rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档 第五章 进程调度](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/4scheduling.html)