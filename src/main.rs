extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::{Path, PathBuf};
use std::fs::{copy, create_dir_all};
use im::ConsList;

mod history;

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

fn collect_diff(from: &history::History, to: &history::History) -> ConsList<Difference> {
    from.histories.iter().fold(
        ConsList::new(),
        |acc, (path, history)| {
            let mut source_path = from.root.clone();
            let mut dist_path = to.root.clone();
            source_path.push(path.as_ref());
            dist_path.push(path.as_ref());
          unimplemented!();
        }
        //     |acc: ConsList<Difference>, (path, source_summary)| {

    //         match to.get(&path) {
    //             Some(dist_summary) => {
    //                 let source_modified = source_summary.modified;
    //                 let dist_modified = dist_summary.modified;
    //                 if source_modified >= dist_modified {
    //                     acc.cons(Difference::new(source_path, dist_path))
    //                 } else {
    //                     acc
    //                 }
    //             }
    //             None => acc.cons(Difference::new(source_path, dist_path)),
    //         }
    //     },
    )
}

fn main() {
    let a_path = Path::new("./fixture/a").to_owned();
    let b_path = Path::new("./fixture/b").to_owned();
    let a_history = history::History::new(a_path);
    let b_history = history::History::new(b_path);
    println!("{:?}", a_history);
    println!("{:?}", b_history);

    let diff_a = collect_diff(&a_history, &b_history);
    println!("{:?}", diff_a);
    diff_a.iter().for_each(|diff| {
        let _ = create_dir_all(diff.to.parent().unwrap());
        let _ = copy(&diff.from, &diff.to);
    })
}
