use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

pub fn init() {
    let mut builder = Builder::from_default_env();
    
    builder
        .target(Target::Stdout)
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .init();

}