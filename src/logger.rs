use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use std::fs::read_to_string;
use std::path::PathBuf;

pub struct AppLogger;
impl AppLogger {
  #[cfg(not(debug_assertions))]
  fn log_directory() -> PathBuf {
    PathBuf::from("/var/log")
  }

  #[cfg(debug_assertions)]
  fn log_directory() -> PathBuf {
    use std::env::current_dir;
    current_dir().and_then(|p| Ok(p.join("log"))).unwrap()
  }

  #[cfg(not(debug_assertions))]
  fn syslog_appender() -> Box<log4rs::append::Append> {
    use log;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs_syslog;
    let appender = log4rs_syslog::SyslogAppender::builder()
      .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
      .openlog(
        "sync-dir.log",
        log4rs_syslog::LogOption::LOG_PID,
        log4rs_syslog::Facility::Daemon,
      )
      .build();
    Box::new(appender)
  }

  #[cfg(debug_assertions)]
  fn syslog_appender() -> Box<log4rs::append::Append> {
    use std::env::current_dir;

    let appender = FileAppender::builder()
      .build(
        current_dir()
          .and_then(|p| Ok(p.join("log").join("sync-dir.log")))
          .unwrap(),
      )
      .unwrap();
    Box::new(appender)
  }

  fn get_log_file() -> PathBuf {
    AppLogger::log_directory().join("sync-dir.log")
  }

  pub fn init() {
    let stdout = ConsoleAppender::builder().build();
    let syslog = AppLogger::syslog_appender();
    let config = Config::builder()
      .appender(Appender::builder().build("stdout", Box::new(stdout)))
      .appender(Appender::builder().build("syslog", syslog))
      .logger(
        Logger::builder()
          .appender("syslog")
          .additive(false)
          .build("app::syslog", LevelFilter::Warn),
      )
      .build(
        Root::builder()
          .appenders(vec!["stdout", "syslog"])
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
