use std::str::FromStr;
use std::result;
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs::{read_dir, File};
use im::{ConsList, HashMap};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

#[derive(Debug)]
pub struct History {
  root: PathBuf,
  histories: HashMap<PathBuf, ConsList<Event>>,
}

// impl Serialize for History {
//   fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
//   where
//     S: Serializer,
//   {
//     let mut s = try!(serializer.serialize_struct("History", 2));
//     s.serialize_field("root", &self.root)?;
//     // let mut map = serializer.serialize_map(Some(self.len()))?;
//     // let mut map = serializer.serialize_map(Some(10))?;
//     // for (k, v) in self {
//     //     map.serialize_entry(k, v)?;
//     // }
//     // map.end()
//     // s.serialize_field("histories", &self.histories)?;
//     // s.end()
//     unimplemented!();
//   }
// }

// impl Deserialize for History {
//   fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
//   where
//     D: Deserializer,
//   {
//     unimplemented!();
//   }
// }

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
            let history_of_file = ConsList::new().cons(Event::Create(i32_of_systemtime(modified)));
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
    // let mut history_path = Path::new(&self.root).to_path_buf();
    // history_path.push(".history.json");
    // match File::create(history_path) {
    //   Ok(mut file) => {
    //     match file.write_all(b"&self.histories") {
    //       Ok(_) => {}
    //       Err(e) => {
    //         println!("{:?}", e);
    //         unreachable!();
    //       }
    //     };
    //   }
    //   Err(e) => {
    //     println!("{:?}", e);
    //     unreachable!();
    //   }
    // };
    // let result = serde_json::from_str::<ConsList<i32>>("");
    let x = serde_json::to_string(&Event::Change(1));
    let y = serde_json::to_string(&ConsList::new());
    // let result = serde_json::from_str::<Event>("");
    unimplemented!();
  }
}
