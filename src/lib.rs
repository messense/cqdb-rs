use std::{
    fs,
    io::{self, Seek, SeekFrom, Write},
    mem,
};

mod hash;

const CHUNKID: &[u8; 4] = b"CQDB";
const BYTEORDER_CHECK: u32 = 0x62445371;
const NUM_TABLES: usize = 256;

fn read_u32(buf: &[u8]) -> io::Result<u32> {
    if buf.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "not enough data for reading u32",
        ));
    }
    Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
}

/// Constant quark database (CQDB)
#[derive(Debug, Clone)]
pub struct CQDB<'a> {
    /// Database file buffer
    buffer: &'a [u8],
    /// Chunk header
    header: Header,
    /// Hash tables (string -> id)
    tables: Vec<Table>,
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
#[derive(Debug, Clone, Copy)]
struct Bucket {
    /// Hash value of the record
    hash: u32,
    /// Offset address to the actual record
    offset: u32,
}

/// Writer for a constant quark database
#[derive(Debug)]
pub struct CQDBWriter {
    file: fs::File,
    /// Operation flag
    flag: u32,
    /// Offset address to the head of this database
    begin: u64,
    /// Offset address to a new key/data pair
    current: u64,
    /// Hash tables (string -> id)
    tables: Vec<Table>,
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
        let chunk_size = read_u32(&buf[index..index + 4])?;
        index += 4;
        let flag = read_u32(&buf[index..index + 4])?;
        index += 4;
        let byte_order = read_u32(&buf[index..index + 4])?;
        index += 4;
        // Check the consistency of byte order
        if byte_order != BYTEORDER_CHECK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid file format, byte order mismatch",
            ));
        }
        let bwd_size = read_u32(&buf[index..index + 4])?;
        index += 4;
        let bwd_offset = read_u32(&buf[index..index + 4])?;
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
        let mut tables = Vec::with_capacity(NUM_TABLES);
        for _ in 0..NUM_TABLES {
            let table_offset = read_u32(&buf[index..index + 4])?;
            index += 4;
            let table_num = read_u32(&buf[index..index + 4])?;
            index += 4;
            let table = if table_offset > 0 {
                let bucket = Self::read_bucket(buf, table_offset as usize, table_num as usize)?;
                Table {
                    bucket,
                    num: table_num,
                }
            } else {
                // An empty hash table
                Table {
                    bucket: Vec::new(),
                    num: 0,
                }
            };
            tables.push(table);
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
            let hash = read_u32(&buf[index..index + 4])?;
            index += 4;
            let offset = read_u32(&buf[index..index + 4])?;
            index += 4;
            buckets.push(Bucket { hash, offset });
        }
        Ok(buckets)
    }

    fn read_backward_links(buf: &[u8], offset: usize, num: usize) -> io::Result<Vec<u32>> {
        let mut bwd = Vec::with_capacity(num);
        let mut index = offset;
        for _ in 0..num {
            bwd.push(read_u32(&buf[index..index + 4])?);
            index += 4;
        }
        Ok(bwd)
    }

    /// Get the number of associations in the database
    pub fn num(&self) -> u32 {
        self.num
    }

    /// Retrieve the identifier associated with a string
    pub fn to_id(&self, s: &str) -> Option<i32> {
        self.to_id_impl(s).unwrap_or_default()
    }

    fn to_id_impl(&self, s: &str) -> io::Result<Option<i32>> {
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
                        let value = read_u32(&self.buffer[index..index + 4])?;
                        index += 4;
                        let ksize = read_u32(&self.buffer[index..index + 4])? - 1; // ksize includes NUL byte
                        index += 4;
                        let actual_str = &self.buffer[index..index + ksize as usize];
                        if s.as_bytes() == actual_str {
                            return Ok(Some(value as i32));
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
    pub fn to_str(&self, id: i32) -> Option<&str> {
        self.to_str_impl(id).unwrap_or_default()
    }

    fn to_str_impl(&self, id: i32) -> io::Result<Option<&str>> {
        // Check if the current database supports the backward lookup
        if !self.bwd.is_empty() && (id as u32) < self.header.bwd_size {
            let offset = self.bwd[id as usize];
            if offset > 0 {
                let mut index = offset as usize + 4; // Skip key data
                let value_size = read_u32(&self.buffer[index..index + 4])? as usize - 1; // value_size includes NUL byte
                index += 4;
                if let Ok(s) = std::str::from_utf8(&self.buffer[index..index + value_size]) {
                    return Ok(Some(s));
                }
            }
        }
        Ok(None)
    }
}

impl CQDBWriter {
    /// Create a new CQDB writer
    pub fn new(file: fs::File) -> io::Result<Self> {
        Self::with_flag(file, 0)
    }

    /// Create a new CQDB writer with flag
    pub fn with_flag(mut file: fs::File, flag: u32) -> io::Result<Self> {
        let begin = file.seek(SeekFrom::Current(0))? as u64;
        let current = (mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES) as u64;
        // Move the file pointer to the offset to the first key/data pair
        file.seek(SeekFrom::Start(begin + current))?;
        Ok(Self {
            file,
            flag,
            begin,
            current,
            tables: Vec::with_capacity(NUM_TABLES),
            bwd: Vec::new(),
            bwd_num: 0,
            bwd_size: 0,
        })
    }

    /// Put a string/identifier association to the database
    pub fn put(&mut self, key: &str, id: i32) -> io::Result<()> {
        todo!()
    }

    /// Close the writer, flush the file stream
    pub fn close(&mut self) -> io::Result<()> {
        todo!()
    }
}
