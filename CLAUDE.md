# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ArceOS tutorial (test branch): 15 `app-*` teaching crates and 5 `exercise-*` exercises that progressively teach OS development — from bare-metal unikernel to monolithic kernel and hypervisor. Each crate builds and runs standalone via QEMU.

## Build & Run Commands

Each `app-*` or `exercise-*` directory is an independent project. Always `cd` into the directory first.

```bash
cargo xtask run                    # default: riscv64
cargo xtask run --arch aarch64
cargo xtask run --arch x86_64
cargo xtask run --arch loongarch64
cargo xtask build --arch riscv64   # build only
```

Batch execution from repo root:
```bash
./scripts/batch_app_exec.sh -c "cargo xtask run"
./scripts/batch_exercise_exec.sh -c "cargo xtask run"
```

Verification is done by running in QEMU and checking console output. Many exercises have `scripts/test.sh` for multi-arch testing.

## Toolchain

- Rust nightly (edition 2024), pinned in each crate's `rust-toolchain.toml`
- `cargo-binutils` (provides `rust-objcopy` for non-x86_64 targets)
- QEMU for target architectures
- `exercise-sysmap` requires musl cross-compilers (from musl.cc)

## Architecture

### cargo xtask pattern

Each crate has a host-native `xtask/src/main.rs` binary:
1. Copies `configs/<arch>.toml` → `.axconfig.toml`
2. Cross-compiles with `cargo build --release --target <rust-target>`
3. Converts ELF to raw binary via `rust-objcopy` (except x86_64)
4. Launches architecture-specific QEMU

### Key dependencies

- **`axstd`** — ArceOS standard library, replaces `std` in `no_std`
- **`axhal`** — Hardware abstraction layer
- **`axfeat`** — Feature aggregator routing to sub-crates
- **`axalloc`** — Global memory allocator
- **`axfs`** — Filesystem (FAT, ramfs)
- ArceOS crate version: `0.3.0-preview.1` (exercise-sysmap uses `0.3.0-preview.3`)

## Exercises

| Exercise | What to implement | Expected output |
|----------|-------------------|-----------------|
| `exercise-printcolor` | ANSI color codes in println | Colored "Hello, Arceos!" |
| `exercise-hashmap` | Add HashMap to axstd::collections | `test_hashmap() OK!` |
| `exercise-altalloc` | Bump allocator (3 traits) | `Bump tests run OK!` |
| `exercise-ramfs-rename` | VFS rename in ramfs | `[Ramfs-Rename]: ok!` |
| `exercise-sysmap` | sys_mmap syscall | `MapFile ok!` |

### Patching pattern

Several exercises require modifying published ArceOS crates. The pattern:
1. Copy the published crate source to a local directory (e.g., `patches/axstd/`)
2. Make the modification (add feature, implement method)
3. Add `[patch.crates-io]` in the exercise's `Cargo.toml`

This preserves external dependency compatibility while adding needed functionality.
