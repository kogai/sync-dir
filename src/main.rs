#![feature(attr_literals)]

extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate log;
extern crate toml;

use clap::{App, Arg};
// use std::path::Path;

mod difference;
mod history;

#[derive(RustEmbed)]
#[folder("./")]
struct Asset;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    package: PackageConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageConfig {
    name: String,
    version: String,
}

fn main() {
    let cargo_toml = Asset::get("Cargo.toml").unwrap();
    let cargo_toml = toml::from_str::<Config>(std::str::from_utf8(&cargo_toml).unwrap()).unwrap();

    let matches = App::new(cargo_toml.package.name)
        .version(cargo_toml.package.version.as_ref())
        .about("Synchronize directories bidirectional")
        .arg(
            Arg::with_name("dir_a")
                .help("Set directory [a] you want to synchronize")
                .index(1)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dir_b")
                .help("Set directory [b] you want to synchronize")
                .index(2)
                .takes_value(true),
        )
        .get_matches();

    let dir_a = value_t!(matches.value_of("dir_a"), String).unwrap();
    let dir_b = value_t!(matches.value_of("dir_b"), String).unwrap();
    println!("Directories {:?} {:?}", &dir_a, &dir_b);

    let a_path = Path::new(&dir_a).to_owned();
    let b_path = Path::new(&dir_b).to_owned();
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
