extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::Path;

mod difference;
mod history;

fn main() {
    let a_path = Path::new("./fixture/a").to_owned();
    let b_path = Path::new("./fixture/b").to_owned();
    let a_history = history::History::new(a_path);
    let b_history = history::History::new(b_path);
    println!("{:?}", a_history);
    println!("{:?}", b_history);

    let diff_a = difference::collect_diff(&a_history, &b_history);
    let diff_b = difference::collect_diff(&b_history, &a_history);
    println!("{:?}", &diff_a);
    println!("{:?}", &diff_b);
    diff_a.iter().for_each(|diff| diff.sync_file());
    diff_b.iter().for_each(|diff| diff.sync_file());
}
