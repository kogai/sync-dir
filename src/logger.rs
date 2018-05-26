use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use std::env::current_dir;
use std::path::PathBuf;
use std::fs::read_to_string;

pub struct AppLogger;
impl AppLogger {
  #[cfg(not(debug_assertions))]
  fn log_directory() -> PathBuf {
    PathBuf::from("/var/log")
  }

  #[cfg(debug_assertions)]
  fn log_directory() -> PathBuf {
    current_dir().and_then(|p| Ok(p.join("log"))).unwrap()
  }

  fn get_log_file() -> PathBuf {
    AppLogger::log_directory().join("sync-dir.log")
  }

  pub fn init() {
    let log_file = AppLogger::get_log_file();
    let stdout = ConsoleAppender::builder().build();

    let fs_log = FileAppender::builder()
      .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
      .build(log_file)
      .unwrap();

    let config = Config::builder()
      .appender(Appender::builder().build("stdout", Box::new(stdout)))
      .appender(Appender::builder().build("fs", Box::new(fs_log)))
      .logger(
        Logger::builder()
          .appender("fs")
          .additive(false)
          .build("app::fs", LevelFilter::Warn),
      )
      .build(
        Root::builder()
          .appenders(vec!["stdout", "fs"])
          .build(LevelFilter::Info),
      )
      .unwrap();

    let _ = log4rs::init_config(config).unwrap();
  }

  pub fn show_log() -> String {
    let log_file = AppLogger::get_log_file();
    read_to_string(log_file).expect("Log file did not exists")
  }
}

#[macro_export]
macro_rules! exit_with_log {
  ($($arg: tt)*) => {
    {
      use log::Level;
      log!(Level::Warn, $($arg)*);
      unreachable!();
    }
  };
}
