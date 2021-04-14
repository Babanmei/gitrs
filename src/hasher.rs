use std::path::PathBuf;
use std::io::{BufReader, Read};
use std::fs::File;
use sha1::Sha1;
use anyhow::Result;
use std::fs;
use crate::objects::tree::Tree;
use crate::objects::blob::Blob;
use crate::write_object_to_file;

/// tree <content length><NUL><file mode> <filename><NUL><item sha>...
pub fn generic_tree_hash(parent: &mut Tree, path: &PathBuf, files: &Vec<String>) -> (Sha1, Vec<u8>) {
    let mut tsize = 0;
    let mut tuple = vec![];
    for entry in fs::read_dir(path).unwrap().into_iter() {
        let file = entry.unwrap().path();
        let (class, mode, len) = classify(&file);
        let name = file.to_str().unwrap().trim_start_matches("./");
        for filter in files.iter() {
            if filter.contains(&name) {
                let name = file.file_name().unwrap().to_str().unwrap().to_string();
                tsize += mode.len() + 1 + name.len() + 1 + 20;
                tuple.push((file.clone(), mode.clone(), class.clone(), len, name.to_string()));
                break;
            }
        }
    }
    tuple.sort_by(|a, b| {
        let a_str = if a.1.as_str() == "40000" {
            a.4.clone() + "/"
        } else {
            a.4.clone()
        };
        let b_str = if b.1.as_str() == "40000" {
            b.4.clone() + "/"
        } else {
            b.4.clone()
        };
        a_str.partial_cmp(&b_str).unwrap()
    });

    let mut sha1 = Sha1::new();
    let s = format!("tree {}\0", tsize);
    let mut tree_body_str = vec![];
    tree_body_str.append(&mut s.as_bytes().to_vec());

    for (path, mode, _class, _len, encoded_form) in tuple {
        let hasher = match mode.as_str() {
            "40000" => {
                let mut child = Tree::new(encoded_form.clone());
                let (child_hasher, tree_body_bytes) = generic_tree_hash(&mut child, &path, files);

                write_object_to_file(
                    child_hasher.digest().to_string().as_str(),
                    &tree_body_bytes,
                ).unwrap();
                child.sha1 = child_hasher.clone().digest().to_string();
                parent.add_child_tree(encoded_form.clone(), child);
                child_hasher
            }
            _ => {
                let hasher = generic_hash(&path, mode.clone());
                let blob = Blob::new(encoded_form.clone());
                *blob.hasher.borrow_mut() = hasher.digest().bytes().to_vec();
                parent.add_blob(encoded_form.clone(), blob);
                hasher
            }
        };
        tree_body_str.append(&mut mode.as_bytes().to_vec());
        tree_body_str.append(&mut b" ".to_vec());
        tree_body_str.append(&mut encoded_form.as_bytes().to_vec());
        tree_body_str.append(&mut b"\0".to_vec());
        tree_body_str.append(&mut hasher.digest().bytes().to_vec());
    }

    sha1.update(tree_body_str.as_slice());

    parent.size = tsize;
    parent.sha1 = sha1.digest().to_string();
    (sha1, tree_body_str)
}

pub fn generic_symlink_hash(path: &PathBuf) -> Sha1 {
    let mut reader = BufReader::new(File::open(path).unwrap());
    let mut bytes: Vec<u8> = vec![];
    let _ = reader.read_to_end(&mut bytes).unwrap();

    let mut sha1 = Sha1::new();
    sha1.update(bytes.as_slice());
    sha1
}

pub fn generic_blob_hash(path: &PathBuf) -> Result<(Sha1, Vec<u8>)> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut bytes: Vec<u8> = vec![];
    reader.read_to_end(&mut bytes)?;
    let s = format!("blob {}\0", bytes.len());
    let mut sha1 = Sha1::new();
    sha1.update(s.as_bytes());
    sha1.update(bytes.as_slice());

    let mut body = vec![];
    body.append(&mut s.as_bytes().to_vec());
    body.append(&mut bytes);
    Ok((sha1, body))
}

fn generic_hash(path: &PathBuf, mode: String) -> Sha1 {
    let hash = match mode.as_str() {
        "120000" => generic_symlink_hash(path),
        _ => generic_blob_hash(path).unwrap().0,
    };
    hash
}

/// 对传入路径的文件或文件夹分类
pub fn classify(path: &PathBuf) -> (String, String, u64) {
    let md = fs::metadata(path).unwrap();
    let ftype = md.file_type();

    let (class, mode) = if ftype.is_dir() {
        ("tree", "40000")
    } else if ftype.is_file() {
        let mode = if md.permissions().readonly() {
            "100755"
        } else { "100644" };
        ("blob", mode)
    } else if ftype.is_symlink() {
        ("blob", "120000")
    } else {
        panic!("未指定文件类型")
    };
    return (class.to_string(), mode.to_string(), md.len());
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use flate2::read::ZlibDecoder;
    use crate::decoder;

    #[test]
    fn test_generic_tree_hash() {
        let mut tree = Tree::new(String::new());
        let mut filter = vec![
            String::from("test_data/d1/d1d/d1df"),
            String::from("test_data/d1/df1"),
            String::from("test_data/f1"),
        ];
        let path = PathBuf::from("./test_data");
        generic_tree_hash(&mut tree, &path, &filter);
        assert_eq!(tree.sha1.to_string().as_str(), "47fd5b86940d6be173bc765d3df2757d8ad7d609");

        filter.push(String::from("test_data/f2"));
        let path = PathBuf::from("./test_data");
        generic_tree_hash(&mut tree, &path, &filter);
        assert_eq!(tree.sha1.to_string().as_str(), "481651d9ca42c91589b10fe1c35b4ba83b2cf057");
    }

    #[test]
    fn test_generic_blob_hash() {
        let path = PathBuf::from("test_data/f1");
        let (sha, sha_vec) = generic_blob_hash(&path).unwrap();

        let mut file = File::open("test_data/.git/objects/8e/1e71d5ce34c01b6fe83bc5051545f2918c8c2b").unwrap();
        let mut z = ZlibDecoder::new(file);
        let mut bytes = vec![];
        let _ = z.read_to_end(&mut bytes).unwrap();

        assert_eq!(bytes, sha_vec);
        assert_eq!(sha.digest().to_string(), "8e1e71d5ce34c01b6fe83bc5051545f2918c8c2b")
    }
}