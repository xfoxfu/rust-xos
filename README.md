# SYSU DCS216 OS UEFI

## 构建运行

- 需要 `OVMF.fd` 放置在当前目录
- 需要安装 QEMU
- 需要 Rustup

```sh
$ rustup component add rust-src
$ make qemu
```

若不想安装 Rust，可以这样执行：

```sh
qemu-system-x86_64 -bios OVMF.fd -drive format=raw,file=fat:rw:esp -net none
```

压缩包里已经带有编译好的内容了。

## 子模块

### Boot

UEFI 引导操作系统。尚未完成。

### 内存布局

| 起始地址              | 用途     |
| --------------------- | -------- |
| 0xFFFF_8000_0000_0000 | 物理内存 |
| 0xFFFF_FF00_0000_0000 | 内核程序 |
| 0xFFFF_FF01_0000_0000 | 内核栈   |
| 0xFFFF_FF80_0000_0000 | 内核堆   |
