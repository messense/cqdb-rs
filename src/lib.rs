use std::{fs, mem};

mod c;

const CHUNKID: &[u8; 4]= b"CQDB";
const BYTEORDER_CHECK: usize = 0x62445371;
const NUM_TABLES: usize = 256;

/// Constant quark database (CQDB)
#[derive(Debug)]
pub struct Db {
    /// Chunk header
    header: Header,
    /// Hash tables (string -> id)
    ht: [Table; NUM_TABLES],
    /// Array for backward lookup (id -> string)
    bwd: Vec<u32>,
    /// Number of key/data pairs
    num: i32,
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
    chunkid: [i8; 4],
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

/// Reference to a hash table
#[derive(Debug)]
#[repr(C)]
pub struct TableRef {
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
    pub fn from_buffer(buf: &[u8]) -> Self {
        if buf.len() < mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES {
            // The minimum size of a valid CQDB
        }
        // Check the file chunkid
        if &buf[0..4] != CHUNKID {

        }

        todo!()
    }

    pub fn num(&self) -> i32 {
        self.num
    }
}
