pub use clap_complete_command::Shell;

pub mod consts {
  use std::{path::PathBuf, sync::LazyLock};

  use super::Shell;
  use crate::console::CONSOLE;

  // Binary name
  pub const BINARY_NAME: &str = "ctr";

  pub static NO_COLOR: LazyLock<bool> = LazyLock::new(|| std::env::var("NO_COLOR").is_ok());

  pub static SHELL: LazyLock<Shell> = LazyLock::new(|| {
    if let Ok(shell) = std::env::var("SHELL") {
      if let Some(shell) = shell.split('/').next_back() {
        return match shell {
          "bash" => Shell::Bash,
          "zsh" => Shell::Zsh,
          "fish" => Shell::Fish,
          "nushell" => Shell::Nu,
          "powershell" => Shell::PowerShell,
          "elvish" => Shell::Elvish,
          _ => {
            CONSOLE.warn("Unknown shell detected. Falling back to Bash.");
            Shell::Bash
          },
        }
      }
    }

    CONSOLE.warn("No <yellow>$SHELL</yellow> variable found. <yellow>SHELL</yellow> might be incorrect.");

    // Fallbacks if $SHELL is not set
    match std::env::consts::FAMILY {
      "Windows" => Shell::PowerShell,
      _ => Shell::Bash,
    }
  });

  pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    match dirs::home_dir() {
      Some(home_dir) => home_dir,
      None => CONSOLE.panic("Unable to get user home directory"),
    }
  });
  pub static CONFIG: LazyLock<PathBuf> = LazyLock::new(|| {
    match dirs::config_dir() {
      Some(home_dir) => home_dir,
      None => CONSOLE.panic("Unable to get user config directory"),
    }
  });
  pub static INSTALL_DIR: LazyLock<PathBuf> = LazyLock::new(|| HOME.join(format!(".{BINARY_NAME}")));
  pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG.join(BINARY_NAME));
  pub static REPO_DIR: LazyLock<PathBuf> = LazyLock::new(|| INSTALL_DIR.join("src"));
  pub const IS_DEBUG: bool = cfg!(debug_assertions);
}
