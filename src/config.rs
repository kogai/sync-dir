use im::*;
use serde_json::{from_slice, to_string_pretty};
use std::env;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration(ConsList<(PathBuf, PathBuf)>);

impl Configuration {
  pub fn new() -> Self {
    let configuration_path = env::home_dir()
      .and_then(|p| Some(p.join(".sync-dir.conf")))
      .unwrap();

    match File::open(&configuration_path) {
      Ok(mut file) => {
        let mut buf = Vec::new();
        let _ = file.read_to_end(&mut buf);
        from_slice::<Configuration>(&buf).unwrap()
      }
      Err(_) => {
        let instance = Configuration(ConsList::new());
        match (
          File::create(&configuration_path),
          to_string_pretty(&instance),
        ) {
          (Ok(mut file), Ok(json)) => {
            match file.write_all(json.as_bytes()) {
              Ok(_) => println!(
                "Configuration file has been created at {:?}",
                &configuration_path
              ),
              Err(e) => unreachable!(e),
            };
          }
          (e1, e2) => unreachable!(format!("{:?}\n{:?}", e1, e2)),
        };
        instance
      }
    }
  }
}
