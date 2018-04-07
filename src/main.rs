use std::path::Path;
use std::fs::{copy, read_dir, DirEntry, File};

fn open_file(d: std::io::Result<DirEntry>) -> std::io::Result<()> {
    let dir_entry = try!(d);
    let file = try!(File::open(dir_entry.path()));
    let meta = try!(dir_entry.metadata());
    let time = try!(meta.modified());
    println!("{:?} was modified at {:?}", file, time);
    // try!(copy(dir_entry.path(), Path::new("./fixture/b/1")));
    Ok(())
}

fn main() {
    let my_dir = read_dir("./fixture/a");
    match my_dir {
        Ok(dir) => {
            dir.for_each(|dir_entry| {
                open_file(dir_entry);
            });
        }
        Err(x) => {
            println!("{}", x);
        }
    };
}
