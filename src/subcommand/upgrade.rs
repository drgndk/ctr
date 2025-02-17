use std::{path::Path, process::Command};

use clap::Args;
use common::{command::{types::ArgumentType, Operation}, console::CONSOLE, struct_gen};
use reqwest::blocking::Client;
use semver::Version;
use std_v2::derive::Command;

use crate::{NAME, VERSION};

#[derive(serde::Deserialize)]
struct CargoPackage {
  version: String
}

#[derive(serde::Deserialize)]
struct CargoToml {
  package: CargoPackage
}

struct_gen! {
  pub struct Options use Args, Command {
    #[arg(short, long)]
    #[help]
    pub let help: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "update";

    fn usage(status: i32) {
      CONSOLE.print_usage::<Self>(vec![ArgumentType::Flags]);
      CONSOLE.print_operation_collection(Self::operations());
      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      match self.fetch_versions() {
        Ok((latest, current)) => {
          if latest > current {
            CONSOLE.print(format!("<brightmagenta>A new version of <italic>{NAME}</italic> is available: <bold><white>{latest}</white></bold></brightmagenta>\n"));
            self.download_update();
          } else {
            CONSOLE.print("<brightmagenta>Nice!</brightmagenta> You are already using the latest version");
          }
        },
        Err(err) => CONSOLE.error(format!("Unable to fetch the latest version: {err}"))
      }

      Ok(())
    }
  }

  mod implementation {
    fn fetch_versions(self: &Self) -> Result<(semver::Version, semver::Version), Box<dyn std::error::Error>> {
      let url = "https://raw.githubusercontent.com/drgndk/ctr/main/Cargo.toml";

      let client = Client::new();
      let response = match client.get(url).send() {
        Ok(response) => response,
        Err(err) => CONSOLE.exit(format!("Failed to fetch the remote Cargo.toml: {err}"))
      };
      let cargo_toml_content = match response.text() {
        Ok(content) => content,
        Err(err) => CONSOLE.exit(format!("Failed to read the remote Cargo.toml: {err}"))
      };

      let cargo_toml: CargoToml = match std_v2::toml::parse::<CargoToml>(&cargo_toml_content) {
        Ok(toml) => toml,
        Err(err) => CONSOLE.exit(format!("Failed to parse the remote Cargo.toml: {err}"))
      };

      if let Ok(latest_version) = Version::parse(&cargo_toml.package.version) {
        if let Ok(current_version) = Version::parse(VERSION) {
          Ok((latest_version, current_version))
        } else {
          CONSOLE.exit("Failed to parse the current version");
        }
      } else {
        CONSOLE.exit("Failed to parse the latest version");
      }
    }

    fn download_update(self: &Self) -> () {

      let current_exe = std::env::current_exe().expect("Failed to get current executable location");
      let repo = Path::new(&current_exe).parent().unwrap().join("../../").canonicalize();

      let repo = match repo {
        Ok(path) => path,
        Err(_) => CONSOLE.exit("Failed to get the repository path")
      };

      let repo_path = match repo.to_str() {
        Some(path) => path,
        None => CONSOLE.exit("Failed to get the repository path")
      };

      let preserve_pwd = match std::env::current_dir() {
        Ok(pwd) => pwd,
        Err(_) => CONSOLE.exit("Failed to get the current working directory")
      };

      if std::env::set_current_dir(repo_path).is_err() {
        CONSOLE.exit("Unable to set pwd to the repository path");
      } else {
        if !repo.join(".git").exists() {
          CONSOLE.exit("Codespace is not a git repository :(");
        }

        #[cfg(not(debug_assertions))] // Don't overwrite the current changes in debug builds
        if let Err(err) = Command::new("git").arg("reset").arg("--hard").status() {
          CONSOLE.exit(format!("{err}"));
        }

        if let Err(err) = Command::new("git").arg("pull").status() {
          CONSOLE.exit(format!("{err}"));
        }

        if let Err(err) = Command::new("fish").arg("-c").arg("./scripts/build.fish").status() {
          if err.kind() == std::io::ErrorKind::NotFound {
            if let Err(err) = Command::new("cargo").arg("b").arg("-r").status() {
              if err.kind() == std::io::ErrorKind::NotFound {
                CONSOLE.exit("cargo not found???");
              }

              CONSOLE.exit("Failed to build ctr");
            }

            CONSOLE.warn("<brightyellow>ctr was built using cargo, but no symlink was created inside <white>/usr/bin</white></brightyellow>");
          } else {
            CONSOLE.exit("Failed to build the project");
          }
        }

        if std::env::set_current_dir(&preserve_pwd).is_err()  {
          CONSOLE.error("Failed to change back to the preserved pwd");
        }

        CONSOLE.print(format!("\n<brightmagenta><bold>Successfully</bold> updated <white>{NAME}</white> to the latest version</brightmagenta>"));
      }
    }
  }
}
