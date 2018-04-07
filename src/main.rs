use std::fs::read_dir;

fn main() {
    let my_dir = read_dir("./fixture/a");
    match my_dir {
        Ok(dir) => {
            dir.for_each(|file| {
                if let Ok(f) = file {
                    if let Ok(meta) = f.metadata() {
                        if let Ok(t) = meta.modified() {
                            println!("{:?} was modified at {:?}", f.file_name(), t);
                        }
                    }
                }
            });
        }
        Err(x) => {
            println!("{}", x);
        }
    }
}
