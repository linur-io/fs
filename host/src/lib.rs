// SPDX-License-Identifier: GPL-3.0-only
//! The std host side of easy-fs: a file-backed [`BlockDevice`].
#![deny(missing_docs)]

use easy_fs::{BLOCK_SZ, BlockDevice};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A block device backed by one file of 512-byte blocks.
pub struct BlockFile {
    file: Mutex<File>,
    reads: AtomicUsize,
    writes: AtomicUsize,
}

impl BlockFile {
    /// Create or truncate the path as an image of `blocks` 512-byte blocks.
    pub fn create(path: impl AsRef<Path>, blocks: usize) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.set_len((blocks * BLOCK_SZ) as u64)?;
        Ok(Self {
            file: Mutex::new(file),
            reads: AtomicUsize::new(0),
            writes: AtomicUsize::new(0),
        })
    }

    /// Device reads so far.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }

    /// Device writes so far.
    pub fn writes(&self) -> usize {
        self.writes.load(Ordering::Relaxed)
    }
}

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.writes.fetch_add(1, Ordering::Relaxed);
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}
