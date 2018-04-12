extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::path::Path;

mod difference;
mod history;

// TODO: Sync with Cagot.toml
const NAME: &str = "syncdir";
const VERSION: &str = "0.0.1";

fn main() {
    let matches = App::new(NAME)
        .version(VERSION)
        .about("Synchronize directories bidirectional")
        .arg(
            Arg::with_name("directories")
                .help("Set two directories you want to synchronize")
                .short("d")
                .index(1)
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let directories = values_t!(matches.values_of("directories"), String).unwrap_or(vec![]);
    match directories.len() {
        0 | 1 => println!("Number of directory you've passed are not enough"),
        2 => {
            println!("Directories {:?}", directories);

            let a_path = Path::new(directories.get(0).unwrap()).to_owned();
            let b_path = Path::new(directories.get(1).unwrap()).to_owned();
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
        _ => println!("Number of directory you've passed are too much"),
    };
}
