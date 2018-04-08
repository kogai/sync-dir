extern crate im;

use std::io::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs::{copy, read_dir, DirEntry, File, FileType};
use im::HashMap;

fn open_file(d: std::io::Result<DirEntry>) -> Result<()> {
    let dir_entry = try!(d);
    let file = try!(File::open(dir_entry.path()));
    let meta = try!(dir_entry.metadata());
    let time = try!(meta.modified());
    println!("{:?} was modified at {:?}", file, time);
    // try!(copy(dir_entry.path(), Path::new("./fixture/b/1")));
    Ok(())
}

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
    match read_dir(path) {
        Ok(entries) => entries
            .fold(Ok(HashMap::new()), |acc: Result<Collection>, d| {
                let dir_entry = try!(d);
                let file_type = try!(dir_entry.file_type());
                let result = try!(acc);
                let a = Summary::new(&dir_entry);
                if file_type.is_dir() {
                    Ok(result.union(&correct(dir_entry.path())))
                } else {
                    Ok(result.insert(dir_entry.path(), a))
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

fn collect_diff(
    from: &Collection,
    to: &Collection,
    root_of_from: PathBuf,
    root_of_to: PathBuf,
) -> im::ConsList<Difference> {
    from.difference(to).iter().fold(
        im::ConsList::new(),
        |acc: im::ConsList<Difference>, (path, _summary)| {
            let root = root_of_from.to_str().unwrap();
            let to_path = path.strip_prefix(root).unwrap();
            let mut path_to = Path::new(&root_of_to).to_path_buf();
            path_to.push(to_path);
            let my_result = Difference::new(Path::new(path.as_ref()).to_path_buf(), path_to);
            acc.cons(my_result)
        },
    )
}

fn main() {
    let a = correct("./fixture/a");
    let b = correct("./fixture/b");
    // println!("{:?}", a);
    // println!("{:?}", b);
    let diff_a = collect_diff(
        &a,
        &b,
        Path::new("./fixture/a").to_path_buf(),
        Path::new("./fixture/b").to_path_buf(),
    );
    println!("{:?}", diff_a);
    diff_a.iter().for_each(|diff| {
        match copy(&diff.from, &diff.to) {
            Ok(id) => {
                println!("Copy succeed {}", id);
            }
            Err(e) => {
                println!("Copy failed {:?}", e);
            }
        };
    })
    // match my_dir {
    //     Ok(dir) => {
    //         dir.for_each(|dir_entry| {
    //             open_file(dir_entry);
    //         });
    //     }
    //     Err(x) => {
    //         println!("{}", x);
    //     }
    // };
}
