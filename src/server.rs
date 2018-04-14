use clap::{App, Arg};
use libudev::{Context, Monitor};
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn listen() {
    let context = Context::new().unwrap();
    let mut monitor = Monitor::new(&context).unwrap();

    let _ = monitor.match_subsystem_devtype("usb", "usb_device");
    // TODO: Examine whether possible to mount device via bluetooth
    let _ = monitor.match_subsystem_devtype("bluetooth", "link");
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
}
