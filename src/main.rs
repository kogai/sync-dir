extern crate im;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate clap;
extern crate libusb;
extern crate termion;
extern crate toml;

use clap::{App, Arg};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::io::Write;

mod config;
mod difference;
mod history;
mod server;

fn main() {
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
        .arg(
            Arg::with_name("kill")
                .help("Kill sync-dir daemon")
                .long("kill")
                .short("k"),
        )
        .get_matches();

    // Initialize server

    if matches.is_present("synchronize") {
        let directories = values_t!(matches.values_of("synchronize"), String).unwrap();
        let dir_a = directories.get(0).unwrap();
        let dir_b = directories.get(1).unwrap();
        server::sync(
            Path::new(&dir_a).to_path_buf().canonicalize().unwrap(),
            Path::new(&dir_b).to_path_buf().canonicalize().unwrap(),
        );
    };
    if matches.is_present("additional") {
        let mut watch_targets = config::WatchTargets::new();
        let directories = values_t!(matches.values_of("additional"), String).unwrap();
        let dir_a = directories.get(0).unwrap();
        let dir_b = directories.get(1).unwrap();
        watch_targets.add((
            Path::new(&dir_a).to_path_buf().canonicalize().unwrap(),
            Path::new(&dir_b).to_path_buf().canonicalize().unwrap(),
        ));
        let mut client = match UnixStream::connect(server::SOCKET_ADDR) {
            Ok(socket) => socket,
            Err(e) => unreachable!("UnixStream Error!\n{:?}", e),
        };
        let payload = serde_json::to_vec(&server::Command::Add(watch_targets)).unwrap();
        let _ = client.write_all(payload.as_slice());
    };
    if matches.is_present("kill") {
        let mut client = match UnixStream::connect(server::SOCKET_ADDR) {
            Ok(socket) => socket,
            Err(e) => unreachable!("UnixStream Error!\n{:?}", e),
        };
        let payload = serde_json::to_vec(&server::Command::Kill).unwrap();
        let _ = client.write_all(payload.as_slice());
    };
    if matches.is_present("watch") {
        let watch_targets = config::WatchTargets::new();
        let (snd, rcv) = channel();
        let promise = thread::spawn(move || server::listen(snd, watch_targets.clone()));
        let _ = rcv.recv();
        let _ = promise.join();
    };
    // TODO: If it doesn't present any options, the tool sync all directories saved at .conf file
}
