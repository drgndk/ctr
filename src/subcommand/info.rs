use clap::Args;
use common::{command::{types::ArgumentType, Operation}, console::CONSOLE, struct_gen};
use sysinfo::System;

use crate::{get_version, NAME};
use std::env::consts::*;

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    pub let help: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "info";

    fn usage(status: i32) {
      CONSOLE.print_usage::<Self>(vec![ArgumentType::Flags]);
      CONSOLE.print_operation_collection(Self::operations());
      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));

      let mut data = vec![];


      CONSOLE.print(format!("<brightmagenta>{NAME} v{}</brightmagenta>", get_version()));
      println!();

      data.push(
        ("os", match System::name() {
          Some(name) => format!("{name} {ARCH}"),
          None => format!("{OS} {ARCH}")
        })
      );

      match FAMILY {
        "unix" => {
          if let Some(kernel_ver) = System::kernel_version() {
            data.push(("kernel", kernel_ver));
          }
        }
        _ => {
          if let Some(os_ver) = System::os_version() {
            data.push(("release", os_ver));
          }
        }
      }

      if let Some(desktop_env) = std::env::var("DESKTOP_SESSION").ok() {
        data.push(("desktop", desktop_env));
      }

      if let Some(window_man) = std::env::var("XDG_CURRENT_DESKTOP").ok() {
        data.push(("wm", window_man));
      }

      if let Some(language) = std::env::var("LANG").ok() {
        data.push(("lang", language));
      }

      let uptime = {
        let uptime_seconds = System::uptime();
        let minutes = (uptime_seconds / 60) % 60;

        let mut uptime_parts = vec![
          (uptime_seconds / 31536000000, 'k'),        // Seriously?! What is wrong with you?
          ((uptime_seconds / 3153600000) % 10, 'c'),  // A whole century? Are you even human?
          ((uptime_seconds / 315360000) % 10, 'd'),   // You are dedicated to your pc, aren't you?
          (uptime_seconds / 29030400, 'y'),           // It's getting rediculous at this point. Poor pc should be taken away from you.
          ((uptime_seconds / 2419200) % 12, 'M'),     // Maybe restart your pc once in a while?
          ((uptime_seconds / 604800) % 4, 'w'),       // do you even know how grass looks like?
          ((uptime_seconds / 86400) % 7, 'd'),
          ((uptime_seconds / 3600) % 24, 'h'),
          (minutes, 'm'),
          (uptime_seconds % 60, 's')
        ].iter()
          .filter_map(|(value, unit)| {
            if *value > 0 {
              Some(format!("{}{}", value, unit))
            } else {
              None
            }
          })
          .collect::<Vec<String>>();

        // only show seconds, if its below a minute.
        // otherwise it clutters the output
        (minutes > 1).then(|| uptime_parts.pop());

        uptime_parts.join(" ")
      };
      data.push(("uptime", uptime));

      let mut sys = System::new_all();
      sys.refresh_all();

      let total_mem = sys.total_memory();
      let used_mem = sys.used_memory();

      data.push(
        ("memory",
          format!("{used_mem} / {total_mem}",
            used_mem = self.format_memory(used_mem),
            total_mem = self.format_memory(total_mem)
          )
        )
      );
      data.push(("cpu", sys.cpus()[0].brand().to_string()));

      if let Ok(shell) = std::env::var("SHELL") {
        data.push(("shell", shell));
      }

      match std::process::Command::new("rustc").arg("--version").output() {
        Ok(rust_version) => data.push(("rustc", String::from_utf8_lossy(&rust_version.stdout).replace('\n', ""))),
        _ => CONSOLE.error("Could not determine rustc version")
      }

      self.format_data(data);

      Ok(())
    }
  }

  mod implementation {
    fn format_memory(self: &Self, bytes: u64) -> String {
      const KIB: u64 = 1024;
      const MIB: u64 = KIB * KIB;
      const GIB: u64 = MIB * KIB;
      const TIB: u64 = GIB * KIB;
      const PIB: u64 = TIB * KIB;
      const EIB: u64 = PIB * KIB;

      match bytes {
        b if b >= EIB => format!("{:.2}EB", b as f64 / EIB as f64),
        b if b >= PIB => format!("{:.2}PB", b as f64 / PIB as f64),
        b if b >= TIB => format!("{:.2}TB", b as f64 / TIB as f64),
        b if b >= GIB => format!("{:.2}GB", b as f64 / GIB as f64),
        b if b >= MIB => format!("{:.2}MB", b as f64 / MIB as f64),
        b if b >= KIB => format!("{:.2}KB", b as f64 / KIB as f64),
        b if b < KIB => format!("{b}B"),
        b => format!("{b}??"),
      }
    }

    fn format_data(self: &Self, data: Vec<(&'static str, impl Into<String>)>) -> () {
      let mut data = data;
      let mut max_key_len = 0;
      data.sort_by(|a, b| {
        max_key_len = max_key_len.max(a.0.len()).max(b.0.len());
        return a.0.cmp(b.0);
      });

      for (key, value) in data {
        let spaces = " ".repeat(max_key_len.saturating_sub(key.len()).saturating_add(4));
        CONSOLE.print(format!("<brightblue>{key}</brightblue>{spaces} {}", value.into()));
      }
    }
  }
}
