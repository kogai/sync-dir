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
extern crate libudev;
extern crate toml;

use clap::{App, Arg};
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

mod config;
mod difference;
mod history;
mod server;

fn main() {
    // Initialize server
    let (sender, receiver) = channel();
    let watch_targets = Arc::new(Mutex::new(config::WatchTargets::new()));
    let promise = thread::spawn(|| {
        server::listen(receiver);
    });
    let _ = sender.send(watch_targets.clone());

    // Setup CLI
    let (name, version) = config::Package::get_config();
    let matches = App::new(name)
        .version(version.as_ref())
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

    watch_targets.lock().unwrap().add((
        Path::new(&dir_a).to_path_buf(),
        Path::new(&dir_b).to_path_buf(),
    ));
    let _ = sender.send(watch_targets.clone());

    let _ = promise.join();
    println!("Server will terminate");
}
