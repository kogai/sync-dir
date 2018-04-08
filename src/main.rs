extern crate im;

use std::io::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs::{copy, create_dir_all, read_dir, DirEntry, FileType};
use im::HashMap;

#[derive(Debug)]
pub struct Summary {
    modified: SystemTime,
    file_type: FileType,
    path: PathBuf,
}

impl Summary {
    fn inner_new(dir_entry: &DirEntry) -> Result<Self> {
        let file_type = try!(dir_entry.file_type());
        let metadata = try!(dir_entry.metadata());
        let modified = try!(metadata.modified());
        Ok(Summary {
            modified: modified,
            file_type: file_type,
            path: dir_entry.path(),
        })
    }
    fn new(dir_entry: &DirEntry) -> Self {
        Summary::inner_new(dir_entry).unwrap()
    }
}
type Collection = HashMap<PathBuf, Summary>;

fn correct<P: AsRef<Path>>(path: P) -> Collection {
    let path = Path::new(path.as_ref());
    match read_dir(path) {
        Ok(entries) => entries
            .fold(Ok(HashMap::new()), |acc: Result<Collection>, d| {
                let dir_entry = try!(d);
                let file_type = try!(dir_entry.file_type());
                let result = try!(acc);
                let a = Summary::new(&dir_entry);
                let key_with_root = dir_entry.path();
                let key = key_with_root.strip_prefix(&path).unwrap().to_path_buf();
                if file_type.is_dir() {
                    Ok(result.union(&correct(&key_with_root)))
                } else {
                    Ok(result.insert(key, a))
                }
            })
            .unwrap(),
        Err(err) => unreachable!(err),
    }
}

#[derive(Debug)]
pub struct Difference {
    from: PathBuf,
    to: PathBuf,
}

impl Difference {
    fn new(from: PathBuf, to: PathBuf) -> Self {
        Difference { from, to }
    }
}

fn collect_diff<P: AsRef<Path>>(
    from: &Collection,
    to: &Collection,
    root_of_from: P,
    root_of_to: P,
) -> im::ConsList<Difference> {
    from.iter().fold(
        im::ConsList::new(),
        |acc: im::ConsList<Difference>, (path, source_summary)| {
            let mut source_path = Path::new(root_of_from.as_ref()).to_path_buf();
            let mut dist_path = Path::new(root_of_to.as_ref()).to_path_buf();
            source_path.push(path.as_ref());
            dist_path.push(path.as_ref());

            match to.get(&path) {
                Some(dist_summary) => {
                    let source_modified = source_summary.modified;
                    let dist_modified = dist_summary.modified;
                    if source_modified >= dist_modified {
                        acc.cons(Difference::new(source_path, dist_path))
                    } else {
                        acc
                    }
                }
                None => acc.cons(Difference::new(source_path, dist_path)),
            }
        },
    )
}

fn main() {
    let a = correct("./fixture/a");
    let b = correct("./fixture/b");
    let diff_a = collect_diff(&a, &b, "./fixture/a", "./fixture/b");
    println!("{:?}", diff_a);
    diff_a.iter().for_each(|diff| {
        let _ = create_dir_all(diff.to.parent().unwrap());
        let _ = copy(&diff.from, &diff.to);
    })
}
