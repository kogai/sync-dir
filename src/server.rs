use difference::collect_diff;
use history::History;
use libudev::{Context, EventType, Monitor};
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn listen() {
    let context = Context::new().unwrap();
    let mut monitor = Monitor::new(&context).unwrap();

    // TODO: Examine whether possible to mount device via bluetooth
    let _ = monitor.match_subsystem_devtype("bluetooth", "link");
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

        match event.event_type() {
            EventType::Add => {
                println!("Syncing...");
                let a_path = Path::new("fixture/a").to_owned();
                let b_path = Path::new("fixture/b").to_owned();
                let a_history = History::new(a_path);
                let b_history = History::new(b_path);

                let diff_a = collect_diff(&a_history, &b_history);
                let diff_b = collect_diff(&b_history, &a_history);

                diff_a.iter().for_each(|diff| diff.sync_file());
                diff_b.iter().for_each(|diff| diff.sync_file());
                println!("Synchronounced");
            }
            // NOTE: It might be better to consider whether to handle a case when a user reject some device while syncing directories
            // EventType::Remove => {}
            _ => {}
        };
    }
}
