# arceos-sysmap

A standalone [ArceOS](https://github.com/arceos-org/arceos) exercise (`arceos-sysmap`): run a **static musl** Linux user program under minimal syscall emulation and implement **`mmap(2)`** (`SYS_MMAP`) so file-backed mappings work. Builds and QEMU runs use **`cargo xtask`**, with a **cross-compiled payload** and a **FAT32 disk image**.

**This repository is the exercise.**

## Background

The kernel creates a user address space, loads an ELF from the root filesystem at `/sbin/mapfile`, maps a user stack, and spawns a task that enters user mode. Traps from user space are handled in a loop: syscalls are dispatched to `handle_syscall` in `src/syscall.rs`.

The reference payload (`payload/mapfile_c/mapfile.c`) creates a small file, then uses `mmap` with a real file descriptor to read it back. Without a working `SYS_MMAP` path, that program cannot complete successfully.

`xtask` cross-compiles the payload with `<arch>-linux-musl-gcc`, places the binary on a virtio-blk FAT32 image as `/sbin/mapfile`, and runs QEMU with that disk.

## Supported Architectures

| Architecture | Rust Target | QEMU Machine | Platform |
|---|---|---|---|
| riscv64 | `riscv64gc-unknown-none-elf` | `qemu-system-riscv64 -machine virt` | riscv64-qemu-virt |
| aarch64 | `aarch64-unknown-none-softfloat` | `qemu-system-aarch64 -machine virt` | aarch64-qemu-virt |
| x86_64 | `x86_64-unknown-none` | `qemu-system-x86_64 -machine q35` | x86-pc |
| loongarch64 | `loongarch64-unknown-none` | `qemu-system-loongarch64 -machine virt` | loongarch64-qemu-virt |

## Prerequisites

### 1. **Rust nightly toolchain** (edition 2024)

```bash
rustup install nightly
rustup default nightly
```

### 2. **Bare-metal targets** (install the ones you need)

```bash
rustup target add riscv64gc-unknown-none-elf
rustup target add aarch64-unknown-none-softfloat
rustup target add x86_64-unknown-none
rustup target add loongarch64-unknown-none
```

### 3. **QEMU** (install the emulators for your target architectures)

```bash
# Ubuntu/Debian
sudo apt install qemu-system-riscv64 qemu-system-aarch64 \
                 qemu-system-x86 qemu-system-loongarch64  # OR qemu-systrem-misc

# macOS (Homebrew)
brew install qemu
```

### 4. **rust-objcopy** (from `cargo-binutils`, required for non-x86_64 targets)

```bash
cargo install cargo-binutils
rustup component add llvm-tools
```

### 5. **Musl cross-compilation toolchains**

The user-space payload (`mapfile.c`) must be compiled with a musl-based cross-compiler for each target architecture. The `xtask` tool searches for `<arch>-linux-musl-gcc` in your `$PATH`.
  
Download from <https://musl.cc/> or build with [musl-cross-make](https://github.com/richfelker/musl-cross-make):
  
```bash
# Example: RISC-V 64 musl cross-compiler
wget https://musl.cc/riscv64-linux-musl-cross.tgz
tar xzf riscv64-linux-musl-cross.tgz -C /opt/
export PATH="/opt/riscv64-linux-musl-cross/bin:$PATH"
  
# Repeat for other architectures as needed:
# aarch64-linux-musl-cross.tgz
# x86_64-linux-musl-cross.tgz
# loongarch64-linux-musl-cross.tgz
```

## Quick Start

### Get Source Code

Method 1: Get source code from crates.io

```bash
# install cargo-clone sub-command
cargo install cargo-clone
# get source code of arceos-hashmap crate from crates.io
cargo clone arceos-sysmap
# into crate dir
cd arceos-sysmap
```

Method 2: Clone the tg-arceos-tutorial repository

```bash
git clone https://github.com/arceos-org/tg-arceos-tutorial.git
cd tg-arceos-tutorial/exercise-sysmap
```

### Build & Run

```bash
# Build and run on RISC-V 64 QEMU (default)
cargo xtask run

# Build and run on other architectures
cargo xtask run --arch aarch64
cargo xtask run --arch x86_64
cargo xtask run --arch loongarch64

# Build only (no QEMU)
cargo xtask build --arch riscv64
cargo xtask build --arch aarch64
```

## Exercise

### Requirements

Implement the `SYS_MMAP` syscall in `src/syscall.rs`. Follow the Linux `mmap` ABI for each target architecture, and return results to user mode using the same convention as the other syscall handlers in this crate. Your implementation should be enough to ensure that the `mapfile` payload can run successfully.

### Expectation

Example excerpt (RISC-V; syscall numbers vary by architecture):

```
handle_syscall [96] ...
handle_syscall [29] ...
Ignore SYS_IOCTL
handle_syscall [66] ...
MapFile ...
handle_syscall [56] ...
handle_syscall [64] ...
handle_syscall [57] ...
handle_syscall [56] ...
handle_syscall [222] ...
handle_syscall [66] ...
Read back content: hello, arceos!
handle_syscall [57] ...
handle_syscall [66] ...
MapFile ok!
handle_syscall [94] ...
[SYS_EXIT_GROUP]: exiting ..
monolithic kernel exit [Some(0)] normally!
[  0.355631 0:2 axplat_riscv64_qemu_virt::power:25] Shutting down...
```

### Verification

- The serial output must contain **`Read back content: hello, arceos!`**.
- The serial output must contain **`MapFile ok!`**.

Run the test script:

```bash
bash scripts/test.sh
```

### Tips

The user address space for the running task is exposed as `USER_ASPACE` in `src/main.rs`; use `crate::USER_ASPACE` in `syscall.rs` to access it.

## Project Structure

```
exercise-sysmap/
├── .cargo/
│   └── config.toml          # cargo xtask alias
├── configs/
│   ├── riscv64.toml
│   ├── aarch64.toml
│   ├── x86_64.toml
│   └── loongarch64.toml
├── payload/
│   └── mapfile_c/
│       ├── mapfile.c        # musl test program (mmap file)
│       └── target/          # per-arch musl build output (generated)
├── scripts/
│   └── test.sh              # Multi-arch run + expected log line checks
├── src/
│   ├── main.rs              # User ASpace, load /sbin/mapfile, stack, spawn
│   ├── loader.rs            # ELF PT_LOAD loader
│   ├── task.rs              # User task + syscall trap loop
│   └── syscall.rs           # Syscall emulation (implement SYS_MMAP here)
├── xtask/
│   └── src/
│       └── main.rs          # Payload build, FAT image, cargo build, QEMU
├── build.rs
├── Cargo.toml
├── rust-toolchain.toml
└── README.md
```

## Key Components

| Component | Role |
|---|---|
| `axstd` | ArceOS standard library (replaces Rust's `std` in `no_std` environment) |
| `axmm` | User page tables: `map_alloc`, `write`, `unmap`, etc. |
| `axhal` | `UserContext`, `PAGE_SIZE_4K`, `MappingFlags`, uspace traps |
| `axfs` | Open files; `read_at` (or equivalent) for file-backed mmap |
| `axtask` | Spawn user task, kernel stack, `exit` |
| `build.rs` | Locates the linker script generated by `axhal` and passes it to the linker |
| `configs/*.toml` | Pre-generated platform configuration for each architecture |

## License

GPL-3.0
