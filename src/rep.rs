use std::fs::File;
use std::io::{Read};
use std::path::{PathBuf};

use anyhow::Result;
use crate::{walk_dir, write_object_to_file};
use crate::hasher::{generic_blob_hash, generic_tree_hash};
use crate::index::{Entry, Index};
use crate::objects::commit::Commit;
use crate::objects::tree::Tree;

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
        let file = PathBuf::from(file);
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
            &self.current_head_path()?,
            sha1.digest().to_string() + "\n",
        )?;
        Ok(())
    }

    pub fn checkout(&self, name: &str, new: bool) -> Result<()> {
        let curr_hash = self.read_branch_hasher(&self.current_head_path()?)?;
        //新建一个分支, 写到HEAD里
        if new {
            let path = PathBuf::new().join(".git/HEAD");
            let content = format!("ref: refs/heads/{}", name);
            self.write_branch_hasher(&path, content)?;
        }
        //新建分支文件, 将HEAD hasher写入
        let path = PathBuf::new().join(".git/refs/heads").join(name);
        self.write_branch_hasher(&path, curr_hash)?;
        //誊文件

        Ok(())
    }

    pub fn list_branch_names(&self) -> Result<()> {
        let current_branch_name = self.current_head()?;
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

    pub fn status(&self) {
    }
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

    fn current_head_path(&self) -> Result<PathBuf> {
        let name = self.current_head()?;
        let head = PathBuf::new().join(".git").join(name);
        Ok(head)
    }

    fn current_head(&self) -> Result<String> {
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
        Ok(content)
    }

    fn write_branch_hasher(&self, path: &PathBuf, content: String) -> Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_repository() {
    }
}