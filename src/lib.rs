//! Rust implementation of [Constant Quark Database](http://www.chokkan.org/software/cqdb/):
//! a database library specialized for serialization and retrieval of static associations between strings and integer identifiers
use std::{
    fmt,
    io::{self, Seek, SeekFrom, Write},
    mem,
};

use bitflags::bitflags;
use bstr::{BStr, ByteSlice};

mod hash;

const CHUNK_ID: &[u8; 4] = b"CQDB";
const BYTEORDER_CHECK: u32 = 0x62445371;
const NUM_TABLES: usize = 256;

bitflags! {
    /// CQDB writer flag
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Flag: u32 {
        /// No flag, default
        const NONE = 0;
        /// A reverse lookup array is omitted
        const ONEWAY = 0x00000001;
    }
}

/// Read a little-endian u32 directly from a buffer at the given offset.
/// Uses a single slice bounds check instead of 4 individual byte accesses.
/// Panics on out-of-bounds (callers must validate buffer structure upfront).
#[inline(always)]
fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
    let b = &buf[offset..offset + 4];
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

#[inline(always)]
fn pack_u32(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Zero-copy hash table reference into the buffer
#[derive(Debug, Clone, Copy, Default)]
struct ReadTable {
    /// Offset into buffer where the bucket array starts
    offset: usize,
    /// Number of elements in the hash table
    num: u32,
}

/// Constant quark database (CQDB)
#[derive(Clone)]
pub struct CQDB<'a> {
    /// Database file buffer
    buffer: &'a [u8],
    /// Chunk header
    header: Header,
    /// Hash tables (string -> id), zero-copy references into buffer
    tables: [ReadTable; NUM_TABLES],
    /// Offset to the backward link array in the buffer (0 if none)
    bwd_offset: usize,
    /// Number of key/data pairs
    num: u32,
}

/// CQDB chunk header
#[derive(Debug, Clone)]
#[repr(C)]
struct Header {
    /// Chunk identifier, "CQDB"
    chunk_id: [u8; 4],
    /// Chunk size including this header
    size: u32,
    /// Global flags
    flag: u32,
    /// Byte-order indicator
    byteorder: u32,
    /// Number of elements in the backward array
    bwd_size: u32,
    /// Offset to the backward array
    bwd_offset: u32,
}

/// A hash table (used by writer)
#[derive(Debug, Clone, Default)]
struct Table {
    size: usize,
    /// Number of elements in the table
    num: u32,
    /// Array of Bucket
    bucket: Vec<Bucket>,
}

#[repr(C)]
struct TableRef {
    /// Offset to a hash table
    offset: u32,
    /// Number of elements in the hash table
    num: u32,
}

/// An element of a hash table
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct Bucket {
    /// Hash value of the record
    hash: u32,
    /// Offset address to the actual record
    offset: u32,
}

/// Writer for a constant quark database
pub struct CQDBWriter<T: Write + Seek> {
    writer: T,
    /// Operation flag
    flag: Flag,
    /// Offset address to the head of this database
    begin: u32,
    /// Offset address to a new key/data pair
    current: u32,
    /// Hash tables (string -> id)
    tables: [Table; NUM_TABLES],
    /// Backlink array
    bwd: Vec<u32>,
    bwd_num: u32,
    /// Number of elements in the backlink array
    bwd_size: u32,
}

impl<'a> fmt::Debug for CQDB<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CQDB")
            .field("header", &self.header)
            .field("bwd_offset", &self.bwd_offset)
            .field("num", &self.num)
            .finish()
    }
}

impl<T: Write + Seek + fmt::Debug> fmt::Debug for CQDBWriter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CQDBWriter")
            .field("writer", &self.writer)
            .field("flag", &self.flag)
            .field("begin", &self.begin)
            .field("current", &self.current)
            .field("bwd", &self.bwd)
            .field("bwd_num", &self.bwd_num)
            .field("bwd_size", &self.bwd_size)
            .finish()
    }
}

impl<'a> CQDB<'a> {
    pub fn new(buf: &'a [u8]) -> io::Result<Self> {
        let min_size = mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES;
        if buf.len() < min_size {
            // The minimum size of a valid CQDB
            return Err(io::Error::other("invalid file format"));
        }
        // Check the file chunkid
        if &buf[0..4] != CHUNK_ID {
            return Err(io::Error::other("invalid file format, magic mismatch"));
        }
        let chunk_size = read_u32_le(buf, 4);
        let flag = read_u32_le(buf, 8);
        let byte_order = read_u32_le(buf, 12);
        // Check the consistency of byte order
        if byte_order != BYTEORDER_CHECK {
            return Err(io::Error::other("invalid file format, byte order mismatch"));
        }
        let bwd_size = read_u32_le(buf, 16);
        let bwd_offset_raw = read_u32_le(buf, 20);
        let header = Header {
            chunk_id: *CHUNK_ID,
            size: chunk_size,
            flag,
            byteorder: byte_order,
            bwd_size,
            bwd_offset: bwd_offset_raw,
        };

        // Parse table references (zero-copy: just store offset + count)
        let mut num_db = 0u32;
        let mut tables = [ReadTable::default(); NUM_TABLES];
        let mut index = 24; // After 6 × u32 header fields
        for table in &mut tables {
            let table_offset = read_u32_le(buf, index) as usize;
            index += 4;
            let table_num = read_u32_le(buf, index);
            index += 4;
            if table_offset > 0 {
                // Validate that bucket data fits within the buffer
                let end = table_offset + (table_num as usize) * 8;
                if end > buf.len() {
                    return Err(io::Error::other("invalid table data: out of bounds"));
                }
                table.offset = table_offset;
                table.num = table_num;
            }
            // The number of records is the half of the table size
            num_db += table_num / 2;
        }

        // Validate backward link array bounds
        let bwd_offset = if bwd_offset_raw > 0 {
            let off = bwd_offset_raw as usize;
            let end = off + (bwd_size as usize) * 4;
            if end > buf.len() {
                return Err(io::Error::other(
                    "invalid backward link data: out of bounds",
                ));
            }
            off
        } else {
            0
        };

        Ok(Self {
            buffer: buf,
            header,
            tables,
            bwd_offset,
            num: num_db,
        })
    }

    /// Get the number of associations in the database
    #[inline]
    pub fn num(&self) -> u32 {
        self.num
    }

    /// Retrieve the identifier associated with a string
    #[inline]
    pub fn to_id(&self, s: &str) -> Option<u32> {
        let hash = crate::hash::jhash(s.as_bytes(), s.len() as u32 + 1, 0);
        let table = &self.tables[(hash % NUM_TABLES as u32) as usize];
        if table.num > 0 {
            let n = table.num;
            let base = table.offset;
            let mut k = (hash >> 8) % n;
            loop {
                // Single bounds check for both hash + offset (8 bytes)
                let bk = &self.buffer[base + (k as usize) * 8..][..8];
                let bucket_offset = u32::from_le_bytes([bk[4], bk[5], bk[6], bk[7]]);
                if bucket_offset > 0 {
                    let bucket_hash = u32::from_le_bytes([bk[0], bk[1], bk[2], bk[3]]);
                    if bucket_hash == hash {
                        // Single bounds check for record header (8 bytes)
                        let rec = &self.buffer[bucket_offset as usize..][..8];
                        let value = u32::from_le_bytes([rec[0], rec[1], rec[2], rec[3]]);
                        let ksize =
                            u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]) as usize - 1;
                        let key_start = bucket_offset as usize + 8;
                        if s.as_bytes() == &self.buffer[key_start..key_start + ksize] {
                            return Some(value);
                        }
                    }
                } else {
                    break;
                }
                k = (k + 1) % n;
            }
        }
        None
    }

    /// Retrieve the string associated with an identifier
    #[inline]
    pub fn to_str(&'a self, id: u32) -> Option<&'a BStr> {
        // Check if the current database supports the backward lookup
        if self.bwd_offset > 0 && id < self.header.bwd_size {
            let offset = read_u32_le(self.buffer, self.bwd_offset + (id as usize) * 4);
            if offset > 0 {
                let index = offset as usize + 4; // Skip id field
                let value_size = read_u32_le(self.buffer, index) as usize - 1; // includes NUL
                let start = index + 4;
                return Some(self.buffer[start..start + value_size].as_bstr());
            }
        }
        None
    }

    /// An iterator visiting all id, string pairs in order.
    pub fn iter(&'a self) -> Iter<'a> {
        Iter { db: self, next: 0 }
    }
}

/// CQDB iterator
pub struct Iter<'a> {
    db: &'a CQDB<'a>,
    next: u32,
}

impl<'a> Iterator for Iter<'a> {
    type Item = io::Result<(u32, &'a BStr)>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next;
        if let Some(s) = self.db.to_str(id) {
            self.next += 1;
            return Some(Ok((id, s)));
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.db.bwd_offset > 0 {
            self.db.header.bwd_size.saturating_sub(self.next) as usize
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

impl<'a> IntoIterator for &'a CQDB<'a> {
    type Item = io::Result<(u32, &'a BStr)>;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Write + Seek> CQDBWriter<T> {
    /// Create a new CQDB writer
    pub fn new(writer: T) -> io::Result<Self> {
        Self::with_flag(writer, Flag::NONE)
    }

    /// Create a new CQDB writer with flag
    pub fn with_flag(mut writer: T, flag: Flag) -> io::Result<Self> {
        let begin = writer.stream_position()? as u32;
        let current = (mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES) as u32;
        // Move the file pointer to the offset to the first key/data pair
        writer.seek(SeekFrom::Start((begin + current) as u64))?;
        Ok(Self {
            writer,
            flag,
            begin,
            current,
            tables: std::array::from_fn(|_| Table::default()),
            bwd: Vec::new(),
            bwd_num: 0,
            bwd_size: 0,
        })
    }

    /// Put a string/identifier association to the database
    pub fn put<K: AsRef<[u8]>>(&mut self, key: K, id: u32) -> io::Result<()> {
        let key = key.as_ref();
        let key_size = key.len() as u32 + 1; // includes NUL byte
        let hash = crate::hash::jhash(key, key_size, 0);
        let table = &mut self.tables[hash as usize % 256];
        // Batch record write: [id(4) | key_size(4) | key | NUL]
        let record_len = 8 + key.len() + 1;
        if record_len <= 264 {
            let mut buf = [0u8; 264]; // 8 header + max 255 key + NUL
            buf[0..4].copy_from_slice(&pack_u32(id));
            buf[4..8].copy_from_slice(&pack_u32(key_size));
            buf[8..8 + key.len()].copy_from_slice(key);
            // buf[8 + key.len()] is already 0 (NUL)
            self.writer.write_all(&buf[..record_len])?;
        } else {
            // Fallback for very large keys
            self.writer.write_all(&pack_u32(id))?;
            self.writer.write_all(&pack_u32(key_size))?;
            self.writer.write_all(key)?;
            self.writer.write_all(b"\0")?;
        }
        // Expand the bucket if necessary
        if table.size <= table.num as usize {
            table.size = (table.size + 1) * 2;
            table.bucket.resize(table.size, Bucket::default());
        }
        // Set the hash value and current offset position
        table.bucket[table.num as usize].hash = hash;
        table.bucket[table.num as usize].offset = self.current;
        table.num += 1;
        // Store the backlink if specified
        if !self.flag.contains(Flag::ONEWAY) {
            // Expand the backlink arrray if necessary
            if self.bwd_size <= id {
                let mut size = self.bwd_size;
                while size <= id {
                    size = (size + 1) * 2;
                }
                self.bwd.resize(size as usize, 0);
                self.bwd_size = size;
            }
            if self.bwd_num <= id {
                self.bwd_num = id + 1;
            }
            self.bwd[id as usize] = self.current;
        }
        // Increment the current position
        self.current += 4 + 4 + key_size;
        Ok(())
    }

    /// Close the writer, flush the file stream
    fn close(&mut self) -> io::Result<()> {
        let mut header = Header {
            chunk_id: *CHUNK_ID,
            flag: self.flag.bits(),
            byteorder: BYTEORDER_CHECK,
            bwd_offset: 0,
            bwd_size: self.bwd_num,
            size: 0,
        };
        // Store the hash tables. At this moment, the file pointer refers to
        // the offset succeeding the last key/data pair.
        // Reuse dst Vec across tables to avoid per-table heap allocation.
        let mut dst: Vec<Bucket> = Vec::new();
        #[cfg(not(target_endian = "little"))]
        let mut write_buf: Vec<u8> = Vec::new();
        for i in 0..NUM_TABLES {
            let table = &self.tables[i];
            // Do not write empty hash tables
            if table.bucket.is_empty() {
                continue;
            }
            // Actual bucket will have the double size; half elements
            // in the bucket are kept empty.
            let n = table.num * 2;
            let n_usize = n as usize;
            // Reuse dst: only grows, never deallocates between tables
            dst.clear();
            dst.resize(n_usize, Bucket::default());
            // Put hash elements to the bucket with the open-address method
            for j in 0..table.num as usize {
                let src = &table.bucket[j];
                let mut k = (src.hash >> 8) % n;
                // Find a vacant element
                while dst[k as usize].offset != 0 {
                    k = (k + 1) % n;
                }
                // Store the hash element
                dst[k as usize].hash = src.hash;
                dst[k as usize].offset = src.offset;
            }
            // Write the entire bucket array for this table in one call.
            // On LE platforms, Bucket repr(C) {u32, u32} matches the on-disk format.
            #[cfg(target_endian = "little")]
            {
                let bytes =
                    unsafe { std::slice::from_raw_parts(dst.as_ptr() as *const u8, n_usize * 8) };
                self.writer.write_all(bytes)?;
            }
            #[cfg(not(target_endian = "little"))]
            {
                write_buf.clear();
                write_buf.reserve(n_usize * 8);
                for bucket in &dst[..n_usize] {
                    write_buf.extend_from_slice(&pack_u32(bucket.hash));
                    write_buf.extend_from_slice(&pack_u32(bucket.offset));
                }
                self.writer.write_all(&write_buf)?;
            }
        }
        // Write the backlink array if specified
        if !self.flag.contains(Flag::ONEWAY) && self.bwd_size > 0 {
            // Store the offset to the head of this array
            let current_offset = self.writer.stream_position()? as u32;
            header.bwd_offset = current_offset - self.begin;
            // Write all backward links in one call.
            #[cfg(target_endian = "little")]
            {
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        self.bwd.as_ptr() as *const u8,
                        self.bwd_num as usize * 4,
                    )
                };
                self.writer.write_all(bytes)?;
            }
            #[cfg(not(target_endian = "little"))]
            {
                write_buf.clear();
                write_buf.reserve(self.bwd_num as usize * 4);
                for i in 0..self.bwd_num as usize {
                    write_buf.extend_from_slice(&pack_u32(self.bwd[i]));
                }
                self.writer.write_all(&write_buf)?;
            }
        }
        // Store the current position
        let offset = self.writer.stream_position()? as u32;
        header.size = offset - self.begin;
        // Rewind the current position to the beginning
        self.writer.seek(SeekFrom::Start(self.begin as u64))?;
        // Write header + table references in a single batch (2072 bytes on stack)
        let mut hdr_buf = [0u8; 24 + NUM_TABLES * 8];
        hdr_buf[0..4].copy_from_slice(&header.chunk_id);
        hdr_buf[4..8].copy_from_slice(&pack_u32(header.size));
        hdr_buf[8..12].copy_from_slice(&pack_u32(header.flag));
        hdr_buf[12..16].copy_from_slice(&pack_u32(header.byteorder));
        hdr_buf[16..20].copy_from_slice(&pack_u32(header.bwd_size));
        hdr_buf[20..24].copy_from_slice(&pack_u32(header.bwd_offset));
        // Write references to hash tables. At this moment, self.current points
        // to the offset succeeding the last key/data pair.
        for i in 0..NUM_TABLES {
            let table_num = self.tables[i].num;
            // Offset to the hash table (or zero for non-existent tables)
            let table_offset = if table_num > 0 { self.current } else { 0 };
            let off = 24 + i * 8;
            hdr_buf[off..off + 4].copy_from_slice(&pack_u32(table_offset));
            // Bucket size is double the number of elements
            hdr_buf[off + 4..off + 8].copy_from_slice(&pack_u32(table_num * 2));
            // Advance the offset counter
            self.current += table_num * 2 * std::mem::size_of::<Bucket>() as u32;
        }
        self.writer.write_all(&hdr_buf)?;
        // Seek to the last position
        self.writer.seek(SeekFrom::Start(offset as u64))?;
        Ok(())
    }
}

impl<T: Write + Seek> Drop for CQDBWriter<T> {
    fn drop(&mut self) {
        if let Ok(()) = self.close() {}
    }
}
