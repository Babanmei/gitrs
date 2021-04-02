use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, Cursor, Read, Seek};
use std::path::PathBuf;

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};

use crate::index::{Entry, Extension, Index};

trait ReadIndex {
    fn read_name(&mut self, name_len: i16) -> Result<String>;
    fn read_n_length_bytes(&mut self, n: i32) -> Result<Vec<u8>>;
    fn read_n_length_string(&mut self, n: i32) -> Result<String>;
    fn read_time(&mut self) -> Result<(i32, i32)>;
}

//https://github.com/sbp/gin/blob/master/gin
impl ReadIndex for Cursor<Vec<u8>> {
    fn read_name(&mut self, name_len: i16) -> Result<String> {
        if name_len < 0xFFF {
            let mut v = with_fill_capacity(name_len as usize);
            self.read_exact(&mut v).unwrap();
            Ok(String::from_utf8(v.to_vec())?)
        } else {
            let mut v = vec![];
            self.read_until(b'\x00', &mut v).unwrap();
            Ok(String::from_utf8(v.to_vec())?)
        }
    }

    fn read_n_length_bytes(&mut self, n: i32) -> Result<Vec<u8>> {
        let mut v = with_fill_capacity(n as usize);
        let _ = self.read(&mut v)?;
        Ok(v)
    }

    fn read_n_length_string(&mut self, n: i32) -> Result<String> {
        let mut v = with_fill_capacity(n as usize);
        let _ = self.read(&mut v)?;
        Ok(String::from_utf8(v)?)
    }

    fn read_time(&mut self) -> Result<(i32, i32)> {
        let sec = self.read_i32::<BigEndian>()?;
        let nano = self.read_i32::<BigEndian>()?;
        //Ok(sec + nano / 1000000000)
        Ok((sec, nano))
    }
}


pub(crate) fn parse_index(index: &PathBuf) -> Result<Index> {
    let mut bytes = vec![];
    let mut f = File::open(index)?;
    let _ = f.read_to_end(&mut bytes);

    let mut rdr = Cursor::new(bytes);

    let signature = rdr.read_n_length_string(4)?;
    let version = rdr.read_i32::<BigEndian>()?;
    let entry_count = rdr.read_i32::<BigEndian>()?;

    let mut entris = vec![];
    for _i in 0..entry_count {
        let ctime = rdr.read_time()?;
        let mtime = rdr.read_time()?;

        let dev = rdr.read_i32::<BigEndian>()?;
        //println!("dev: {}", dev);
        let inode = rdr.read_i32::<BigEndian>()?;
        let mode = rdr.read_i32::<BigEndian>()?;
        let uid = rdr.read_i32::<BigEndian>()?;
        let gid = rdr.read_i32::<BigEndian>()?;
        let file_size = rdr.read_i32::<BigEndian>()?;

        let hahs_vec = rdr.read_n_length_bytes(20)?;
        let mut hasher = String::new();
        for b in hahs_vec.iter() {
            let s = if *b < 16_u8 {
                format!("0{:x}", b)
            } else {
                format!("{:x}", b)
            };
            hasher.push_str(&s);//&format!("{:x}", b));
        }
        let mut v = with_fill_capacity(2);
        let _ = rdr.read(&mut v);
        let flag = v.to_vec().as_slice().read_i16::<BigEndian>()?;

        let extended = flag & (0b01000000 << 8);
        let mut entry_len = 62;
        if extended == 1 && version == 3 {
            let _extra_flags = rdr.read_n_length_bytes(2)?;
            entry_len += 2;
        }
        let name_len = flag & 0xFFF;
        let name = rdr.read_name(name_len)?;
        if name_len < 0xFFF { entry_len += name_len } else { entry_len += v.len() as i16 }

        let pad_len = 8 - (entry_len % 8);
        let pad = rdr.read_n_length_bytes(pad_len as i32)?;
        let e = Entry {
            dev,
            inode,
            mode,
            ctime,
            mtime,
            uid,
            gid,
            file_size,
            hasher,
            hasher_vec: hahs_vec,
            flag,
            name,
            pad,
        };
        entris.push(e);
    }

    let (index_len, mut _idx, mut extensions) = (f.stream_len()?, 0_i8, vec![]);
    loop {
        if rdr.position() >= index_len - 20 {
            break;
        }
        _idx += 1;
        let signature = rdr.read_n_length_string(4)?;
        let size = rdr.read_i32::<BigEndian>()?;
        let data = rdr.read_n_length_bytes(size)?;
        extensions.push(Extension {
            extension: "".to_string(),
            signature,
            size,
            data,
        });
    }
    let checksum = rdr.read_n_length_bytes(20)?;

    Ok(Index {
        signature,
        version,
        entry_count: RefCell::new(entry_count),
        entries: RefCell::new(entris),
        extensions,
        checksum,
    })
}

fn with_fill_capacity(size: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(size);
    for _b in 0..size {
        v.push(0);
    }
    v
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser_index() {
        let idx = parse_index(&PathBuf::from("test_data/.git/index")).unwrap();
        //d1/d1d/d1df, d1/df1, f1, f2
        assert_eq!(idx.entry_count.into_inner(), 4);
    }
}