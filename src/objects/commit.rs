use crate::objects::tree::Tree;
use core::fmt;
use std::fmt::Formatter;
use sha1::Sha1;
use std::os::macos::fs::MetadataExt;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug)]
pub struct Commit {
    parent: Option<String>,
    pub tree: Tree,
    author: String,
    committer: String,
    msg: String,
}

impl Commit {
    pub fn new(tree: Tree, msg: String) -> Commit {
        let hasher = tree.sha1.to_string();
        let tree_path = PathBuf::new().join(".git").join("objects")
            .join(&hasher[0..2]).join(&hasher[2..]);
        let meta = std::fs::metadata(tree_path).unwrap();
        let author = format!("bernie <xiongyuanbiao01@renmaitech.com> {} +0800", meta.st_ctime());
        let committer = format!("bernie <xiongyuanbiao01@renmaitech.com> {} +0800", meta.st_ctime());
        Commit {
            parent: None,
            tree,
            author,
            committer,
            msg,
        }
    }

    pub fn generate_commit_body(&self) -> (Sha1, Vec<u8>) {
        let body = format!("tree {}\nauthor {}\ncommitter {}\n\n{}\n",
                           self.tree.sha1.as_str(),
                           self.author,
                           self.committer,
                           self.msg
        );
        let body = format!("commit {}\0{}", body.len(), body);
        let mut sha1 = Sha1::new();
        sha1.update(body.as_bytes());
        (sha1, body.into_bytes())
    }
}

/// parser commit file
impl Commit {
    pub fn from_hasher(hasher: &str) -> Result<Commit> {
        let bytes = read_object(hasher)?;
        let (_, commit) = parse_commit(bytes.as_slice()).unwrap();
        Ok(commit)
    }

    fn from_objects_file(a: &[u8], b: &[u8], parent: Option<String>, c: &[u8], d: &[u8], e: &[u8]) -> Commit {
        let cc = String::from_utf8(a.to_vec()).unwrap();
        let _count = cc.parse::<i32>().unwrap();

        let tree_hasher = String::from_utf8(b.to_vec()).unwrap();
        let mut tree = Tree::from_hasher(&tree_hasher).unwrap();
        tree.sha1 = tree_hasher;

        Commit {
            parent,
            tree,
            author: String::from_utf8(c.to_vec()).unwrap(),
            committer: String::from_utf8(d.to_vec()).unwrap(),
            msg: String::from_utf8(e.to_vec()).unwrap(),
        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "tree {}\nauthor {}\ncommitter {}\n\n{}",
               self.tree.sha1.as_str(),
               self.author,
               self.committer,
               self.msg
        )
    }
}


use nom::{do_parse, is_not, named, tag, take, take_until};
use nom::bytes::complete::{tag, take, take_until};
use nom::combinator::opt;
use crate::read_object;

named!(parse_commit<Commit>,
    do_parse!(
        _commit: tag!("commit ") >>
        count: is_not!("\0") >>
        _a: take!(1) >>
        _child: take_until!(" ") >>
        _b: take!(1) >>
        sha: take_until!("\n") >>
        _c: take!(1) >>
        parent: parent_str >>
        author: take_until!("\n") >>
        _d: take!(1) >>
        committer: take_until!("\n\n") >>
        _e: take!(2) >>
        m: take_until!("\n") >>
        (Commit::from_objects_file(count, sha, parent, author, committer, m))
    )
);

fn parent_str(bytes: &[u8]) -> nom::IResult<&[u8], Option<String>> {
    let (input, p) = opt(tag("parent"))(bytes)?;

    if p.is_some() {
        let (input, _s) = take(1usize)(input)?;
        let (input, parent) = take_until("\n")(input)?;
        let (input, _) = take(1usize)(input)?;

        let s = String::from_utf8(parent.to_vec()).unwrap();
        Ok((input, Some(s)))
    } else {
        Ok((bytes, None))
    }
}

/*

#[test]
fn test_commit() {
    let path = "test_data2/git2/objects/79/c02b8801e3e3d262fdf1565cb5d21e24bde8de";
    let mut f = File::open(path).unwrap();
    let mut bytes = vec![];
    let _ = f.read_to_end(&mut bytes).unwrap();
    let bytes = decoder(&bytes).unwrap();
    let s = String::from_utf8_lossy(bytes.as_slice());
    println!("{:?}", bytes);

    let ok = vec![99, 111, 109, 109, 105, 116, 32, 49, 56, 49, 0, 116, 114, 101, 101, 32, 52, 55, 102, 100, 53, 98, 56, 54, 57, 52, 48, 100, 54, 98, 101, 49, 55, 51, 98, 99, 55, 54, 53, 100, 51, 100, 102, 50, 55, 53, 55, 100, 56, 97, 100, 55, 100, 54, 48, 57, 10, 97, 117, 116, 104, 111, 114, 32, 98, 101, 114, 110, 105, 101, 32, 60, 120, 105, 111, 110, 103, 121, 117, 97, 110, 98, 105, 97, 111, 48, 49, 64, 114, 101, 110, 109, 97, 105, 116, 101, 99, 104, 46, 99, 111, 109, 62, 32, 49, 54, 49, 55, 49, 55, 52, 54, 53, 48, 32, 43, 48, 56, 48, 48, 10, 99, 111, 109, 109, 105, 116, 116, 101, 114, 32, 98, 101, 114, 110, 105, 101, 32, 60, 120, 105, 111, 110, 103, 121, 117, 97, 110, 98, 105, 97, 111, 48, 49, 64, 114, 101, 110, 109, 97, 105, 116, 101, 99, 104, 46, 99, 111, 109, 62, 32, 49, 54, 49, 55, 49, 55, 52, 54, 53, 48, 32, 43, 48, 56, 48, 48, 10, 10, 118, 49, 10];
    let er = vec![99, 111, 109, 109, 105, 116, 32, 49, 56, 49, 0, 116, 114, 101, 101, 32, 52, 55, 102, 100, 53, 98, 56, 54, 57, 52, 48, 100, 54, 98, 101, 49, 55, 51, 98, 99, 55, 54, 53, 100, 51, 100, 102, 50, 55, 53, 55, 100, 56, 97, 100, 55, 100, 54, 48, 57, 10, 97, 117, 116, 104, 111, 114, 32, 98, 101, 114, 110, 105, 101, 32, 60, 120, 105, 111, 110, 103, 121, 117, 97, 110, 98, 105, 97, 111, 48, 49, 64, 114, 101, 110, 109, 97, 105, 116, 101, 99, 104, 46, 99, 111, 109, 62, 32, 49, 54, 49, 55, 48, 56, 56, 53, 54, 53, 32, 43, 48, 56, 48, 48, 10, 99, 111, 109, 109, 105, 116, 116, 101, 114, 32, 98, 101, 114, 110, 105, 101, 32, 60, 120, 105, 111, 110, 103, 121, 117, 97, 110, 98, 105, 97, 111, 48, 49, 64, 114, 101, 110, 109, 97, 105, 116, 101, 99, 104, 46, 99, 111, 109, 62, 32, 49, 54, 49, 55, 48, 56, 56, 53, 54, 53, 32, 43, 48, 56, 48, 48, 10, 10, 118, 49, 10];
    println!("{}", String::from_utf8_lossy(&ok));
    println!("{}", String::from_utf8_lossy(&er));
    let s = "commit 181\0tree 47fd5b86940d6be173bc765d3df2757d8ad7d609\nauthor bernie <xiongyuanbiao01@renmaitech.com> 1617174650 +0800\ncommitter bernie <xiongyuanbiao01@renmaitech.com> 1617174650 +0800\n\nv1\n";
    let mut sha = Sha1::new();
    sha.update(s.as_bytes());
    println!("{}, {}", s.len(), sha.digest().to_string());


    let path = "test_data2/.git/objects/47/fd5b86940d6be173bc765d3df2757d8ad7d609";
    let meta = std::fs::metadata(path).unwrap();
    println!("{}", meta.st_ctime());
}*/
