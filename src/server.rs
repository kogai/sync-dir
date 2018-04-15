use config::WatchTargets;
use difference::Differences;
use history::History;
use libudev::{Context, EventType, Monitor};
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

    // diff_a.iter().for_each(|diff| diff.sync_file());
    // diff_b.iter().for_each(|diff| diff.sync_file());
    diff_a.merge_with(diff_b).sync_all();
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
                let targets = watch_targets.lock().unwrap().get_available_directories();
                println!("Syncing...{:?}", targets);
                targets.iter().for_each(|x| {
                    let a = &x.0;
                    let b = &x.1;
                    sync(a.clone(), b.clone());
                });
                println!("All directories synchronized.");
            }
            // NOTE: It might be better to consider whether to handle a case when a user reject some device while syncing directories
            // EventType::Remove => {}
            x => {
                println!("Event::{}", x);
            }
        };
    }
}
