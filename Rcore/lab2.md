# chapter4练习

## 功能实现概述

本次练习我通过添加对虚拟地址到物理地址的转化调用重写了 `sys_get_time` 和 `sys_task_info` 系统调用，并实现了 `mmap` 和 `munmap ` 系统调用。在实现 `mmap` 和 `munmap` 系统调用时，将可能的错误进行排除后，将起始地址和终止地址按页对齐并进行页表项的分配与回收，`mmap` 分配过程中由于参数 `port` 与 `MapPermission` 有些许不同，需进行微调。`munmap` 函数执行过程中，由于代码未给出相应回收代码，因此需要自己在 `memory_set` 中进行实现，即寻找区域相应位置并进行删除。



## 简答作业

### 第 1 题

SV39 分页模式下的页表项，其中 [53:10] 这44位是物理页号，最低的 8 位 [7:0] 位则是标志位。标志位对应着一个页表的属性。由低到高依次为 V (页表项是否合法) 、R/W/X(对应虚拟页面是否可读/写/取指) 、 U(当前页表项是否允许用户态访问) 、G(不知道) 、A (当前页表项对应虚拟页表是否访问过) 、D (当前页表项对应虚拟页表是否修改过)。

### 第 2 题

1.  `Load Access Fault`（访存错误）、`Store Access Fault` （存储错误）、 `Instruction Access Fault` （指令访问错误）、 `Load Page Fault` （缺页异常）可能是缺页导致的；
    发生缺页时，`scause` 描述异常的原因， `stval` 给出异常的附加信息，  `stvec` 控制异常处理代码的入口地址。

2.  好处：使编译期所需要的时间缩减，同时页降低了在运行时所耗费的时间和内存资源，因为页面只有在有需求时才会被调入主存中，对于没有需求的页面则不会被调入主存。

3.  处理10G连续的内存页面， 对应的 SV39 页表大致占用 20~25 MB 的内存。
    若要实现 Lazy 策略，需要先将 .text 段在磁盘的位置信息保存在内存中相应位置，等需要调用缺失的页面时，执行缺页中断并将相应页面通过其在磁盘中的位置读入内存中，并映射到 `memory_set` 中，随后继续执行相应代码。

4.  置页表项(PTE)的 V 标志位为 0，即表示页面失效。

### 第 3 题

1.  在单页表的情况下，通过要被替换的页面确定及其虚拟地址，随后将被替换的物理页即相关信息移除并将信的物理页移入相应位置。

2.  ☞内核页面相应的页表项（PTE）的 U 标志位为 0，即表示页面不可被用户态访问。

3.  单页表管理起来较为容易，且不用频繁切换页表。

4.  当进行用户态/内核态切换或进程切换时时需要更换页表。 若我写一个单页表操作系统，我会选择在线程切换时更换页表。
   
## 荣誉守则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    暂无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    暂无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。