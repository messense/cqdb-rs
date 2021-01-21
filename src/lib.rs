use std::{
    fs,
    io::{self, Cursor, Read, Seek, SeekFrom},
    mem,
    path::Path,
};

use byteorder::{ReadBytesExt, LE};

mod c;

const CHUNKID: &[u8; 4] = b"CQDB";
const BYTEORDER_CHECK: u32 = 0x62445371;
const NUM_TABLES: usize = 256;

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
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let buf = fs::read(path)?;
        Self::from_reader(&buf)
    }

    pub fn from_reader(buf: &[u8]) -> io::Result<Self> {
        if buf.len() < mem::size_of::<Header>() + mem::size_of::<TableRef>() * NUM_TABLES {
            // The minimum size of a valid CQDB
            return Err(io::Error::new(io::ErrorKind::Other, "invalid file format"));
        }
        let mut cursor = Cursor::new(buf);
        let mut magic = [0; 4];
        cursor.read_exact(&mut magic)?;
        // Check the file chunkid
        if &magic != CHUNKID {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid file format, magic mismatch",
            ));
        }
        let chunk_size = cursor.read_u32::<LE>()?;
        let flag = cursor.read_u32::<LE>()?;
        let byte_order = cursor.read_u32::<LE>()?;
        // Check the consistency of byte order
        if byte_order != BYTEORDER_CHECK {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid file format, byte order mismatch",
            ));
        }
        let bwd_size = cursor.read_u32::<LE>()?;
        let bwd_offset = cursor.read_u32::<LE>()?;
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
            let table_ref = Self::read_table_ref(&mut cursor)?;
            let table = if table_ref.offset > 0 {
                let bucket =
                    Self::read_bucket(buf, table_ref.offset as u64, table_ref.num as usize)?;
                Table {
                    bucket,
                    num: table_ref.num,
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
            num_db += table_ref.num / 2;
        }
        let bwd = if bwd_offset > 0 {
            Self::read_backward_links(buf, bwd_offset as u64, num_db as usize)?
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

    #[inline]
    fn read_table_ref(cursor: &mut Cursor<&[u8]>) -> io::Result<TableRef> {
        let offset = cursor.read_u32::<LE>()?;
        let num = cursor.read_u32::<LE>()?;
        Ok(TableRef { offset, num })
    }

    fn read_bucket(buf: &[u8], offset: u64, num: usize) -> io::Result<Vec<Bucket>> {
        let mut cursor = Cursor::new(buf);
        cursor.seek(SeekFrom::Start(offset))?;
        let mut buckets = Vec::with_capacity(num);
        for _ in 0..num {
            let hash = cursor.read_u32::<LE>()?;
            let offset = cursor.read_u32::<LE>()?;
            buckets.push(Bucket { hash, offset });
        }
        Ok(buckets)
    }

    fn read_backward_links(buf: &[u8], offset: u64, num: usize) -> io::Result<Vec<u32>> {
        let mut cursor = Cursor::new(buf);
        cursor.seek(SeekFrom::Start(offset))?;
        let mut bwd = Vec::with_capacity(num);
        for _ in 0..num {
            bwd.push(cursor.read_u32::<LE>()?);
        }
        Ok(bwd)
    }

    pub fn num(&self) -> u32 {
        self.num
    }
}

#[cfg(test)]
mod tests {
    use super::Db;

    #[test]
    fn test_cqdb_reader() {
        let db = Db::open("tests/fixtures/test.cqdb").unwrap();
        assert_eq!(100, db.num());
    }
}
