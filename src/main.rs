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
use std::thread;
use std::time::Duration;

mod config;
mod difference;
mod history;

fn main() {
    let context = libudev::Context::new().unwrap();
    let mut monitor = libudev::Monitor::new(&context).unwrap();

    let _ = monitor.match_subsystem_devtype("usb", "usb_device");
    let mut socket = monitor.listen().unwrap();

    loop {
        let event = match socket.receive_event() {
            Some(evt) => evt,
            None => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        println!(
            "{}: {} {} (sysname={}, devtype={})",
            event.sequence_number(),
            event.event_type(),
            event.syspath().to_str().unwrap_or("---"),
            event.sysname().to_str().unwrap_or(""),
            event.devtype().map_or("", |s| s.to_str().unwrap_or(""))
        );
    }
    // let context = libudev::Context::new().unwrap();
    // let mut enumerator = libudev::Enumerator::new(&context).unwrap();

    // enumerator.match_subsystem("tty").unwrap();

    // for device in enumerator.scan_devices().unwrap() {
    //     // println!("found device: {:?}", device.syspath());
    //     println!("");
    //     println!("initialized: {:?}", device.is_initialized());
    //     println!("     devnum: {:?}", device.devnum());
    //     println!("    syspath: {:?}", device.syspath());
    //     println!("    devpath: {:?}", device.devpath());
    //     println!("  subsystem: {:?}", device.subsystem());
    //     println!("    sysname: {:?}", device.sysname());
    //     println!("     sysnum: {:?}", device.sysnum());
    //     println!("    devtype: {:?}", device.devtype());
    //     println!("     driver: {:?}", device.driver());
    //     println!("    devnode: {:?}", device.devnode());
    // }
    /*
    let (name, version) = config::Config::get_config();

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
    */}
