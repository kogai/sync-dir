use history::{Event, History};
use im::ConsList;
use std::fs::{copy, create_dir_all, remove_file};
use std::path::PathBuf;

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

    pub fn sync_file(&self) {
        match &self.event {
            Event::Delete(_) => {
                let _ = remove_file(&self.to);
            }
            _ => {
                let _ = create_dir_all(self.to.parent().unwrap());
                let _ = copy(&self.from, &self.to);
            }
        }
    }
}

pub fn collect_diff(from: &History, to: &History) -> ConsList<Difference> {
    from.histories
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
        })
}
