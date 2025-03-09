use std::{path::Path, thread::sleep};

use clap::Args;
use std_v2::{
  command::{types::ArgumentType, Operation},
  console::CONSOLE,
  struct_gen,
};

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    let help: bool = false;

    #[arg(short, long), flag("Urgency level: 0 (Low), 1 (Normal), 2 (Critical)", example = "1")]
    let level: Option<u8> = Some(1);

    #[arg(short, long), flag("Notification delay in milliseconds", example = "0")]
    let delay: Option<u64> = None;

    #[arg(short, long), flag("Path to notification icon", example = "/usr/share/icons/icon.png")]
    let icon: Option<String> = None;

    #[arg(short, long), flag("Notification title", example = "ctr")]
    let title: Option<String> = Some("ctr".to_string());

    #[variadic(name = "message", "Notification message")]
    let args: Vec<String> = vec![];
  }

  impl Operation {
    const NAME: &'static str = "notify";

    fn main(self: &Self) -> std::io::Result<()> {
      self.help.then(|| Self::usage(0));

      let mut notification = notify_rust::Notification::new();
      notification
        .summary(&self.title.clone().map_or("ctr".to_string(), |e| {
          if e.trim().is_empty() {
            "ctr".to_string()
          } else {
            e
          }
        }))
        .body(&self.args.join(" "));

      if let Some(level) = self.level {
        notification.urgency(match level {
          0 => notify_rust::Urgency::Low,
          2 => notify_rust::Urgency::Critical,
          _ => notify_rust::Urgency::Normal,
        });
      }

      if let Some(icon) = &self.icon {
        if Path::new(icon).exists() {
          notification.icon(icon);
        } else {
          CONSOLE.warn(format!("Icon file not found: {icon}"));
        }
      }

      if let Some(delay) = self.delay {
        if delay > 0 {
          sleep(std::time::Duration::from_millis(delay));
        }
      }

      if let Err(e) = notification.show() {
        CONSOLE.exit(format!("{e}"));
      }

      Ok(())
    }

    fn validate(self: &Self) -> std::io::Result<()> {

      if notify_rust::get_capabilities().is_err() {
        CONSOLE.exit("Unable to display notifications. Please ensure that your system supports notifications.");
      }

      if !self.help {
        if self.args.is_empty() {
          CONSOLE.exit("No message provided");
        }

        if let Some(level) = self.level {
          if level > 2 {
            CONSOLE.exit("Invalid urgency level");
          }
        }
      }

      Ok(())
    }
  }
}
