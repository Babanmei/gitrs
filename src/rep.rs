use std::fs::File;
use std::io::{Read};
use std::path::{PathBuf};

use anyhow::Result;
use crate::{walk_dir, write_object_to_file, blob_to_file};
use crate::hasher::{generic_blob_hash, generic_tree_hash};
use crate::index::{Entry, Index};
use crate::objects::commit::Commit;
use crate::objects::tree::Tree;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Repository {
    pub stage: Index,
}

impl Repository {
    pub fn new() -> Result<Repository> {
        let base = PathBuf::new();
        let git_path = base.join(".git");
        if !git_path.exists() {
            return Err(anyhow!("please init git"));
        }

        let index_path = git_path.join("index");
        let stage = if index_path.exists() {
            Index::from_index_file(&index_path)?
        } else { Index::default() };

        Ok(Repository {
            stage,
        })
    }

    pub fn add(&self, file: &str) -> Result<()> {
        let file = PathBuf::new().join(file);
        if file.is_dir() {
            let mut names = vec![];
            walk_dir(&file, &file, &mut names)?;
            for name in names {
                self.add_file(&name)?;
            }
        } else {
            self.add_file(&file)?;
        }
        //write file
        self.stage.write_index_file()?;
        Ok(())
    }

    pub fn commit(&self, msg: &str) -> Result<()> {
        let mut tree = Tree::new(String::new());

        let filter: Vec<String> = self
            .stage
            .entries
            .borrow()
            .iter()
            .map(|ent| ent.name.clone())
            .collect();

        let (tree_sha, tree_body_bytes) = generic_tree_hash(&mut tree, &PathBuf::from("."), &filter);
        write_object_to_file(
            tree_sha.digest().to_string().as_str(),
            &tree_body_bytes,
        )?;

        let commit = Commit::new(tree, msg.to_string());
        let (sha1, commit_body) = commit.generate_commit_body();
        write_object_to_file(
            sha1.digest().to_string().as_str(),
            &commit_body,
        )?;

        self.write_branch_hasher(
            &self.head_path()?,
            sha1.digest().to_string() + "\n",
        )?;
        Ok(())
    }

    pub fn create_new_branch(&self, name: &str) -> Result<()> {
        let curr_hash = self.read_branch_hasher(&self.head_path()?)?;
        //新建一个分支, 分支名称写到HEAD里
        let path = PathBuf::new().join(".git/HEAD");
        let content = format!("ref: refs/heads/{}", name);
        self.write_branch_hasher(&path, content)?;
        //新建分支文件, 将HEAD hasher写入
        let path = PathBuf::new().join(".git/refs/heads").join(name);
        self.write_branch_hasher(&path, curr_hash)?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let path = PathBuf::from(format!(".git/refs/heads/{}", name));
        let commit_hasher = self.read_branch_hasher(&path)?;
        //根据name_hasher构建一棵树
        let commit = Commit::from_hasher(&commit_hasher)?;

        let tree = commit.tree;
        let (pre, mut map) = (PathBuf::new(), HashMap::new());
        visit_tree(&tree, pre, &mut map);
        for (name, b) in tree.blobs.iter() {
            map.insert(PathBuf::from(name), b.content.clone());
        }

        for (path, content) in map.iter() {
            let (mt1, mt2) = blob_to_file(path, content)?;
            let file_name = path.file_name().unwrap().to_str().unwrap();
            self.stage.update_entrie_mtime(file_name, (mt1 as i32, mt2 as i32));
        }
        Ok(())
    }

    pub fn list_branch_names(&self) -> Result<()> {
        let current_branch_name = self.head_content()?;
        let path = PathBuf::new().join(".git/refs/heads");
        for ent in std::fs::read_dir(&path)?.into_iter() {
            let name = ent?.file_name().to_str().unwrap().to_string();
            if current_branch_name.contains(&name) {
                println!("* {}", name);
            } else {
                println!("  {}", name);
            }
        }
        Ok(())
    }

    pub fn status(&self) {}
}

impl Repository {
    fn add_file(&self, file: &PathBuf) -> Result<()> {
        let entry = Entry::from(file)?;
        let (sha, body) = generic_blob_hash(file)?;
        self.stage.add_entry(entry);
        let hasher = sha.digest().to_string();
        write_object_to_file(hasher.as_str(), &body)?;
        Ok(())
    }

    fn head_path(&self) -> Result<PathBuf> {
        let name = self.head_content()?;
        let head = PathBuf::new().join(".git").join(name);
        Ok(head)
    }

    fn head_content(&self) -> Result<String> {
        let head = PathBuf::new().join(".git").join("HEAD");
        let mut f = File::open(&head)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        let v: Vec<&str> = content.split(": ").collect();
        Ok(v[1].trim().to_string())
    }

    fn read_branch_hasher(&self, path: &PathBuf) -> Result<String> {
        let mut f = File::open(&path)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        Ok(content.trim().to_string())
    }

    fn write_branch_hasher(&self, path: &PathBuf, content: String) -> Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// 迭代遍历给定的tree, 生成目录结构->blob内容
fn visit_tree(tree: &Tree, pre: PathBuf, all_pair: &mut HashMap<PathBuf, Vec<u8>>) {
    for (dir_name, child) in tree.child_tree.iter() {
        let pre = pre.join(dir_name);
        for (name, b) in child.blobs.iter() {
            let pre = pre.join(name);
            all_pair.insert(pre.clone(), b.content.clone());
        }
        visit_tree(&*child, pre, all_pair)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_repository() {}
}