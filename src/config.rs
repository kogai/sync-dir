use im::*;
use serde_json::{from_slice, to_string_pretty};
use std::env;
use std::fs::read_dir;
use std::io::Write;
use std::path::PathBuf;
use std::str::from_utf8;
use toml;

#[derive(RustEmbed)]
#[folder("./")]
struct Asset;

#[derive(Debug, Deserialize, Serialize)]
struct PackageConfig {
  name: String,
  version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
  package: PackageConfig,
}

impl Package {
  fn new() -> Self {
    let cargo_toml = Asset::get("Cargo.toml").unwrap();
    toml::from_str::<Package>(from_utf8(&cargo_toml).unwrap()).unwrap()
  }

  pub fn get_config() -> (String, String) {
    let config = Self::new();
    (config.package.name, config.package.version)
  }
}

#[derive(Clone)]
pub struct WatchTargets2(PathBuf);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatchTargets(pub Set<(PathBuf, PathBuf)>);

impl WatchTargets {
  fn configuration_path() -> PathBuf {
    env::home_dir()
      .and_then(|p| Some(p.join(".sync-dir.conf")))
      .unwrap()
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
