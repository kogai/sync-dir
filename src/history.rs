use std::str::FromStr;
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs::{read_dir, File};
use im::{ConsList, HashMap};
use regex::Regex;
use serde_json;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
  Create(i32),
  Change(i32),
  Delete(i32),
}

pub enum Dawn {
  PreHistory,
  HasHistory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
  root: PathBuf,
  histories: HashMap<PathBuf, ConsList<Event>>,
}

impl History {
  pub fn new(root: PathBuf) -> Self {
    let histories = match File::open(&root) {
      // TODO: Handle pattern when already exist History
      Ok(_) => History::generate_history(root.clone(), None, &Dawn::HasHistory),
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
            let history_of_file = ConsList::new().cons(Event::Create(i32_of_systemtime(modified)));
            if file_type.is_dir() {
              Ok(acc.union(&History::generate_history(
                key_with_root,
                Some(root_path.clone()),
                has_history,
              )))
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
    let mut history_path = Path::new(&self.root).to_path_buf();
    history_path.push(".history.json");
    match (
      File::create(&history_path),
      serde_json::to_string_pretty(&self.histories),
    ) {
      (Ok(mut file), Ok(json)) => {
        match file.write_all(json.as_bytes()) {
          Ok(_) => println!("History file generated at {:?}", &history_path),
          Err(e) => unreachable!(e),
        };
      }
      (Err(e), _) => unreachable!(e),
      (_, Err(e)) => unreachable!(e),
    };
  }
}
