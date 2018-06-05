extern crate im;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate log4rs_syslog;
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

#[macro_use]
mod logger;
mod config;
mod difference;
mod history;
mod server;

use clap::{App, Arg, SubCommand};
use logger::AppLogger;
use std::io::{stdout, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    AppLogger::init();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_authors!())
        .about("Synchronize directories bidirectional")
        .subcommands(vec![
            SubCommand::with_name("sync")
                .about("Sync pair of directories")
                .arg(
                    Arg::with_name("DIRECTORIES")
                        .index(1)
                        .takes_value(true)
                        .multiple(true),
                ),
            SubCommand::with_name("add")
                .about("Add pair of directories targets to be monitored")
                .arg(
                    Arg::with_name("DIRECTORIES")
                        .index(1)
                        .takes_value(true)
                        .multiple(true),
                ),
            SubCommand::with_name("watch").about("Start monitoring directories"),
            SubCommand::with_name("stop").about("Stop monitoring directories"),
            SubCommand::with_name("list").about("Show list of directories to be monitored"),
            // TODO: Want to add tail and head option
            SubCommand::with_name("log").about("Stop watch process"),
        ])
        .get_matches();

    let mut watch_targets = config::WatchTargets::new();
    let message = match matches.subcommand() {
        ("sync", Some(cmd)) => {
            let directories = values_t!(cmd.values_of("DIRECTORIES"), String).unwrap_or(vec![]);
            let dir_a = directories.get(0).unwrap();
            let dir_b = directories.get(1).unwrap();
            server::sync(
                Path::new(&dir_a).to_path_buf().canonicalize().unwrap(),
                Path::new(&dir_b).to_path_buf().canonicalize().unwrap(),
            );
            None
        }
        ("add", Some(cmd)) => {
            let directories = values_t!(cmd.values_of("DIRECTORIES"), String).unwrap();
            let dir_a = directories.get(0).unwrap();
            let dir_b = directories.get(1).unwrap();
            watch_targets.add((
                Path::new(&dir_a).to_path_buf().canonicalize().unwrap(),
                Path::new(&dir_b).to_path_buf().canonicalize().unwrap(),
            ));
            let mut client = match UnixStream::connect(server::SOCKET_ADDR) {
                Ok(socket) => socket,
                Err(e) => exit_with_log!("UnixStream exit_with_log!\n{:?}", e),
            };
            let payload = serde_json::to_vec(&server::Command::Add(watch_targets)).unwrap();
            let _ = client.write_all(payload.as_slice());
            None
        }
        ("watch", Some(_)) => {
            let (snd, rcv) = channel();
            let promise = thread::spawn(move || server::listen(snd, watch_targets.clone()));
            let _ = rcv.recv();
            let _ = promise.join();
            None
        }
        ("stop", Some(_)) => {
            let mut client = match UnixStream::connect(server::SOCKET_ADDR) {
                Ok(socket) => socket,
                Err(e) => exit_with_log!("UnixStream exit_with_log!\n{:?}", e),
            };
            let payload = serde_json::to_vec(&server::Command::Kill).unwrap();
            let _ = client.write_all(payload.as_slice());
            None
        }
        ("list", Some(_)) => Some(watch_targets.list()),
        ("log", Some(_)) => Some(AppLogger::show_log().trim_right().to_owned()),
        // TODO: If it doesn't present any options, the tool sync all directories saved at .conf file
        _ => exit_with_log!("No subcommands has been specified"),
    };
    if let Some(msg) = message {
        let _ = stdout().write(format!("{}\n", msg).as_bytes());
    };
}
