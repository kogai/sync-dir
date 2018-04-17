use history::{Event, History};
use im::*;
use std::fs::{copy, create_dir_all, remove_file};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};
use std::io::{stdout, Write};

#[derive(Debug, Clone)]
pub struct Difference {
    from: PathBuf,
    to: PathBuf,
    event: Event,
}

impl Hash for Difference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
    }
}

impl PartialEq for Difference {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
            || self.from == other.to && self.to == other.from
    }
}
impl Eq for Difference {}

impl Difference {
    fn new(from: PathBuf, to: PathBuf, event: Event) -> Self {
        Difference { from, to, event }
    }

    pub fn sync_file(&self, sender: &Sender<PathBuf>) {
        let filename = &self.to.file_name().unwrap();
        match &self.event {
            &Event::Delete(_) => {
                let _ = remove_file(&self.to);
                let _ = sender.send(Path::new(filename).to_owned());
            }
            _ => {
                let _ = create_dir_all(self.to.parent().unwrap());
                let _ = copy(&self.from, &self.to);
                let _ = sender.send(Path::new(filename).to_owned());
            }
        }
    }
}

#[derive(Debug)]
pub struct Differences(Set<Difference>);

impl Differences {
    pub fn new(from: &History, to: &History) -> Self {
        let list = from.histories
            .iter()
            .fold(Set::new(), |acc, (path, history)| {
                let source_path = path.to_path_buf();
                let dist_path = History::replace_with(&path, &from.root, &to.root);
                match to.histories.get(&dist_path) {
                    Some(dist_history) => match (history.head(), dist_history.head()) {
                        (Some(h1), Some(h2)) => {
                            if h1.get_timestamp() > h2.get_timestamp() {
                                acc.insert(Difference::new(source_path, dist_path, *h1))
                            } else {
                                acc
                            }
                        }
                        (_, _) => unreachable!(),
                    },
                    None => acc.insert(Difference::new(
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
        Differences(merge_diffs(a.clone(), b.clone()))
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
        loop {
            let file_name = match receiver.recv_timeout(throttle) {
                Ok(file_name) => {
                    completed += 1;
                    Some(file_name)
                }
                Err(_) => None,
            };
            handle.write(b"\r").unwrap();
            sleep(throttle);
            handle
                .write(format!("{}", derive_indicator(max, completed, file_name)).as_bytes())
                .unwrap();
            if completed >= max {
                break;
            };
        }
        let _ = promise.join();
    }
}

fn derive_indicator(max: usize, current: usize, file_name: Option<PathBuf>) -> String {
    let progress = "=";
    let pad = " ";
    let pst = (current as f32) / (max as f32);
    format!(
        "[{}]{}>{}[{}{}%]{}",
        match file_name {
            Some(ref file) => file.to_str().unwrap(),
            _ => "",
        }.to_owned(),
        progress.repeat((pst * 50.0).round() as usize),
        pad.repeat(((1.0 - pst) * 50.0).round() as usize),
        match pst {
            p if p == 1.0 => "",
            p if p < 0.1 => "  ",
            _ => " ",
        },
        (pst * 100.0).round(),
        if max == current { "\n" } else { "" }
    )
}

fn merge_diffs(a: Set<Difference>, b: Set<Difference>) -> Set<Difference> {
    fn find(x: &Difference, ys: &Set<Difference>) -> Option<Difference> {
        ys.iter().fold(
            None,
            |acc, y| {
                if &*y == x {
                    Some((*y).clone())
                } else {
                    acc
                }
            },
        )
    }

    fn difference(xs: &Set<Difference>, ys: &Set<Difference>) -> Set<Difference> {
        xs.iter().fold(ys.clone(), |acc, x| match find(&x, ys) {
            Some(y) => acc.remove(&y),
            _ => acc,
        })
    }

    a.iter()
        .fold(Set::new(), |acc: Set<Difference>, x| match find(&x, &b) {
            Some(y) => match (x.event, y.event) {
                (Event::Delete(_), _) => acc.insert(x),
                (_, Event::Delete(_)) => acc.insert(y),
                (xe, ye) => {
                    if xe.get_timestamp() >= ye.get_timestamp() {
                        acc.insert(x)
                    } else {
                        acc.insert(y)
                    }
                }
            },
            _ => acc.insert(x),
        })
        .union(difference(&a, &b))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_merge_diffs_ordinarly() {
        let a = Set::new().insert(Difference {
            from: Path::new("a/1").to_path_buf(),
            to: Path::new("b/1").to_path_buf(),
            event: Event::Create(0),
        });
        let b = Set::new().insert(Difference {
            from: Path::new("b/2").to_path_buf(),
            to: Path::new("a/2").to_path_buf(),
            event: Event::Create(0),
        });

        assert_eq!(
            merge_diffs(a, b),
            Set::new()
                .insert(Difference {
                    from: Path::new("a/1").to_path_buf(),
                    to: Path::new("b/1").to_path_buf(),
                    event: Event::Create(0),
                })
                .insert(Difference {
                    from: Path::new("b/2").to_path_buf(),
                    to: Path::new("a/2").to_path_buf(),
                    event: Event::Create(0),
                })
        );
    }

    #[test]
    fn test_merge_diffs_drop_old_history() {
        let a = Set::new().insert(Difference {
            from: Path::new("a/1").to_path_buf(),
            to: Path::new("b/1").to_path_buf(),
            event: Event::Create(0),
        });
        let b = Set::new().insert(Difference {
            from: Path::new("b/1").to_path_buf(),
            to: Path::new("a/1").to_path_buf(),
            event: Event::Create(1),
        });

        assert_eq!(
            merge_diffs(a, b),
            Set::new().insert(Difference {
                from: Path::new("b/1").to_path_buf(),
                to: Path::new("a/1").to_path_buf(),
                event: Event::Create(1),
            })
        );
    }

    #[test]
    fn test_merge_diffs_preffer_to_delete() {
        let a = Set::new().insert(Difference {
            from: Path::new("a/1").to_path_buf(),
            to: Path::new("b/1").to_path_buf(),
            event: Event::Delete(0),
        });
        let b = Set::new().insert(Difference {
            from: Path::new("b/1").to_path_buf(),
            to: Path::new("a/1").to_path_buf(),
            event: Event::Create(1),
        });

        assert_eq!(
            merge_diffs(a, b),
            Set::new().insert(Difference {
                from: Path::new("a/1").to_path_buf(),
                to: Path::new("b/1").to_path_buf(),
                event: Event::Delete(0),
            })
        );
    }

    #[test]
    fn test_indicator() {
        let file_name = Some(Path::new("foo.file").to_path_buf());
        assert_eq!(
            derive_indicator(5, 3, file_name.to_owned()),
            format!("[foo.file]{}>{}[ 60%]", "=".repeat(30), " ".repeat(20))
        );
        assert_eq!(
            derive_indicator(5, 5, file_name.to_owned()),
            format!("[foo.file]{}>[100%]\n", "=".repeat(50)) // format!("{}>\n", "=".repeat(100))
        );
    }
}
