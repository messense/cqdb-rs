use std::{
    fs,
    io::{self, Seek, SeekFrom, Write},
    mem,
};

use arr_macro::arr;
use bitflags::bitflags;

mod hash;

const CHUNKID: &[u8; 4] = b"CQDB";
const BYTEORDER_CHECK: u32 = 0x62445371;
const NUM_TABLES: usize = 256;

bitflags! {
    pub struct Flag: u32 {
        /// No flag, default
        const NONE = 0;
        /// A reverse lookup array is omitted
        const ONEWAY = 0x00000001;
    }
}

fn unpack_u32(buf: &[u8]) -> io::Result<u32> {
    if buf.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "not enough data for unpacking u32",
        ));
    }
    Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
}

#[inline(always)]
fn pack_u32(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Constant quark database (CQDB)
#[derive(Debug, Clone)]
pub struct CQDB<'a> {
    /// Database file buffer
    buffer: &'a [u8],
    /// Chunk header
    header: Header,
    /// Hash tables (string -> id)
    tables: [Table; NUM_TABLES],
    /// Array for backward lookup (id -> string)
    bwd: Vec<u32>,
    /// Number of key/data pairs
    num: u32,
}

/// CQDB chunk header
#[derive(Debug, Clone)]
#[repr(C)]
struct Header {
    /// Chunk identifier, "CQDB"
    chunkid: [u8; 4],
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

/// A hash table
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
#[derive(Debug)]
pub struct CQDBWriter<'a> {
    file: &'a mut fs::File,
    /// Operation flag
    flag: Flag,
    /// Offset address to the head of this database
    begin: u64,
    /// Offset address to a new key/data pair
    current: u64,
    /// Hash tables (string -> id)
    tables: [Table; NUM_TABLES],
    /// Backlink array
    bwd: Vec<u32>,
    bwd_num: u32,
    /// Number of elements in the backlink array
    bwd_size: u32,
}

impl<'a> CQDB<'a> {
    pub fn new(buf: &'a [u8]) -> io::Result<Self> {
        if buf.len() < mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES {
            // The minimum size of a valid CQDB
            return Err(io::Error::new(io::ErrorKind::Other, "invalid file format"));
        }
        let magic = &buf[0..4];
        // Check the file chunkid
        if magic != CHUNKID {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid file format, magic mismatch",
            ));
        }
        let mut index = 4; // skip magic
        let chunk_size = unpack_u32(&buf[index..])?;
        index += 4;
        let flag = unpack_u32(&buf[index..])?;
        index += 4;
        let byte_order = unpack_u32(&buf[index..])?;
        index += 4;
        // Check the consistency of byte order
        if byte_order != BYTEORDER_CHECK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid file format, byte order mismatch",
            ));
        }
        let bwd_size = unpack_u32(&buf[index..])?;
        index += 4;
        let bwd_offset = unpack_u32(&buf[index..])?;
        index += 4;
        let header = Header {
            chunkid: CHUNKID.clone(),
            size: chunk_size,
            flag,
            byteorder: byte_order,
            bwd_size,
            bwd_offset,
        };
        let mut num_db = 0;
        let mut tables = arr![Table::default(); 256];
        for i in 0..NUM_TABLES {
            let table_offset = unpack_u32(&buf[index..])?;
            index += 4;
            let table_num = unpack_u32(&buf[index..])?;
            index += 4;
            if table_offset > 0 {
                let bucket = Self::read_bucket(buf, table_offset as usize, table_num as usize)?;
                tables[i].bucket = bucket;
                tables[i].num = table_num;
            }
            // The number of records is the half of the table size
            num_db += table_num / 2;
        }
        let bwd = if bwd_offset > 0 {
            Self::read_backward_links(buf, bwd_offset as usize, num_db as usize)?
        } else {
            Vec::new()
        };
        Ok(Self {
            buffer: buf,
            header,
            tables,
            bwd,
            num: num_db,
        })
    }

    fn read_bucket(buf: &[u8], offset: usize, num: usize) -> io::Result<Vec<Bucket>> {
        let mut buckets = Vec::with_capacity(num);
        let mut index = offset;
        for _ in 0..num {
            let hash = unpack_u32(&buf[index..])?;
            index += 4;
            let offset = unpack_u32(&buf[index..])?;
            index += 4;
            buckets.push(Bucket { hash, offset });
        }
        Ok(buckets)
    }

    fn read_backward_links(buf: &[u8], offset: usize, num: usize) -> io::Result<Vec<u32>> {
        let mut bwd = Vec::with_capacity(num);
        let mut index = offset;
        for _ in 0..num {
            bwd.push(unpack_u32(&buf[index..])?);
            index += 4;
        }
        Ok(bwd)
    }

    /// Get the number of associations in the database
    pub fn num(&self) -> u32 {
        self.num
    }

    /// Retrieve the identifier associated with a string
    pub fn to_id(&self, s: &str) -> Option<u32> {
        self.to_id_impl(s).unwrap_or_default()
    }

    fn to_id_impl(&self, s: &str) -> io::Result<Option<u32>> {
        let hash = crate::hash::jhash(s.as_bytes(), s.len() + 1, 0);
        let table_index = hash % NUM_TABLES as u32;
        let table = &self.tables[table_index as usize];
        if table.num > 0 && !table.bucket.is_empty() {
            let n = table.num;
            let mut k = (hash >> 8) % n;
            loop {
                let bucket = &table.bucket[k as usize];
                if bucket.offset > 0 {
                    if bucket.hash == hash {
                        let mut index = bucket.offset as usize;
                        let value = unpack_u32(&self.buffer[index..])?;
                        index += 4;
                        let ksize = unpack_u32(&self.buffer[index..])? - 1; // ksize includes NUL byte
                        index += 4;
                        let actual_str = &self.buffer[index..index + ksize as usize];
                        if s.as_bytes() == actual_str {
                            return Ok(Some(value));
                        }
                    }
                } else {
                    break;
                }
                k = (k + 1) % n;
            }
        }
        Ok(None)
    }

    /// Retrieve the string associated with an identifier
    pub fn to_str(&self, id: u32) -> Option<&str> {
        self.to_str_impl(id).unwrap_or_default()
    }

    fn to_str_impl(&self, id: u32) -> io::Result<Option<&str>> {
        // Check if the current database supports the backward lookup
        if !self.bwd.is_empty() && (id as u32) < self.header.bwd_size {
            let offset = self.bwd[id as usize];
            if offset > 0 {
                let mut index = offset as usize + 4; // Skip key data
                let value_size = unpack_u32(&self.buffer[index..])? as usize - 1; // value_size includes NUL byte
                index += 4;
                if let Ok(s) = std::str::from_utf8(&self.buffer[index..index + value_size]) {
                    return Ok(Some(s));
                }
            }
        }
        Ok(None)
    }
}

impl<'a> CQDBWriter<'a> {
    /// Create a new CQDB writer
    pub fn new(file: &'a mut fs::File) -> io::Result<Self> {
        Self::with_flag(file, Flag::NONE)
    }

    /// Create a new CQDB writer with flag
    pub fn with_flag(file: &'a mut fs::File, flag: Flag) -> io::Result<Self> {
        let begin = file.seek(SeekFrom::Current(0))? as u64;
        let current = (mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES) as u64;
        // Move the file pointer to the offset to the first key/data pair
        file.seek(SeekFrom::Start(begin + current))?;
        Ok(Self {
            file,
            flag,
            begin,
            current,
            tables: arr![Table::default(); 256],
            bwd: Vec::new(),
            bwd_num: 0,
            bwd_size: 0,
        })
    }

    /// Put a string/identifier association to the database
    pub fn put(&mut self, key: &str, id: u32) -> io::Result<()> {
        let key_size = key.len() + 1; // includes NUL byte
        let hash = crate::hash::jhash(key.as_bytes(), key_size, 0);
        let table = &mut self.tables[hash as usize % 256];
        // Write out the current data
        self.file.write_all(&pack_u32(id))?;
        self.file.write_all(&pack_u32(key_size as u32))?;
        self.file.write_all(key.as_bytes())?;
        self.file.write_all(b"\0")?;
        // Expand the bucket if necessary
        if table.size <= table.num as usize {
            table.size = (table.size + 1) * 2;
            table.bucket.resize(table.size, Bucket::default());
        }
        // Set the hash value and current offset position
        table.bucket[table.num as usize].hash = hash;
        table.bucket[table.num as usize].offset = self.current as u32;
        table.num += 1;
        // Store the backlin if specified
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
            self.bwd[id as usize] = self.current as u32;
        }
        // Increment the current position
        self.current += 4 + 4 + key_size as u64;
        Ok(())
    }

    /// Close the writer, flush the file stream
    pub fn close(&mut self) -> io::Result<()> {
        let mut header = Header {
            chunkid: CHUNKID.clone(),
            flag: self.flag.bits,
            byteorder: BYTEORDER_CHECK,
            bwd_offset: 0,
            bwd_size: self.bwd_num,
            size: 0,
        };
        // Store the hash tables. At this moment, the file pointer refers to
        // the offset succeeding the last key/data pair.
        for i in 0..NUM_TABLES {
            let table = &self.tables[i];
            // Do not write empty hash tables
            if table.bucket.is_empty() {
                continue;
            }
            // Actual bucket will have the double size; half elements
            // in the bucket are kept empty.
            let n = table.num * 2;
            let mut dst = vec![Bucket::default(); n as usize];
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
            // Write the bucket
            for k in 0..n as usize {
                self.file.write_all(&pack_u32(dst[k].hash))?;
                self.file.write_all(&pack_u32(dst[k].offset))?;
            }
        }
        // Write the backlink array if specified
        if !self.flag.contains(Flag::ONEWAY) && self.bwd_size > 0 {
            // Store the offset to the head of this array
            let current_offset = self.file.seek(SeekFrom::Current(0))?;
            header.bwd_offset = (current_offset - self.begin) as u32;
            // Stroe the contents of the backlink array
            for i in 0..self.bwd_num as usize {
                self.file.write_all(&pack_u32(self.bwd[i]))?;
            }
        }
        // Store the current position
        let offset = self.file.seek(SeekFrom::Current(0))?;
        header.size = (offset - self.begin) as u32;
        // Rewind the current position to the beginning
        self.file.seek(SeekFrom::Start(self.begin))?;
        // Write the file header
        self.file.write_all(&header.chunkid)?;
        self.file.write_all(&pack_u32(header.size))?;
        self.file.write_all(&pack_u32(header.flag))?;
        self.file.write_all(&pack_u32(header.byteorder))?;
        self.file.write_all(&pack_u32(header.bwd_size))?;
        self.file.write_all(&pack_u32(header.bwd_offset))?;
        // Write references to hash tables. At this moment, self.current points
        // to the offset succeeding the last key/data pair.
        for i in 0..NUM_TABLES {
            let table_num = self.tables[i].num;
            // Offset to the hash table (or zero for non-existent tables)
            let table_offset = if table_num > 0 {
                self.current as u32
            } else {
                0
            };
            self.file.write_all(&pack_u32(table_offset))?;
            // Bucket size is double the number of elements
            self.file.write_all(&pack_u32(table_num * 2))?;
            // Advance the offset counter
            self.current += table_num as u64 * 2 * std::mem::size_of::<Bucket>() as u64;
        }
        // Seek to the last position
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
}

impl<'a> Drop for CQDBWriter<'a> {
    fn drop(&mut self) {
        match self.close() {
            Ok(()) => {}
            Err(_) => {}
        }
    }
}
