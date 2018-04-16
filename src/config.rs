use im::*;
use serde_json::{from_slice, to_string_pretty};
use std;
use std::fs::{read_dir, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Clone)]
pub struct WatchTargets2(PathBuf);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatchTargets(pub Set<(PathBuf, PathBuf)>);

impl WatchTargets {
  // TODO: Consider how to avoid warning by using modules either feature=debug or not.
  #[cfg(not(feature = "debug"))]
  fn configuration_path() -> PathBuf {
    std::env::home_dir()
      .and_then(|p| Some(p.join(".sync-dir.conf")))
      .unwrap()
  }

  #[cfg(feature = "debug")]
  fn configuration_path() -> PathBuf {
    std::path::Path::new(".sync-dir.conf").to_owned()
  }

  pub fn new() -> Self {
    let configuration_path = Self::configuration_path();
    match File::open(&configuration_path) {
      Ok(mut file) => {
        let mut buf = Vec::new();
        let _ = file.read_to_end(&mut buf);
        from_slice::<WatchTargets>(&buf).unwrap()
      }
      Err(_) => {
        let instance = WatchTargets(Set::new());
        instance.write(configuration_path);
        instance
      }
    }
  }

  fn write(&self, configuration_path: PathBuf) {
    match (File::create(&configuration_path), to_string_pretty(&self)) {
      (Ok(mut file), Ok(json)) => {
        match file.write_all(json.as_bytes()) {
          Ok(_) => println!(
            "Configuration file has been {} at {:?}",
            if self.0.is_empty() {
              "created"
            } else {
              "updated"
            },
            configuration_path
          ),
          Err(e) => unreachable!(e),
        };
      }
      (e1, e2) => unreachable!(format!("{:?}\n{:?}", e1, e2)),
    };
  }

  fn is_pair_available(pair: (&PathBuf, &PathBuf)) -> bool {
    let (a, b) = pair;
    match (read_dir(&a), read_dir(&b)) {
      (Ok(_), Ok(_)) => true,
      _ => false,
    }
  }

  pub fn get_available_directories(&self) -> Set<(PathBuf, PathBuf)> {
    self
      .0
      .iter()
      .filter(|x| {
        let a = &x.0;
        let b = &x.1;
        Self::is_pair_available((a, b))
      })
      .collect()
  }

  pub fn add(&mut self, pair: (PathBuf, PathBuf)) {
    let (a, b) = pair;
    match (read_dir(&a), read_dir(&b)) {
      (Ok(_), Ok(_)) => {
        self.0 = self.0.insert((a, b));
        let configuration_path = Self::configuration_path();
        self.write(configuration_path);
      }
      (e1, e2) => unreachable!(format!("{:?}\n{:?}", e1, e2)),
    }
  }
}
