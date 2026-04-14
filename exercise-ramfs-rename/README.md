# arceos-ramfs-rename

A standalone [ArceOS](https://github.com/arceos-org/arceos) unikernel exercise (`arceos-ramfs-rename`): use ramfs as the root filesystem and implement `rename` support, with dependencies from [crates.io](https://crates.io), multi-architecture builds and QEMU runs via `cargo xtask`.

**This repository is the exercise.**

## Background: how ramfs is mounted as the root filesystem

At runtime, `axruntime` initializes the filesystem layer and passes the discovered block devices to `axfs`. `axfs` then scans the disk (such as via a partition scan). If this scan finds no usable partitions—for example, if the virtio block device reports zero capacity—`axfs` follows the path that mounts ramfs as the root filesystem: instead of a disk-backed root, it constructs an in-memory tree using the `axfs_ramfs` crate (`RamFileSystem`).

In this exercise, `xtask` creates `target/disk.img` (zero bytes) and passes it to QEMU as the virtio-blk backing file. The kernel therefore detects a block device but finds its capacity is zero, and the partition scan yields no results. As a result, `axfs` will automatically mount `RamFileSystem` as the root filesystem in memory.

## Supported Architectures

| Architecture | Rust Target | QEMU Machine | Platform |
|---|---|---|---|
| riscv64 | `riscv64gc-unknown-none-elf` | `qemu-system-riscv64 -machine virt` | riscv64-qemu-virt |
| aarch64 | `aarch64-unknown-none-softfloat` | `qemu-system-aarch64 -machine virt` | aarch64-qemu-virt |
| x86_64 | `x86_64-unknown-none` | `qemu-system-x86_64 -machine q35` | x86-pc |
| loongarch64 | `loongarch64-unknown-none` | `qemu-system-loongarch64 -machine virt` | loongarch64-qemu-virt |

## Prerequisites

- **Rust nightly toolchain** (edition 2024)

  ```bash
  rustup install nightly
  rustup default nightly
  ```

- **Bare-metal targets** (install the ones you need)

  ```bash
  rustup target add riscv64gc-unknown-none-elf
  rustup target add aarch64-unknown-none-softfloat
  rustup target add x86_64-unknown-none
  rustup target add loongarch64-unknown-none
  ```

- **QEMU** (install the emulators for your target architectures)

  ```bash
  # Ubuntu/Debian
  sudo apt install qemu-system-riscv64 qemu-system-aarch64 \
                   qemu-system-x86 qemu-system-loongarch64  # OR qemu-system-misc

  # macOS (Homebrew)
  brew install qemu
  ```

- **rust-objcopy** (from `cargo-binutils`, required for non-x86_64 targets)

  ```bash
  cargo install cargo-binutils
  rustup component add llvm-tools
  ```

## Quick Start

### Get Source Code

Method 1: Get source code from crates.io

```bash
# install cargo-clone sub-command
cargo install cargo-clone
# get source code of arceos-ramfs-rename crate from crates.io
cargo clone arceos-ramfs-rename
# into crate dir
cd arceos-ramfs-rename
```

Method 2: Clone the tg-arceos-tutorial repository

```bash
git clone https://github.com/arceos-org/tg-arceos-tutorial.git
cd tg-arceos-tutorial/exercise-ramfs-rename
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

The crates.io releases of `axfs` and `axfs_ramfs` do not fully implement `rename` operation. As a result, using `std::fs::rename` in the unikernel will fail when working with ramfs paths.

Therefore, you need to modify and use local versions of the relevant crates to add support for the `rename` operation.

### Expectation

Serial output should include:

```
[Ramfs-Rename]: ok!
```

### Verification

- The serial output must contain **`[Ramfs-Rename]: ok!`**.

Run the test script:

```bash
bash scripts/test.sh
```

### Tips

1. If compiler errors mention `std`, that name refers to **`axstd`**: the application uses `extern crate axstd as std;` in `src/main.rs`.
2. **`std::fs::rename`** goes through **`axfs`** into **`axfs_ramfs`**: you need **`VfsNodeOps::rename`** on the ramfs directory node, and the composite root in **`axfs`** must forward **`rename`** the same way it does for **`create`** / **`remove`**.
3. To hack on ArceOS crates locally, clone the crates and override them with **`[patch.crates-io]`** in this exercise’s `Cargo.toml`:

```bash
cargo clone axfs@0.3.0-preview.1
cargo clone axfs_ramfs@0.1.2
```

```toml
[patch.crates-io]
axfs = { path = "./axfs" }
axfs_ramfs = { path = "./axfs_ramfs" }
```

(Adjust the path to match where you put the sources.)

## Project Structure

```
exercise-ramfs-rename/
├── .cargo/
│   └── config.toml       # cargo xtask alias
├── configs/
│   ├── riscv64.toml
│   ├── aarch64.toml
│   ├── x86_64.toml
│   └── loongarch64.toml
├── scripts/
│   └── test.sh           # Multi-arch run + expected log line checks
├── src/
│   └── main.rs           # Application entry (ramfs rename demo)
├── xtask/
│   └── src/
│       └── main.rs       # build/run and QEMU + disk image wiring
├── build.rs              # Linker script path (arch auto-detect)
├── Cargo.toml            # Crate manifest; add [patch.crates-io] when using local axfs / axfs_ramfs
├── rust-toolchain.toml   # Nightly, targets, llvm-tools
└── README.md
```

## Key Components

| Component | Role |
|---|---|
| `axstd` | ArceOS standard library (`std::fs` maps onto axfs) |
| `axfs` | Filesystem module: root layout, partition scan, ramfs bootstrap |
| `axfs_ramfs` | In-memory ramfs implementation (`RamFileSystem` / directory nodes) |
| `axhal` | Hardware abstraction layer, linker script at build time |
| `build.rs` | Locates the linker script from `axhal` for the link step |
| `configs/*.toml` | Pre-generated platform configuration per architecture |
| `xtask` | Builds the kernel and runs QEMU with virtio-blk + empty backing file |

## License

GPL-3.0
