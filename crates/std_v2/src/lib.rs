use std::{path::PathBuf, sync::LazyLock};

pub use common::*;
use console::CONSOLE;

pub use operation_derive as derive;

pub mod toml;

pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| {
  match dirs::home_dir() {
    Some(home_dir) => home_dir,
    None => CONSOLE.panic("Unable to get user home directory")
  }
});

pub static INSTALL_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
  HOME.join(format!(".{NAME}"))
});

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
  HOME.join(format!(".config/{NAME}"))
});

pub static REPO_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
  INSTALL_DIR.join("src")
});

pub const IS_DEBUG: bool = cfg!(debug_assertions);
