extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use im::ConsList;
use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};

mod history;

#[derive(Debug)]
pub struct Difference {
    from: PathBuf,
    to: PathBuf,
    event: history::Event,
}

impl Difference {
    fn new(from: PathBuf, to: PathBuf, event: history::Event) -> Self {
        Difference { from, to, event }
    }
}

fn collect_diff(from: &history::History, to: &history::History) -> ConsList<Difference> {
    from.histories
        .iter()
        .fold(ConsList::new(), |acc, (path, history)| {
            if from.is_history(&path) {
                return acc;
            }
            let mut source_path = from.root.clone();
            let mut dist_path = to.root.clone();
            source_path.push(path.as_ref());
            dist_path.push(path.as_ref());
            match to.histories.get(&path) {
                Some(dist_history) => match (history.head(), dist_history.head()) {
                    (Some(h1), Some(h2)) => {
                        if h1.get_timestamp() >= h2.get_timestamp() {
                            acc.cons(Difference::new(source_path, dist_path, *h1))
                        } else {
                            acc
                        }
                    }
                    (_, _) => unreachable!(),
                },
                None => acc.cons(Difference::new(
                    source_path,
                    dist_path,
                    *history.head().unwrap(),
                )),
            }
        })
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
    // diff_a.iter().for_each(|diff| {
    //     let _ = create_dir_all(diff.to.parent().unwrap());
    //     let _ = copy(&diff.from, &diff.to);
    // })
}
