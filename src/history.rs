use im::{ConsList, HashMap};
use regex::Regex;
use serde_json;
use std::fs::{read_dir, File};
use std::io::{Read, Result, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;

fn i32_of_systemtime(x: SystemTime) -> i32 {
  let stringified = format!("{:?}", x);
  match Regex::new(r"tv_sec: ([0-9]+), tv_nsec") {
    Ok(reg) => {
      let result = reg
        .captures(&stringified)
        .map(|captures| captures.get(1))
        .unwrap()
        .unwrap()
        .as_str();
      i32::from_str(result).unwrap()
    }
    Err(e) => {
      println!("{:?}", e);
      unreachable!();
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Event {
  Create(i32),
  Change(i32),
  Delete(i32),
}

impl Event {
  pub fn get_timestamp(&self) -> i32 {
    match *self {
      Event::Create(t) | Event::Change(t) | Event::Delete(t) => t,
    }
  }
}

pub enum Dawn {
  PreHistory,
  HasHistory(HashMap<PathBuf, ConsList<Event>>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
  pub root: PathBuf,
  pub histories: HashMap<PathBuf, ConsList<Event>>,
}

impl History {
  pub fn history_path(root: &PathBuf) -> PathBuf {
    let mut history_path = Path::new(&root).to_path_buf();
    history_path.push(".history.json");
    history_path
  }

  pub fn is_history(&self, p: &PathBuf) -> bool {
    let mut path = Path::new(&self.root).to_path_buf();
    path.push(p);
    Self::history_path(&self.root) == path
  }

  pub fn new(root: PathBuf) -> Self {
    let histories = match File::open(&History::history_path(&root)) {
      Ok(mut file) => {
        let mut json_buf = Vec::new();
        match file.read_to_end(&mut json_buf) {
          Ok(_) => match serde_json::from_slice::<HashMap<PathBuf, ConsList<Event>>>(&json_buf) {
            Ok(old_histories) => {
              let new_histories = History::generate_history(
                root.clone(),
                None,
                &Dawn::HasHistory(old_histories.clone()),
              );
              // TODO: Handle case which always update .history.json
              old_histories.iter().fold(new_histories, |ns, old_history| {
                let key = old_history.clone().0;
                let value = old_history.clone().1;
                match ns.get(&key) {
                  Some(_) => ns,
                  None => {
                    let hd = value.head().unwrap();
                    match *hd {
                      Event::Delete(_) => ns.insert(key, value),
                      _ => ns.insert(
                        key,
                        value.cons(Event::Delete(i32_of_systemtime(SystemTime::now()))),
                      ),
                    }
                  }
                }
              })
            }
            Err(e) => {
              println!("JSON of history file can't parse normaly.\n{:?}", e);
              History::generate_history(root.clone(), None, &Dawn::PreHistory)
            }
          },
          Err(e) => {
            println!("History file can't read normaly.\n{:?}", e);
            History::generate_history(root.clone(), None, &Dawn::PreHistory)
          }
        }
      }
      Err(_) => History::generate_history(root.clone(), None, &Dawn::PreHistory),
    };
    let instance = History { root, histories };
    instance.write();
    instance
  }

  fn generate_history(
    root_path: PathBuf,
    current_path: Option<PathBuf>,
    has_history: &Dawn,
  ) -> HashMap<PathBuf, ConsList<Event>> {
    let strip_path = match current_path {
      Some(path) => path,
      None => root_path.clone(),
    };
    match read_dir(&root_path) {
      Ok(entries) => entries
        .fold(
          Ok(HashMap::new()),
          |acc: Result<HashMap<PathBuf, ConsList<Event>>>, dir_entry| {
            let dir_entry = try!(dir_entry);
            let file_type = try!(dir_entry.file_type());
            let metadata = try!(dir_entry.metadata());
            let modified = try!(metadata.modified());
            let acc = try!(acc);
            let key_with_root = dir_entry.path();
            let key = key_with_root
              .strip_prefix(&strip_path)
              .unwrap()
              .to_path_buf();
            let history_of_file = match *has_history {
              Dawn::PreHistory => ConsList::new().cons(Event::Create(i32_of_systemtime(modified))),
              Dawn::HasHistory(ref history) => match history.get(&key) {
                Some(h) => {
                  let timestamp_latest = h.head().unwrap().get_timestamp();
                  let event = Event::Change(i32_of_systemtime(modified));
                  if event.get_timestamp() > timestamp_latest {
                    h.cons(event)
                  } else {
                    ConsList::new().append(h)
                  }
                }
                None => ConsList::new().cons(Event::Create(i32_of_systemtime(modified))),
              },
            };
            if file_type.is_dir() {
              Ok(acc.union(&History::generate_history(
                key_with_root,
                Some(root_path.clone()),
                has_history,
              )))
            } else if &key.to_string_lossy() == ".history.json" {
              Ok(acc)
            } else {
              Ok(acc.insert(key, history_of_file))
            }
          },
        )
        .unwrap(),
      Err(e) => unreachable!(e),
    }
  }

  fn write(&self) {
    let history_path = History::history_path(&self.root);
    match (
      File::create(&history_path),
      serde_json::to_string_pretty(&self.histories),
    ) {
      (Ok(mut file), Ok(json)) => {
        match file.write_all(json.as_bytes()) {
          Ok(_) => println!("History file updated at {:?}", &history_path),
          Err(e) => unreachable!(e),
        };
      }
      (Err(e), _) => unreachable!(e),
      (_, Err(e)) => unreachable!(e),
    };
  }
}

mod test {
  use super::*;

  #[test]
  fn test_history_path() {
    let p = History::history_path(&Path::new("foo").to_path_buf());
    assert_eq!(p, Path::new("foo/.history.json").to_path_buf());
  }
}
