use crate::{struct_gen, enum_gen};

struct_gen! {
  pub struct EffectSettings use PartialEq, PartialOrd, Eq, Ord, Clone {
    let bright: bool = false;
    let bg: bool = false;
  }
}

struct_gen! {
  pub struct EffectArray use PartialEq, PartialOrd, Eq, Ord, Clone {
    let effects: Vec<Effect> = Vec::new();
  }

  impl From<Vec<Effect>> {
    fn from(effects: Vec<Effect>) -> Self {
      Self { effects }
    }
  }

  mod vec_functions {
    pub fn push(self: &mut Self, effect: Effect) {
      // If the NO_COLOR environment variable is set, don't add any effects.
      if std::env::var("NO_COLOR").is_ok() {
        return;
      }

      // If the current effect is the same as the last effect, don't add it.
      if let Some(last_effect) = self.effects.last() {
        if last_effect == &effect {
          return;
        }
      }

      self.effects.push(effect);
    }

    pub fn extend(self: &mut Self, effects: Vec<Effect>) {
      self.effects.extend(effects);
    }

    pub fn clear(self: &mut Self) {
      self.effects.clear();
    }
  }
}

enum_gen! {
  pub enum Effect use PartialEq, PartialOrd, Eq, Ord, Clone {
    Reset,
    Bold,
    Italic,
    Underline,
    Blink,
    Inverse,
    Hidden,
    Strikethrough,

    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    BlackBackground,
    RedBackground,
    GreenBackground,
    YellowBackground,
    BlueBackground,
    MagentaBackground,
    CyanBackground,
    WhiteBackground,

    BrightBlackBackground,
    BrightRedBackground,
    BrightGreenBackground,
    BrightYellowBackground,
    BrightBlueBackground,
    BrightMagentaBackground,
    BrightCyanBackground,
    BrightWhiteBackground,
  }

  impl Into<String> {
    fn into(self: Self) -> String {
      self.to_ansi()
    }
  }

  mod utils {
    fn return_base(base: Effect, settings: &EffectSettings) -> u8 {
      base.get_ansi_value()
        .saturating_add(settings.bg.then(|| 10).unwrap_or(0))
        .saturating_add(settings.bright.then(|| 60).unwrap_or(0))
    }

    pub fn get_ansi_value(self: &Self) -> u8 {
      match self {
        Self::Reset => 0,
        Self::Bold => 1,
        Self::Italic => 3,
        Self::Underline => 4,
        Self::Blink => 5,
        Self::Inverse => 7,
        Self::Hidden => 8,
        Self::Strikethrough => 9,

        Self::Black => 30,
        Self::Red => 31,
        Self::Green => 32,
        Self::Yellow => 33,
        Self::Blue => 34,
        Self::Magenta => 35,
        Self::Cyan => 36,
        Self::White => 37,

        Self::BrightBlack => Self::return_base(Self::Black, &EffectSettings::generate(true, false)),
        Self::BrightRed => Self::return_base(Self::Red, &EffectSettings::generate(true, false)),
        Self::BrightGreen => Self::return_base(Self::Green, &EffectSettings::generate(true, false)),
        Self::BrightYellow => Self::return_base(Self::Yellow, &EffectSettings::generate(true, false)),
        Self::BrightBlue => Self::return_base(Self::Blue, &EffectSettings::generate(true, false)),
        Self::BrightMagenta => Self::return_base(Self::Magenta, &EffectSettings::generate(true, false)),
        Self::BrightCyan => Self::return_base(Self::Cyan, &EffectSettings::generate(true, false)),
        Self::BrightWhite => Self::return_base(Self::White, &EffectSettings::generate(true, false)),

        Self::BlackBackground => Self::return_base(Self::Black, &EffectSettings::generate(false, true)),
        Self::RedBackground => Self::return_base(Self::Red, &EffectSettings::generate(false, true)),
        Self::GreenBackground => Self::return_base(Self::Green, &EffectSettings::generate(false, true)),
        Self::YellowBackground => Self::return_base(Self::Yellow, &EffectSettings::generate(false, true)),
        Self::BlueBackground => Self::return_base(Self::Blue, &EffectSettings::generate(false, true)),
        Self::MagentaBackground => Self::return_base(Self::Magenta, &EffectSettings::generate(false, true)),
        Self::CyanBackground => Self::return_base(Self::Cyan, &EffectSettings::generate(false, true)),
        Self::WhiteBackground => Self::return_base(Self::White, &EffectSettings::generate(false, true)),

        Self::BrightBlackBackground => Self::return_base(Self::Black, &EffectSettings::generate(true, true)),
        Self::BrightRedBackground => Self::return_base(Self::Red, &EffectSettings::generate(true, true)),
        Self::BrightGreenBackground => Self::return_base(Self::Green, &EffectSettings::generate(true, true)),
        Self::BrightYellowBackground => Self::return_base(Self::Yellow, &EffectSettings::generate(true, true)),
        Self::BrightBlueBackground => Self::return_base(Self::Blue, &EffectSettings::generate(true, true)),
        Self::BrightMagentaBackground => Self::return_base(Self::Magenta, &EffectSettings::generate(true, true)),
        Self::BrightCyanBackground => Self::return_base(Self::Cyan, &EffectSettings::generate(true, true)),
        Self::BrightWhiteBackground => Self::return_base(Self::White, &EffectSettings::generate(true, true)),
      }
    }

    pub fn to_ansi(self: &Self) -> String {
      format!("\x1b[{value}m", value = self.get_ansi_value())
    }
  }
}
