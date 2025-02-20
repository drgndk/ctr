use std::{collections::HashMap, sync::LazyLock};
use crate::{command::{types::{ArgumentType, CommandType}, Command, Operation}, string::StringV2, struct_gen, NAME};

const SPACING: usize = 24;

fn operation_name(operation: &Command) -> StringV2 {
  match operation.command_type() {
    CommandType::Flag { short: ' ', name } => StringV2::from(format!("    --{name}")),
    CommandType::Flag { short, name } => StringV2::from(format!("-{short}, --{name}")),
    _ => StringV2::from(format!("{}", operation.command_type().name()))
  }
}

fn colorizer(content: &StringV2, code: &str) -> String {
  if content.is_whitespace() {
    return content.to_string();
  }

  format!("<{code}>{content}</{code}>")
}

struct_gen! {
  pub struct Console {
    pub let strip_ansi: bool = false;
  }

  mod debugging {
    #[cfg(debug_assertions)]
    pub fn debug(self: &Self, message: impl Into<String>) {
      let prefix = self.generate_tag("DEBUG", "black", "brightmagenta");
      eprintln!("{prefix} {}", StringV2::from(message.into()));
    }

    #[cfg(debug_assertions)]
    pub fn suggest(self: &Self, message: impl Into<String>, better_use: Vec<&'static str>) {

      if better_use.is_empty() {
        return self.debug(message);
      }

      self.debug(
        format!("{message}\nConsider using {items} instead.",
          message = message.into(),
          items = {

            match better_use.len() {
              1 => format!("<magenta>{}</magenta>", better_use[0]),
              2 => format!("<magenta>{}</magenta> or <magenta>{}</magenta>", better_use[0], better_use[1]),
              _ => {
                let str_iter = better_use.iter();
                let length = str_iter.len();
                let mut items = str_iter.take(length.saturating_sub(1)).map(|e| format!("<magenta>{e}</magenta>")).collect::<Vec<String>>().join(", ");

                if let Some(last) = better_use.last() {
                  items.push_str(&format!(" or <magenta>{last}</magenta>"));
                }
                items
              }
            }
          }
        )
      );
    }

    #[cfg(debug_assertions)]
    pub fn dir(self: &Self, message: impl std::fmt::Debug) {
      self.debug(format!("{:#?}", message));
    }
  }

  mod implementations {
    pub fn print(self: &Self, message: impl Into<String>) {
      println!("{}", StringV2::from(message.into()));
    }

    pub fn eprint(self: &Self, message: impl Into<String>) {
      eprintln!("{}", StringV2::from(message.into()));
    }

    pub fn assert(self: &Self, condition: bool, message: impl Into<String>) {
      if !condition {
        self.error(message);
        std::process::exit(1);
      }
    }

    fn generate_tag(self: &Self, tag: impl Into<String>, color: impl Into<String>, font: impl Into<String>) -> StringV2 {
      let color_tag = color.into();
      StringV2::from(&format!("<{color_tag}>[</{color_tag}>{}<{color_tag}>]</{color_tag}>", tag.into()))
        .push_effect(font.into()).bold()
        .push_effect(&format!("{color_tag}background"))
    }

    pub fn log(self: &Self, message: impl Into<String>) {
      let prefix = self.generate_tag("LOG", "black", "brightblue");
      println!("{prefix} {}", StringV2::from(message.into()));
    }

    pub fn warn(self: &Self, message: impl Into<String>) {
      let prefix = self.generate_tag("WARN", "black", "brightyellow");
      eprintln!("{prefix} {}", StringV2::from(message.into()));
    }

    pub fn error(self: &Self, message: impl Into<String>) {
      let prefix = self.generate_tag("ERROR", "black", "brightred");
      eprintln!("{prefix} {}", StringV2::from(message.into()));
    }

    pub fn exit(self: &Self, message: impl Into<String>) -> ! {
      self.error(message);
      std::process::exit(1);
    }

    pub fn panic(self: &Self, message: impl Into<String>) -> ! {
      let prefix = self.generate_tag("PANIC", "black", "red");
      eprintln!("{prefix} {}", StringV2::from(message.into()));
      std::process::exit(1);
    }
  }

  mod help_rendering {
    fn print_operation(self: &Self, operation: &Command, sizes: (&str, usize, usize)) {
      let (code, mut spacing, largest_operation_name) = sizes;

      let name = operation_name(&operation);
      let mut formatted = StringV2::from(colorizer(&name, code)).bold();
      let unformatted_len = name.strip_ansi().len();

      if largest_operation_name < unformatted_len {
        spacing = spacing.saturating_sub(unformatted_len - largest_operation_name);
      }

      let additional_spacing = ((largest_operation_name.max(unformatted_len) + 1) as usize).saturating_sub(unformatted_len);


      spacing = operation.example().to_owned().map_or(spacing + additional_spacing, |example| {
        formatted.push_str(&format!("{space}<black bright>{example}</black bright>", space = " ".repeat(additional_spacing)));
        spacing.saturating_sub(example.len())
      });

      let desc = StringV2::from(operation.about().trim());

      if !desc.is_whitespace() {
        let space_between = " ".repeat(spacing);

        let chunks = desc.split('\n');
        let mut chunks = chunks.iter();

        if let Some(desc) = chunks.next() {
          self.print(format!("{formatted}{space_between}{desc}"));
        }

        if chunks.len() >= 1 {
          let spacing_length = formatted.strip_ansi().len();

          while let Some(desc) = chunks.next() {
            self.print(format!("{}{space_between}{desc}", " ".repeat(spacing_length)));
          }

          println!("");
        }
      } else {
        self.print(formatted);
      }
    }

    pub fn print_operation_collection(self: &Self, operations_list: Vec<Vec<Command>>) {
      let mut collections: HashMap<String, Vec<Command>> = HashMap::new();

      for operation in operations_list.into_iter().flat_map(|operations| operations) {
        let cmd_type = operation.command_type().clone();
        collections
          .entry(cmd_type.to_string())
          .or_insert_with(Vec::new)
          .push(operation);
      }

      let mut collections = collections.into_iter().collect::<Vec<(String, Vec<Command>)>>();
      if collections.len() > 0 {
        let mut spacing = SPACING;
        let mut largest_operation = 0;

        collections.sort_by(|(a, _), (b, _)| a.cmp(&b));

        for (header, operations) in collections.iter_mut() {
          if operations.is_empty() {
            continue;
          }

          match CommandType::from(&header.clone()) {
            CommandType::Other { .. } => {},
            _ => {
              if operations.len() > 1 {
                header.push_str("S");
              }
            },
          };

          *header = header.to_uppercase();

          operations.sort_by(|a, b| a.command_type().name().cmp(&b.command_type().name()));
          for operation in operations.iter() {
            let op_name = operation_name(&operation).strip_ansi();
            let op_len = op_name.len();
            let mut example_len = SPACING.saturating_sub(op_len);

            if let Some(example) = operation.example() {
              largest_operation = largest_operation.max(op_len);
              example_len = example.trim().len();
            }

            let space = format!("{}{}", op_name, " ".repeat(example_len)).len();
            spacing = spacing.max(space);
          }
        }

        for (header, operations) in collections {
          if !operations.is_empty() {
            println!("");
            self.print(format!("<bold>{header}</bold>"));
            for operation in operations {
              self.print_operation(&operation, (operation.command_type().to_color(), spacing, largest_operation));
            }
          }
        }
      }
    }

    pub fn print_usage<Command: Operation>(self: &Self, argument_types: Vec<ArgumentType>) {
      let command_name = Command::NAME.to_lowercase();
      let usage_command = {
        let mut name = NAME.to_lowercase();

        if command_name == name {
          None
        } else {
          if let Some(parent) = Command::PARENT {
            name.push_str(&format!(" {parent}"));
          }

          Some(name)
        }
      };

      let command = {
        let mut args_v2 = StringV2::new();
        argument_types.iter().for_each(|argument_type| {
          args_v2.push_str(format!("{} ", argument_type.to_str_v2()));
        });

        if usage_command.is_some() {
          StringV2::from(format!("{} {}", command_name, args_v2))
        } else {
          args_v2
        }
      };

      let usage = usage_command.map_or(command_name, |e| format!("<brightblack>{e}</brightblack>"));
      self.print(format!("<bold>USAGE</bold>\n{usage} {command}"));
    }
  }
}

pub static CONSOLE: LazyLock<Console> = LazyLock::new(|| Console::new());
