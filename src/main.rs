extern crate im;

use std::io::Result;
use std::path::{Path, PathBuf};
use std::fs::{copy, read_dir, DirEntry, File};
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

fn correct<P: AsRef<Path>>(path: P) -> HashMap<PathBuf, DirEntry> {
    match read_dir(path) {
        Ok(entries) => entries
            .fold(
                Ok(HashMap::new()),
                |acc: Result<HashMap<PathBuf, DirEntry>>, d| {
                    let dir_entry = try!(d);
                    let file_type = try!(dir_entry.file_type());
                    let result = try!(acc);
                    if file_type.is_dir() {
                        Ok(result.union(&correct(dir_entry.path())))
                    } else {
                        Ok(result.insert(dir_entry.path(), dir_entry))
                    }
                },
            )
            .unwrap(),
        Err(err) => unreachable!(err),
    }
}

fn main() {
    let a = correct("./fixture/a");
    let b = correct("./fixture/b");
    println!("{:?}", a);
    println!("{:?}", b);
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
