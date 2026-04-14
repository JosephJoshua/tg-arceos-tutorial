# ArceOS 五个练习实验报告

## 背景

`tg-arceos-tutorial` 的 `test` 分支包含 15 个 `app-*` 教学示例和 5 个 `exercise-*` 练习。每个 exercise 是独立的 crate，通过 `cargo xtask run` 在 QEMU 中运行，通过 `scripts/test.sh` 在四个架构（riscv64、aarch64、x86_64、loongarch64）上验证。

ArceOS crate 版本为 `0.3.0-preview.1`（exercise-sysmap 用 `0.3.0-preview.3`）。

---

## 练习 1：exercise-printcolor — 彩色输出

### 要求

在 `println!` 输出中添加 ANSI 颜色转义序列，使 "Hello, Arceos!" 显示彩色。

### 实现

在 `src/main.rs` 的 `println!` 字符串中嵌入 `\x1b[32m`（绿色前景）和 `\x1b[0m`（重置）：

```rust
println!("[WithColor]: \x1b[32mHello, Arceos!\x1b[0m");
```

ANSI 转义码直接通过 QEMU 虚拟串口输出到终端。在 `no_std` 环境下无法使用 `colored` 等 crate，但字节级转义序列可以正常工作。

### 验证

测试脚本用 `grep -P '\x1b\[[0-9;]*[1-9][0-9;]*m'` 验证输出包含 ANSI SGR 序列。

---

## 练习 2：exercise-hashmap — 为 axstd 添加 HashMap 支持

### 要求

使 `extern crate axstd as std; use std::collections::HashMap;` 能编译通过。

### 实现

练习的本意是修改 `axstd` 源码。由于 axstd 来自 crates.io 不可直接修改，采用本地 patch：

1. 将 `axstd-0.3.0-preview.1` 源码复制到 `patches/axstd/`
2. 修改 `lib.rs`，将 `pub use alloc::collections;` 替换为自定义模块：
   ```rust
   pub mod collections {
       pub use alloc::collections::*;
       pub use hashbrown::{HashMap, HashSet};
   }
   ```
3. 在 `patches/axstd/Cargo.toml` 添加 `hashbrown = "0.16"` 依赖
4. 在练习 `Cargo.toml` 添加 `[patch.crates-io]` 指向 `patches/axstd`

`alloc::collections` 不包含 `HashMap` 是因为 HashMap 的默认 hasher（`RandomState`）依赖随机数源，而 `alloc` crate 无法提供 OS 级随机性。`hashbrown` 是 Rust 标准库内部使用的 hash table 实现，在 `no_std` 下可独立工作。

### 验证

```
test_hashmap() OK!
Memory tests run OK!
```

---

## 练习 3：exercise-altalloc — Bump Allocator 作为全局分配器

### 要求

实现 bump allocator，同时满足 `BaseAllocator`、`ByteAllocator`、`PageAllocator` 三个 trait，作为系统全局内存分配器运行 3M 元素的 Vec 排序。

### 实现

练习已预配置好集成：`modules/axalloc` 是 vendored 的 axalloc 版本，通过 `[patch.crates-io]` 替换官方 axalloc，并在 `cfg_if!` 块中将 bump allocator 设为 `DefaultByteAllocator`。只需填写 `modules/bump_allocator/src/lib.rs` 中的 `todo!()` 桩。

双端内存布局：

```
[ bytes-used | avail-area | pages-used ]
|            | -->    <-- |            |
start       b_pos        p_pos       end
```

关键实现：

| 方法 | 实现 |
|------|------|
| `ByteAllocator::alloc` | 将 `b_pos` 对齐上调后推进，检查不超过 `p_pos`，递增 `count` |
| `ByteAllocator::dealloc` | `count.saturating_sub(1)`；归零时重置 `b_pos = start`（批量释放） |
| `PageAllocator::alloc_pages` | 从 `p_pos` 向后分配，对齐下调，检查不低于 `b_pos` |
| `PageAllocator::dealloc_pages` | 空操作（页永不释放） |

`dealloc` 使用 `saturating_sub` 防止 release 模式下 `count` 下溢到 `usize::MAX`。

### 验证

```
Running bump tests...
Bump tests run OK!
```

`Vec::with_capacity(3_000_000)` 分配约 24MB，通过 bump allocator 的 `ByteAllocator::alloc` 完成。排序过程中的临时分配和释放测试了 count-based 批量释放机制。

---

## 练习 4：exercise-ramfs-rename — RAM 文件系统 Rename

### 要求

在 ramfs 中实现 `rename` 操作，使 `std::fs::rename()` 可用。

### 实现

`axfs_ramfs` 的 `DirNode` 默认不实现 `rename`（`VfsNodeOps::rename` 返回 `Unsupported`）。需要 patch 两个 crate：

**axfs_ramfs**（`exercise-ramfs-rename/axfs_ramfs/`）：

在 `DirNode` 中添加：
- `rename_node(src_name, dst_name)` — 在同一目录内重命名（单锁操作）
- `navigate_to(path)` — 从当前目录沿路径导航到目标节点
- `VfsNodeOps::rename` 实现 — 解析源/目标路径，导航到各自的父目录，从源目录的 `children` BTreeMap 移除条目并插入目标目录

```rust
fn rename(&self, src_path: &str, dst_path: &str) -> VfsResult {
    let (src_dir_path, src_name) = split_parent_path(src_path);
    let (dst_dir_path, dst_name) = split_parent_path(dst_path);
    let src_dir = self.navigate_to(src_dir_path)?;
    let dst_dir = self.navigate_to(dst_dir_path)?;
    // downcast to DirNode, remove from src, insert into dst
}
```

**axfs**（`exercise-ramfs-rename/axfs/`）：

在 `src/root.rs` 的 `RootDirectory` 的 `VfsNodeOps` 实现中添加 `rename` 方法，将调用委托给 `main_fs.root_dir().rename()`。

两个 patch 通过 `[patch.crates-io]` 引入。

### 验证

```
Create '/tmp/f1' and write [hello] ...
Read '/tmp/f1' content: [hello] ok!
Rename '/tmp/f1' to '/tmp/f2' ...
Read '/tmp/f2' content: [hello] ok!

[Ramfs-Rename]: ok!
```

---

## 练习 5：exercise-sysmap — sys_mmap 系统调用

### 要求

实现 `sys_mmap` 系统调用（riscv64 上编号 222），支持文件映射，使 musl-libc 编译的 C 程序能通过 `mmap()` 读取文件内容。

### 实现

练习中所有其它系统调用（open、close、read、write、writev、brk、exit 等）已实现，只有 `sys_mmap` 是 `unimplemented!()` 桩。

C 测试程序流程：
1. `creat("test_file")` + `write(fd, "hello, arceos!")` — 创建文件
2. `open("test_file", O_RDONLY)` — 重新打开
3. `mmap(NULL, 32, PROT_READ, MAP_PRIVATE, fd, 0)` — 映射到内存
4. `printf("%s", addr)` — 从映射地址读取并打印

`sys_mmap` 实现：

```rust
static MMAP_ADDR: AtomicUsize = AtomicUsize::new(0x1000_0000);

fn sys_mmap(addr, length, prot, flags, fd, offset) -> isize {
    let aligned_length = (length + 0xFFF) & !0xFFF;
    let vaddr = MMAP_ADDR.fetch_add(aligned_length, Ordering::Relaxed);

    // 1. 通过 USER_ASPACE 获取用户地址空间
    // 2. map_alloc() 分配物理页并映射到 vaddr
    // 3. 对于文件映射：用 with_file_fd() 读文件内容
    // 4. aspace.write() 将内容写入映射区域
    // 5. 返回 vaddr
}
```

使用 `AtomicUsize` 单调递增计数器分配虚拟地址。匿名映射（`MAP_ANONYMOUS`）只分配页，文件映射额外读取文件内容并写入。

### 验证

```
handle_syscall [222] ...
sys_mmap: length=0x20, prot=PROT_READ, flags=MAP_PRIVATE, fd=3
Read back content: hello, arceos!
MapFile ok!
monolithic kernel exit [Some(0)] normally!
```
