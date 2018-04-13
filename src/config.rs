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
pub struct Config {
  package: PackageConfig,
}

impl Config {
  fn new() -> Self {
    let cargo_toml = Asset::get("Cargo.toml").unwrap();
    toml::from_str::<Config>(from_utf8(&cargo_toml).unwrap()).unwrap()
  }

  pub fn get_config() -> (String, String) {
    let config = Self::new();
    (config.package.name, config.package.version)
  }
}
