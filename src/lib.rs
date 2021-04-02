#![feature(seek_convenience)]

#[macro_use]
extern crate anyhow;
extern crate binwrite;
extern crate sha1;
extern crate nom;

pub mod rep;
pub mod index;
pub mod hasher;
pub mod objects;


use anyhow::Result;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::{Write, Read};
use std::path::PathBuf;
use flate2::read::ZlibDecoder;
use nom::AsBytes;


pub fn compression(body: &Vec<u8>) -> Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(body.as_slice())?;
    let bytes = e.finish()?;
    Ok(bytes)
}

pub fn decoder(body: &Vec<u8>) -> Result<Vec<u8>> {
    let mut z = ZlibDecoder::new(body.as_bytes());
    let mut bytes = vec![];
    let _ = z.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn write_object_to_file(sha: &str, body: &Vec<u8>) -> Result<()> {
    let file = PathBuf::new().join(".git").join("objects");
    let dir = file.join(&sha[0..2]);
    let object = dir.join(&sha[2..]);

    let bytes = compression(body)?;
    std::fs::create_dir(dir)?;
    std::fs::write(object, bytes)?;
    Ok(())
}

pub fn walk_dir(dir: &PathBuf, prefix: &PathBuf, names: &mut Vec<PathBuf>) -> Result<()> {
    let dirs = std::fs::read_dir(dir)?;
    for dir in dirs {
        let dir = dir?;
        let name = dir.file_name().to_str().unwrap().to_string();
        if dir.metadata()?.is_dir() {
            walk_dir(&dir.path(), &prefix.join(name), names)?;
        } else {
            names.push(prefix.join(name));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::decoder;

    #[test]
    fn test_decoder_encoder() {
        let c = "test data";
        let v = c.as_bytes().to_vec();
        let _encoder = compression(&v).unwrap();
        assert_ne!(v, _encoder);
        let _decoder = decoder(&_encoder).unwrap();
        assert_eq!(v, _decoder);
    }
}