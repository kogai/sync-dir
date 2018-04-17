use config::WatchTargets;
use difference::Differences;
use history::History;
use libusb::Context;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn sync(a_path: PathBuf, b_path: PathBuf) {
    let a_history = History::new(a_path);
    let b_history = History::new(b_path);

    let diff_a = Differences::new(&a_history, &b_history);
    let diff_b = Differences::new(&b_history, &a_history);

    diff_a.merge_with(diff_b).sync_all();
}

enum EventType {
    Add,
    Remove,
    Stay,
}

pub fn listen(dir_listener: Receiver<Arc<Mutex<WatchTargets>>>) {
    let context = Context::new().unwrap();
    let throttle = Duration::from_millis(10);
    let mut watch_targets = dir_listener.recv().unwrap();
    let mut current_devices = 0;
    println!("Start listening...");

    loop {
        watch_targets = match dir_listener.recv_timeout(throttle) {
            Ok(x) => x,
            _ => watch_targets,
        };

        let next_devices = context.devices().unwrap().iter().count();
        let event_type = match (current_devices, next_devices) {
            (c, n) if c < n => EventType::Add,
            (c, n) if c > n => EventType::Remove,
            _ => EventType::Stay,
        };
        current_devices = next_devices;
        match event_type {
            EventType::Stay | EventType::Remove => {
                thread::sleep(throttle);
                continue;
            }
            EventType::Add => {
                let targets = watch_targets.lock().unwrap().get_available_directories();
                println!("Syncing...{:?}", targets);
                targets.iter().for_each(|x| {
                    let a = &x.0;
                    let b = &x.1;
                    sync(a.clone(), b.clone());
                });
                println!("All directories synchronized.");
            }
        }
    }
}
