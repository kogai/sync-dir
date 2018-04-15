use history::{Event, History};
use im::ConsList;
use std::fs::{copy, create_dir_all, remove_file};
use std::path::PathBuf;
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};
use std::io::{stdout, Write};

#[derive(Debug)]
pub struct Difference {
    from: PathBuf,
    to: PathBuf,
    event: Event,
}

impl Difference {
    fn new(from: PathBuf, to: PathBuf, event: Event) -> Self {
        Difference { from, to, event }
    }

    pub fn sync_file(&self, sender: &Sender<()>) {
        match &self.event {
            &Event::Delete(_) => {
                let _ = remove_file(&self.to);
                let _ = sender.send(());
            }
            _ => {
                let _ = create_dir_all(self.to.parent().unwrap());
                let _ = copy(&self.from, &self.to);
                let _ = sender.send(());
            }
        }
    }
}

pub struct Differences(ConsList<Difference>);

impl Differences {
    pub fn new(from: &History, to: &History) -> Self {
        let list = from.histories
            .iter()
            .fold(ConsList::new(), |acc, (path, history)| {
                if from.is_history(&path) {
                    return acc;
                }
                let mut source_path = from.root.clone();
                let mut dist_path = to.root.clone();
                source_path.push(path.as_ref());
                dist_path.push(path.as_ref());
                match to.histories.get(&path) {
                    Some(dist_history) => match (history.head(), dist_history.head()) {
                        (Some(h1), Some(h2)) => {
                            if h1.get_timestamp() > h2.get_timestamp() {
                                acc.cons(Difference::new(source_path, dist_path, *h1))
                            } else {
                                acc
                            }
                        }
                        (_, _) => unreachable!(),
                    },
                    None => acc.cons(Difference::new(
                        source_path,
                        dist_path,
                        *history.head().unwrap(),
                    )),
                }
            });
        Differences(list)
    }

    pub fn merge_with(&self, to: Self) -> Self {
        let a = &self.0;
        let b = &to.0;
        a.append(b);
        Differences(a.append(b))
    }

    pub fn sync_all(&self) {
        let diffs = self.0.iter();
        let max = diffs.len();
        let mut completed = 0;
        let (sender, receiver) = channel();
        let promise = spawn(move || diffs.for_each(|diff| diff.sync_file(&sender)));

        let stdout = stdout();
        let mut handle = stdout.lock();
        let throttle = Duration::from_millis(10);
        let progress = "=";
        loop {
            if completed >= max {
                handle.write(b">\n").unwrap();
                break;
            };
            match receiver.recv_timeout(throttle) {
                Ok(_) => completed += 1,
                _ => {}
            };
            handle.write(b"\r").unwrap();
            sleep(throttle);
            handle
                .write(format!("{}", progress.repeat(completed)).as_bytes())
                .unwrap();
        }
        let _ = promise.join();
    }
}
