# axalloc (modified for exercise)

This directory is a modified copy of **`axalloc` 0.3.0-preview.1** from crates.io, used only by [`arceos-altalloc`](../../). 

The parent project uses the `[patch.crates-io]` section in the workspace root `Cargo.toml` to override the published crate, ensuring that all dependent crates utilize this modified version.

## Differences from upstream

- The default byte allocator is [`bump_allocator`](../bump_allocator)’s `EarlyAllocator`, gated by the `bump_allocator` Cargo feature.
- `level-1` is enabled by default: the whole heap region is handed to that byte allocator (`GlobalAllocator::init` only calls `balloc.init`), instead of the stock two-level setup (bitmap page allocator + separate TLSF heap).
- The exact `default` feature set is `bump_allocator`, `level-1`, and `axallocator/page-alloc-256m`.

Optional byte-allocator features (`tlsf`, `slab`, `buddy`) are still present for comparison or switching back. **Do not** enable `bump_allocator` together with another byte-allocator feature: `cfg_if` will match only the first branch.

## Role

Same as upstream: provides [`GlobalAllocator`] implementing [`core::alloc::GlobalAlloc`] for `#[global_allocator]`; traits and helpers come from [axallocator](https://docs.rs/axallocator).

## License

GPL-3.0-or-later OR Apache-2.0 OR MulanPSL-2.0
