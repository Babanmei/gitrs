use nom::lib::std::collections::HashMap;
use crate::objects::blob::Blob;
use crate::read_object;
use anyhow::Result;

#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub name: String,
    pub size: usize,
    pub sha1: String,
    pub blobs: HashMap<String, Blob>,
    pub child_tree: HashMap<String, Box<Tree>>,
}

impl Tree {
    pub fn new(name: String) -> Tree {
        Tree {
            name,
            ..Default::default()
        }
    }

    pub fn add_blob(&mut self, name: String, blob: Blob) {
        self.blobs.insert(name, blob);
    }

    pub fn add_child_tree(&mut self, name: String, child: Tree) {
        self.child_tree.insert(name, Box::new(child));
    }

    pub fn set_size(&mut self, c: usize) {
        self.size = c;
    }

    pub fn find_blob(&self, name: String) -> Option<Blob> {
        if name.contains("/") {
            let ct = &self.child_tree;
            let index = name.find("/").unwrap();
            let first = &name[0..index];
            match ct.get(first) {
                Some(child) => {
                    let cs = &name[index + 1..];
                    let x = child.find_blob(cs.to_string());
                    if x.is_some() {
                        let x = x.unwrap().clone();
                        return Some(x);
                    }
                }
                None => return None,
            }
        } else {
            let bs = &self.blobs;
            let blob = bs.get(&name);
            if blob.is_some() {
                return Some(blob.unwrap().clone());
            }
        }
        None
    }
}

/// 从文件构建树
impl Tree {
    pub fn from_hasher(hasher: &str) -> Result<Tree> {
        let bytes = read_object(&hasher)?;
        let mut tree = Tree::new("".to_string());
        parse_tree(&mut tree, bytes.as_slice());
        Ok(tree)
    }
}


use nom::{do_parse, is_not, named, tag, take, take_until};

pub fn parse_tree(tree: &mut Tree, bytes: &[u8]) {
    let (mut input, count) = tree_head(bytes).unwrap();
    tree.set_size(b_2_i(count));
    loop {
        if let Ok((surplus, (mode, name, sha))) = tree_body(input) {
            let name = bytes_to_str(name);
            let sha_str = slice_to_sha_string(sha);
            match bytes_to_str(mode).as_str() {
                "40000" => {
                    let bytes = read_object(&sha_str).unwrap();
                    let mut child_tree = Tree::new(name.clone());
                    parse_tree(&mut child_tree, bytes.as_slice());
                    tree.add_child_tree(name, child_tree);
                }
                _ => tree.add_blob(name.clone(), Blob::from(&name, &sha_str).unwrap()),
            }
            if surplus.len() == 0 {
                break;
            }
            input = surplus;
        }
    }
}

named!(tree_head,
    do_parse!(
       _tree: tag!("tree ") >>
       count: is_not!("\0") >>
       _c1: take!(1) >>
       (count)
    )
);


named!(tree_body<(&[u8], &[u8], &[u8])>,
    do_parse!(
        mode: take_until!(" ") >>
        _s: take!(1) >>
        name: take_until!("\0") >>
        _s2: take!(1) >>
        sha: take!(20) >>
        ((mode, name, sha))
    )
);

pub fn slice_to_sha_string(b: &[u8]) -> String {
    let mut hasher = String::new();
    for x in b {
        let s = if *x < 16_u8 {
            format!("0{:x}", x)
        } else {
            format!("{:x}", x)
        };
        hasher.push_str(&s);
    }
    hasher
}

fn bytes_to_str(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

fn b_2_i(b: &[u8]) -> usize {
    let c = String::from_utf8(b.to_vec()).unwrap();
    let i = c.parse::<i32>().unwrap();
    i as usize
}
