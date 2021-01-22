use std::{fs, io, mem, path::Path};

mod c;

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
#[derive(Debug)]
pub struct Db {
    /// Chunk header
    header: Header,
    /// Hash tables (string -> id)
    tables: Vec<Table>,
    /// Array for backward lookup (id -> string)
    bwd: Vec<u32>,
    /// Number of key/data pairs
    num: u32,
}

/// Writer for a constant quark database
#[derive(Debug)]
pub struct DbWriter {
    /// Operation flag
    flag: u32,
    /// File
    file: fs::File,
    /// Offset address to the head of this database
    begin: u32,
    /// Offset address to a new key/data pair
    cur: u32,
    /// Hash tables (string -> id)
    ht: [Table; NUM_TABLES],
    /// Backlink array
    bwd: Vec<u32>,
    bwd_num: u32,
    /// Number of elements in the backlink array
    bwd_size: u32,
}

/// CQDB chunk header
#[derive(Debug)]
#[repr(C)]
pub struct Header {
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
#[derive(Debug)]
pub struct Table {
    /// Number of elements in the table
    num: u32,
    /// Maxinum number of elements
    size: u32,
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
#[derive(Debug)]
pub struct Bucket {
    /// Hash value of the record
    hash: u32,
    /// Offset address to the actual record
    offset: u32,
}

impl Db {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let buf = fs::read(path)?;
        Self::from_reader(&buf)
    }

    pub fn from_reader(buf: &[u8]) -> io::Result<Self> {
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
                    size: 0,
                }
            } else {
                // An empty hash table
                Table {
                    bucket: Vec::new(),
                    num: 0,
                    size: 0,
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
        Ok(Db {
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

    pub fn num(&self) -> u32 {
        self.num
    }
}
