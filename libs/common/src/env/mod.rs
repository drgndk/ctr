pub use clap_complete_command::Shell;

pub mod consts {
  use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
  };

  use super::Shell;
  use crate::{console::CONSOLE, lazy_var, option_var};

  pub static UTF_QUESTIONMARK: &str = "\u{FFFD}";

  // Binary name
  pub const BINARY_NAME: &str = "ctr";

  lazy_var!(pub NO_COLOR<bool> {
    option_var!(NO_COLOR).map_or(false, |v| v.eq("1"))
  });

  lazy_var!(pub SHELL<Shell> {
    if let Some(shell) = option_var!(SHELL) {
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

  lazy_var!(pub HOME<PathBuf> {
    // Respect $HOME if set
    if let Some(home) = option_var!(HOME) {
      return Path::new(&home).to_path_buf();
    }

    match dirs::home_dir() {
      Some(home_dir) => home_dir,
      None => CONSOLE.panic("Unable to get user home directory"),
    }
  });

  lazy_var!(pub USER_CONFIG_DIR<PathBuf> {
    match dirs::config_dir() {
      Some(home_dir) => home_dir,
      None => CONSOLE.panic("Unable to get user config directory"),
    }
  });

  lazy_var!(pub CTR_CONFIG_DIR<PathBuf> {
    USER_CONFIG_DIR.join(BINARY_NAME)
  });

  lazy_var!(pub INSTALL_DIR<PathBuf> {
    HOME.join(format!(".{BINARY_NAME}"))
  });

  lazy_var!(pub REPO_DIR<PathBuf> {
    INSTALL_DIR.join("src")
  });

  pub const IS_DEBUG: bool = cfg!(debug_assertions);
}
