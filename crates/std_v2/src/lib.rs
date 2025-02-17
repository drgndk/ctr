use std::{path::PathBuf, sync::LazyLock};


pub use common::*;
use console::CONSOLE;

pub use operation_derive as derive;

pub mod toml;

pub const CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
  if let Some(config_dir) = dirs::config_dir() {
    return config_dir.join("ctr");
  }

  CONSOLE.panic("Could not find config directory");
});
