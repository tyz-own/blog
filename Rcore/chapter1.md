# 应用程序执行环境与平台支持

## 执行应用程序

用 Cargo 攻击创建 Rust 项目：

```
cargo new os
```

输入 cargo run 构建并运行项目：
```
   Compiling os v0.1.0 (/home/shinbokuow/workspace/v3/rCore-Tutorial-v3/os)
    Finished dev [unoptimized + debuginfo] target(s) in 1.15s
     Running `target/debug/os`
Hello, world!
```
## 理解应用程序执行环境
在现代通用操作系统（如 Linux）上运行应用程序，需要多层次的执行环境栈支持：
![alt text](./image/image.png)

## 平台与目标三元组
编译器在编译、链接得到可执行文件时需要知道，程序要在哪个 平台 (Platform) 上运行， 目标三元组 (Target Triplet) 描述了目标平台的 CPU 指令集、操作系统类型和标准运行时库。

我们研究一下现在 `Hello, world!` 程序的目标三元组是什么：
```
$ rustc --version --verbose
   rustc 1.61.0-nightly (68369a041 2022-02-22)
   binary: rustc
   commit-hash: 68369a041cea809a87e5bd80701da90e0e0a4799
   commit-date: 2022-02-22
   host: x86_64-unknown-linux-gnu
   release: 1.61.0-nightly
   LLVM version: 14.0.0
```
其中 host 一项表明默认目标平台是 `x86_64-unknown-linux-gnu`， CPU 架构是 x86_64，CPU 厂商是 unknown，操作系统是 linux，运行时库是 gnu libc。

接下来，我们希望把 `Hello, world!` 移植到 RICV 目标平台` riscv64gc-unknown-none-elf` 上运行。

## 修改目标平台
将程序的目标平台换成 `riscv64gc-unknown-none-elf`，试试看会发生什么：
```
$ cargo run --target riscv64gc-unknown-none-elf
   Compiling os v0.1.0 (/home/shinbokuow/workspace/v3/rCore-Tutorial-v3/os)
error[E0463]: can't find crate for `std`
  |
  = note: the `riscv64gc-unknown-none-elf` target may not be installed
```
报错的原因是目标平台上确实没有 Rust 标准库 std，也不存在任何受 OS 支持的系统调用。 这样的平台被我们称为 裸机平台 (bare-metal)。
