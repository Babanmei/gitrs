use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

#[derive(Clone, Debug)]
pub struct Blob {
    count: i32,
    pub name: RefCell<String>,
    //pub hasher: RefCell<String>,
    pub hasher: RefCell<Vec<u8>>,
    pub content: Vec<u8>,
}

impl Blob {
    pub fn new(name: String) -> Blob {
        Blob {
            count: 0,
            name: RefCell::new(name),
            hasher: RefCell::new(vec![]),
            content: vec![],
        }
    }
}

/// parser file to blob struct
impl Blob {
    pub fn from(name: &str, hasher: Vec<u8>) -> Result<Blob> {
        let path = PathBuf::new().join(".git").join("objects");
        let path = path.join(String::from_utf8((&hasher[0..2]).to_vec())?);
        let path = path.join(String::from_utf8((&hasher[2..]).to_vec())?);
        //let path = path.join(&hasher[0..2]).join(&hasher[2..]);
        let mut file = File::open(path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        let bytes = decoder(&bytes)?;
        let (_, blob) = parse_blob(&bytes).unwrap();
        *blob.name.borrow_mut() = name.to_string();
        *blob.hasher.borrow_mut() = hasher;//hasher.to_string();
        Ok(blob)
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "blob {:?} {:?}", self.name.borrow().as_str(), self.hasher.borrow())
    }
}


use nom::{do_parse, named, tag, take, take_while};
use nom::character::is_digit;
use std::cell::RefCell;
use core::fmt;
use nom::lib::std::fmt::Formatter;
use crate::decoder;

named!(parse_blob<Blob>,
   do_parse!(
    _blob: tag!("blob ") >>
    count: take_while!(is_digit)  >>
    _c: take!(1) >>
    c: content >>
    (from_objects_file(count, c))
   )
);

pub fn content(c: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    Ok((&b""[..], c))
}

fn from_objects_file(count: &[u8], content: &[u8]) -> Blob {
    let content = content.to_vec();

    let c = String::from_utf8(count.to_vec()).unwrap();
    let count = c.parse::<i32>().unwrap();

    let hasher = RefCell::new(vec![]);
    let name = RefCell::new(String::new());
    Blob { count, name, hasher, content }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use crate::decoder;
    use std::io::Read;
    use super::parse_blob;

    #[test]
    fn test_parse_blob() {
        let f = "test_data/.git/objects/35/3e81709e49f3e29d2354d77d98c84534f7fe03";
        let mut file = File::open(f).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();
        let bytes = decoder(&bytes).unwrap();
        let (_, blob) = parse_blob(bytes.as_slice()).unwrap();
        let content = String::from_utf8(blob.content).unwrap();
        assert_eq!(content, "d1df\n");
    }
}
