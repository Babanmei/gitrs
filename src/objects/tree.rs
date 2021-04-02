use nom::lib::std::collections::HashMap;
use crate::objects::blob::Blob;
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
/*
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (name, blob) in self.blobs.borrow().iter(){
            //write!(blob)
        }
        Ok(())
    }
}
 */

