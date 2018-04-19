use config::WatchTargets;
use difference::Differences;
use history::History;
use libusb::Context;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::process;
use std::fs::remove_file;
use std::io::Read;
use std::time::Duration;
use serde_json;

pub const SOCKET_ADDR: &'static str = "/tmp/sync-dir.sock";

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Add(WatchTargets),
    Kill,
}

pub fn listen(done: Sender<()>, initial_watch_targets: WatchTargets) {
    remove_file(SOCKET_ADDR).unwrap_or(());
    let (snd, rcv) = channel();
    println!("Start listening...");

    let _ = thread::spawn(move || {
        let listener = UnixListener::bind(SOCKET_ADDR).expect("Server process failed to start");
        let _ = done.send(());
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let cmd = parse_command(&mut stream);
                    match cmd {
                        Command::Kill => process::exit(0),
                        Command::Add(targets) => {
                            let _ = snd.send(targets);
                        }
                    };
                }
                Err(e) => unreachable!("{:?}", e),
            };
        }
    });

    let mut watch_targets = initial_watch_targets;
    let context = Context::new().unwrap();
    let throttle = Duration::from_millis(10);
    let mut current_devices = context.devices().unwrap().iter().count();
    loop {
        watch_targets = match rcv.recv_timeout(throttle) {
            Ok(targets) => targets,
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
                let targets = watch_targets.get_available_directories();
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

fn parse_command(stream: &mut UnixStream) -> Command {
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    match serde_json::from_slice::<Command>(&buf) {
        Ok(cmd) => cmd,
        Err(e) => unreachable!("{:?}", e),
    }
}
