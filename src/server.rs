use config::WatchTargets;
use difference::collect_diff;
use history::History;
use libudev::{Context, EventType, Monitor};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn sync(dir_a: String, dir_b: String) {
    let a_path = Path::new(&dir_a).to_owned();
    let b_path = Path::new(&dir_b).to_owned();

    let a_history = History::new(a_path);
    let b_history = History::new(b_path);

    let diff_a = collect_diff(&a_history, &b_history);
    let diff_b = collect_diff(&b_history, &a_history);

    diff_a.iter().for_each(|diff| diff.sync_file());
    diff_b.iter().for_each(|diff| diff.sync_file());
}

pub fn listen(
    dir_listener: Receiver<Arc<Mutex<WatchTargets>>> // initial_watch_targets: WatchTargets
) {
    let context = Context::new().unwrap();
    let throttle = Duration::from_millis(10);
    let mut watch_targets = dir_listener.recv().unwrap();
    let mut monitor = Monitor::new(&context).unwrap();
    println!("Start listening...");

    // TODO: Examine whether possible to mount device via bluetooth
    let _ = monitor.match_subsystem_devtype("bluetooth", "link");
    let _ = monitor.match_subsystem_devtype("usb", "usb_device");
    let mut socket = monitor.listen().unwrap();

    loop {
        watch_targets = match dir_listener.recv_timeout(throttle) {
            Ok(x) => x,
            _ => watch_targets,
        };
        let event = match socket.receive_event() {
            Some(evt) => evt,
            None => {
                thread::sleep(throttle);
                continue;
            }
        };
        match event.event_type() {
            EventType::Add => {
                // TODO: Check whether devices includes path defined in directories.
                println!("Syncing...{:?}", watch_targets);
                // sync("fixture/a".to_owned(), "fixture/b".to_owned());
                println!("Synchronounced");
            }
            // NOTE: It might be better to consider whether to handle a case when a user reject some device while syncing directories
            // EventType::Remove => {}
            x => {
                println!("Event::{}", x);
            }
        };
    }
}
