# easy-fs

linur's on-disk filesystem: the durable half of the kernel's VFS, extracted into its
own repository so it can be tested on the host. It is `#![no_std]` + `alloc` with no
dependencies, so the kernel and the host test run the same code. It descends from the
rCore-Tutorial-v3 `easy-fs` (see [NOTICE](NOTICE)), extended with ownership and file
types, hard links, `rename`, directory shrink and a hash-queue block cache with
delayed write.

The on-disk format (v3) is specified in the kernel's
[docs/kernel/vfs.md](https://github.com/linur-io/linur/blob/main/docs/kernel/vfs.md) §2; this
repository does not restate it.

## Layout

| Path | What |
| --- | --- |
| `src/lib.rs` | crate root: `BLOCK_SZ`, the re-exports |
| `src/block_dev.rs` | `BlockDevice`: the trait a device implements |
| `src/block_cache.rs` | the 64-entry hash-queue block cache, `block_cache_sync_all` |
| `src/bitmap.rs` | the inode and data block bitmaps |
| `src/layout.rs` | on-disk structures: super block, disk inode, directory entry |
| `src/efs.rs` | `EasyFileSystem`: create and open, block and inode allocation |
| `src/vfs.rs` | `Inode`: the path-level operations |
| `src/sync.rs` | `SpinMutex`: the crate's own lock, not the kernel's |
| `host/src/lib.rs` | `easy-fs-host`: the std, file-backed `BlockFile` |
| `host/tests/efs.rs` | the filesystem test, run by `cargo test` |

## Public API

* `EasyFileSystem` — `create` / `open` over an `Arc<dyn BlockDevice>`,
  `root_inode`, and the block and inode allocation the `Inode` layer builds on
  (`alloc_inode`, `alloc_data`, `dealloc_*`, `free_data_blocks`,
  `get_disk_inode_pos`, `get_inode_id`).
* `Inode` — `find`, `create_file`, `create_dir`, `create_node`, `remove`,
  `link`, `rename`, `dirent_at`, `read_at`, `write_at`, `truncate`, plus the
  metadata accessors `mode`, `uid`, `gid`, `dev`, `size`, `nlink`, `is_dir`,
  `is_file`, `inode_id` and the setters `set_mode`, `set_owner`.
* `BlockDevice` — `read_block` / `write_block`, one `BLOCK_SZ` (512-byte) block
  per call.
* `VfsError` — `NotFound`, `Exists`, `NotEmpty`, `NotDirectory`, `IsDirectory`,
  `InvalidName`.
* `S_IFMT`, `S_IFREG`, `S_IFDIR`, `S_IFCHR`, `S_IFBLK`, `S_IFIFO` and
  `makedev` — the mode-word type bits and `(major << 8) | minor`, the same encoding
  the kernel's `stat` reports.
* `BLOCK_SZ`, `block_cache_sync_all`, `SpinMutex`, `Dirent`,
  `NAME_LENGTH_LIMIT`.

## One filesystem per process

The block cache is a process-wide singleton keyed by block id alone: there is one
64-entry cache and it does not record which device a block came from. Two
`EasyFileSystem` instances opened in one process on different devices would alias
each other's blocks, so a process may have exactly one filesystem open at a time.
The kernel mounts this filesystem at `/` and lives with the constraint.

For tests this means a single `#[test]` function: `cargo test` runs the tests of one
binary as threads of one process, and two of them would race over the cache.
`host/tests/efs.rs` is therefore one test that builds and exercises a whole image in
order; it uses `target/fs.img` (cargo runs integration tests with the package root,
`host/`, as the working directory, and `target/` is ignored).

## How the kernel consumes it

The kernel repository keeps this one as a git submodule at `fs/`, next to the
bootloader, the allocator and the user library, and depends on the path `fs` as the
`easy-fs` package. Bumping the filesystem is a submodule commit plus the kernel's
`Cargo.lock`; the kernel's tree carries the image packer (`easy-fs-fuse`), not this
repository.

## Build and test

```sh
rustup target add riscv64gc-unknown-none-elf
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --target riscv64gc-unknown-none-elf -p easy-fs
```

CI runs exactly these. The host adapter is a development and test device; the kernel
supplies its own `BlockDevice` over virtio-blk.

## License

GPL-3.0-only. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
