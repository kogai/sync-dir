use std::result;
use std::io::{Result, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use std::fs::{read_dir, File};
use im::{ConsList, HashMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// #[derive(Debug, Serialize, Deserialize)]
#[derive(Debug)]
pub enum Event {
  Create(SystemTime),
  Change(SystemTime),
  Delete(SystemTime),
}

impl Serialize for Event {
  fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    unimplemented!();
  }
}

impl Deserialize for Event {
  fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
  where
    D: Deserializer,
  {
    unimplemented!();
  }
}

#[derive(Debug)]
pub struct History {
  root: PathBuf,
  histories: HashMap<PathBuf, ConsList<Event>>,
}

impl Serialize for History {
  fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    unimplemented!();
  }
}

impl Deserialize for History {
  fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
  where
    D: Deserializer,
  {
    unimplemented!();
  }
}

impl History {
  pub fn new(root: PathBuf) -> Self {
    // TODO: Handle pattern when already exist History
    let histories = History::generate_history(root.clone(), None);
    let instance = History { root, histories };
    instance.write();
    instance
  }

  fn generate_history(
    root_path: PathBuf,
    current_path: Option<PathBuf>,
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
            let history_of_file = ConsList::new().cons(Event::Create(modified));
            if file_type.is_dir() {
              Ok(acc.union(&History::generate_history(
                key_with_root,
                Some(root_path.clone()),
              )))
            } else {
              Ok(acc.insert(key, history_of_file))
            }
          },
        )
        .unwrap(),
      Err(err) => unreachable!(err),
    }
  }

  fn write(&self) {
    match File::create(&self.root) {
      Ok(mut file) => {
        match file.write_all(b"&self.histories") {
          Ok(_) => {}
          Err(e) => {
            println!("{:?}", e);
            unreachable!();
          }
        };
      }
      Err(e) => {
        println!("{:?}", e);
        unreachable!();
      }
    };
    unimplemented!();
  }
}
