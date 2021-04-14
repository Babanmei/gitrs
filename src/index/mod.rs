use std::cell::RefCell;
use std::io::Write;
use std::os::macos::fs::MetadataExt;
use std::path::PathBuf;

use anyhow::Result;
use binwrite::{BinWrite, Endian, WriterOption};

use crate::hasher::generic_blob_hash;

pub mod parse_index;

#[derive(Debug, Clone, Default)]
#[derive(BinWrite)]
#[binwrite(big)]
pub struct Extension {
    extension: String,
    signature: String,
    size: i32,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Index {
    signature: String,
    version: i32,
    pub entry_count: RefCell<i32>,
    pub entries: RefCell<Vec<Entry>>,
    extensions: Vec<Extension>,
    checksum: Vec<u8>,
}


impl Default for Index {
    fn default() -> Self {
        Index {
            signature: "DIRC".to_string(),
            version: 2,
            entry_count: RefCell::new(0),
            entries: RefCell::new(vec![]),
            extensions: vec![],
            checksum: vec![],
        }
    }
}

impl Index {
    /// from index file
    pub fn from_index_file(index: &PathBuf) -> Result<Index> {
        parse_index::parse_index(index)
    }

    pub fn add_entry(&self, ent: Entry) {
        if !self.is_exists_entrie(ent.hasher.clone()) {
            let (mut equal, mut idx) = (false, 0);
            for _ent in self.entries.borrow().iter() {
                if _ent.name == ent.name {
                    equal = true;
                    break;
                }
                idx += 1;
            }
            if equal {
                self.entries.borrow_mut().remove(idx);
                self.entries.borrow_mut().push(ent);
            } else {
                self.entries.borrow_mut().push(ent);
                *self.entry_count.borrow_mut() += 1
            }
        }
    }

    pub fn write_index_file(&self) -> Result<()> {
        let file = PathBuf::new().join(".git").join("index");
        let mut bytes = vec![];
        let _ = self.write(&mut bytes)?;
        let _ = std::fs::write(file, bytes)?;
        Ok(())
    }

    pub fn is_exists_entrie(&self, hasher: String) -> bool {
        for ent in self.entries.borrow().iter() {
            if ent.hasher == hasher {
                return true;
            }
        }
        false
    }

    pub fn update_entrie_mtime(&self, name: &str, mtime: (i32, i32)) -> Result<()> {
        for ent in self.entries.borrow_mut().iter_mut() {
            if ent.name.as_str() == name {
                ent.mtime = mtime;
                break;
            }
        }
        self.write_index_file()?;
        Ok(())
    }
}


impl BinWrite for Index {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let wo = &mut WriterOption::default();
        wo.endian = Endian::Big;
        self.write_options(writer, &wo)
    }

    fn write_options<W: Write>(&self, writer: &mut W, options: &WriterOption) -> std::io::Result<()> {
        BinWrite::write_options(&self.signature, writer, options)?;
        BinWrite::write_options(&self.version, writer, options)?;
        let x = *self.entry_count.borrow_mut();
        BinWrite::write_options(&x, writer, options)?;
        let entries = &*self.entries.borrow_mut();
        BinWrite::write_options(entries, writer, options)?;
        BinWrite::write_options(&self.extensions, writer, options)?;
        BinWrite::write_options(&self.checksum, writer, options)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct Entry {
    pub ctime: (i32, i32),
    pub mtime: (i32, i32),
    pub dev: i32,
    pub inode: i32,
    pub mode: i32,
    pub uid: i32,
    pub gid: i32,
    pub file_size: i32,
    pub hasher: String,
    pub hasher_vec: Vec<u8>,
    pub flag: i16,
    pub name: String,
    pub pad: Vec<u8>,
}

impl Entry {
    pub fn from(file: &PathBuf) -> Result<Entry> {
        let name = file.to_str().unwrap().to_string();
        let (sha, _) = generic_blob_hash(file)?;
        let size = file.metadata()?.len();
        let meta = std::fs::metadata(file)?;

        let flag = name.len();
        let extended = flag & (0b01000000 << 8);
        let (mut entry_len, version) = (62, 2);
        if extended == 1 && version == 3 {
            entry_len += 2;
        }
        let name_len = flag & 0xFFF;
        if name_len < 0xFFF { entry_len += name_len } else { entry_len += 2 }

        let pad_len = 8 - (entry_len % 8);
        let mut pad = vec![];
        for _ in 0..pad_len {
            pad.push(0);
        }
        let ent = Entry {
            ctime: (meta.st_ctime() as i32, meta.st_ctime_nsec() as i32),
            mtime: (meta.st_mtime() as i32, meta.st_mtime_nsec() as i32),
            dev: meta.st_dev() as i32,
            inode: meta.st_ino() as i32,
            mode: meta.st_mode() as i32,
            uid: meta.st_uid() as i32,
            gid: meta.st_gid() as i32,
            file_size: size as i32,
            hasher: sha.digest().to_string(),
            hasher_vec: sha.digest().bytes().to_vec(),
            flag: name.len() as i16,
            name,
            pad,
        };
        Ok(ent)
    }
}

impl BinWrite for Entry {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let wo = &mut WriterOption::default();
        wo.endian = Endian::Big;
        self.write_options(writer, &wo)
    }

    fn write_options<W: Write>(&self, writer: &mut W, options: &WriterOption) -> std::io::Result<()> {
        BinWrite::write_options(&self.ctime.0, writer, options)?;
        BinWrite::write_options(&self.ctime.1, writer, options)?;
        BinWrite::write_options(&self.mtime.0, writer, options)?;
        BinWrite::write_options(&self.mtime.1, writer, options)?;
        BinWrite::write_options(&self.dev, writer, options)?;
        BinWrite::write_options(&self.inode, writer, options)?;
        BinWrite::write_options(&self.mode, writer, options)?;
        BinWrite::write_options(&self.uid, writer, options)?;
        BinWrite::write_options(&self.gid, writer, options)?;
        BinWrite::write_options(&self.file_size, writer, options)?;
        BinWrite::write_options(&self.hasher_vec, writer, options)?;
        BinWrite::write_options(&self.flag, writer, options)?;
        BinWrite::write_options(&self.name, writer, options)?;
        BinWrite::write_options(&self.pad, writer, options)?;
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use std::path::Path;
    use std::str::FromStr;

    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_entry() {
        let index = Index::default();
        let ent = Entry::from(&PathBuf::from("test_data/f1")).unwrap();
        index.add_entry(ent);
        let ent = Entry::from(&PathBuf::from("test_data/f2")).unwrap();
        index.add_entry(ent.clone());

        assert_eq!(index.entry_count.clone().into_inner(), 2);
        //同一文件重复提交, 后者覆盖前者, count不变
        index.add_entry(ent);
        assert_eq!(index.entry_count.into_inner(), 2);
    }
}
