# tg-arceos-tutorial

`tg-arceos-tutorial` 是一个集合 crate，用于把与`arceos`相关的 `app-*` 和 `exercise-*` 教学 crate 的源码打包到一个压缩包里，便于通过 `cargo clone` 后离线解包恢复完整目录。

## 准备

```bash
cargo clone tg-arceos-tutorial # 或 git@github.com:rcore-os/tg-arceos-tutorial.git
cd tg-arceos-tutorial
bash scripts/extract_crates.sh
```

解包后会在当前目录生成 20 个 crate 目录，包括 15 个 `app-*` 和 5 个 `exercise-*`。

## `app-*` 教学示例

### unikernel
- `app-helloworld` : https://github.com/arceos-org/app-helloworld
- `app-collections` : https://github.com/arceos-org/app-collections
- `app-readpflash` : https://github.com/arceos-org/app-readpflash
- `app-childtask` : https://github.com/arceos-org/app-childtask
- `app-msgqueue` : https://github.com/arceos-org/app-msgqueue
- `app-fairsched` : https://github.com/arceos-org/app-fairsched
- `app-readblk` : https://github.com/arceos-org/app-readblk
- `app-loadapp` : https://github.com/arceos-org/app-loadapp
### monolithic kernel
- `app-userprivilege` : https://github.com/arceos-org/app-userprivilege
- `app-lazymapping` : https://github.com/arceos-org/app-lazymapping
- `app-runlinuxapp` : https://github.com/arceos-org/app-runlinuxapp
### hypervisor
- `app-guestmode` : https://github.com/arceos-org/app-guestmode
- `app-guestaspace` : https://github.com/arceos-org/app-guestaspace
- `app-guestvdev` : https://github.com/arceos-org/app-guestvdev
- `app-guestmonolithickernel` : https://github.com/arceos-org/app-guestmonolithickernel

## `exercise-*` 实验内容

| 目录 | 仓库地址 | 实验内容说明 |
|------|----------------|------|
| `exercise-printcolor` | https://github.com/arceos-org/exercise-printcolor | unikernel，彩色终端输出（ANSI） |
| `exercise-hashmap` | https://github.com/arceos-org/exercise-hashmap | unikernel，在 `axstd` 中实现 `collections::HashMap` |
| `exercise-altalloc` | https://github.com/arceos-org/exercise-altalloc | unikernel，实现 bump 式内存分配器 |
| `exercise-ramfs-rename` | https://github.com/arceos-org/exercise-ramfs-rename | unikernel，ramfs 根文件系统上的 `rename` 支持 |
| `exercise-sysmap` | https://github.com/arceos-org/exercise-sysmap | monolithic kernel，用户态程序与 `mmap` 系统调用实现 |

## 运行

进入任意 `app-*` 或 `exercise-*` 目录即可独立构建/运行，例如：

```bash
cd tg-arceos-tutorial/app-helloworld
cargo xtask run # 或 cargo xtask run --arch=riscv64
cargo xtask run --arch=aarch64
cargo xtask run --arch=loongarch64
cargo xtask run --arch=x86_64
```

### 批量执行

在仓库根目录（已解包、且各 crate 目录存在）下，可按前缀批量执行同一命令：

- 对所有 `app-*`：`./scripts/batch_app_exec.sh -c "cargo xtask run"`
- 对所有 `exercise-*`：`./scripts/batch_exercise_exec.sh -c "cargo xtask run"`

其它示例：

```bash
./scripts/batch_app_exec.sh -c "cargo xtask run --arch=aarch64"
./scripts/batch_exercise_exec.sh -c "bash scripts/test.sh"
```

## 维护者：重新生成 bundle

在根目录中执行：

```bash
cd tg-arceos-tutorial
bash scripts/compress_crates.sh
```

将生成：

- `bundle/apps.tar.gz`

该压缩包会被 `Cargo.toml` 的 `include` 字段打包进发布产物。