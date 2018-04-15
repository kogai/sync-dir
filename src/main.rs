extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate clap;
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

    // Setup CLI
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_authors!())
        .about("Synchronize directories bidirectional")
        .arg(
            Arg::with_name("synchronize")
                .help("Synchronize directories")
                .long("synchronize")
                .short("s")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("additional")
                .help("Add watch target of directories")
                .long("additional")
                .short("a")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("watch")
                .help("Watch targets while change status of devices")
                .long("watch")
                .short("w"),
        )
        .get_matches();

    if matches.is_present("synchronize") {
        let directories = values_t!(matches.values_of("synchronize"), String).unwrap();
        let dir_a = directories.get(0).unwrap();
        let dir_b = directories.get(1).unwrap();
        server::sync(
            Path::new(&dir_a).to_path_buf(),
            Path::new(&dir_b).to_path_buf(),
        );
        std::process::exit(0);
    };
    if matches.is_present("additional") {
        let directories = values_t!(matches.values_of("additional"), String).unwrap();
        let dir_a = directories.get(0).unwrap();
        let dir_b = directories.get(1).unwrap();
        watch_targets.lock().unwrap().add((
            Path::new(&dir_a).to_path_buf(),
            Path::new(&dir_b).to_path_buf(),
        ));
        let _ = sender.send(watch_targets.clone());
        std::process::exit(0);
    };
    if matches.is_present("watch") {
        let promise = thread::spawn(|| {
            server::listen(receiver);
        });
        let _ = sender.send(watch_targets.clone());
        let _ = promise.join();
    };
}
