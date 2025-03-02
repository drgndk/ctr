use crate::{enum_gen, env::consts::NO_COLOR, struct_gen};

struct_gen! {
  pub struct EffectSettings use PartialEq, PartialOrd, Eq, Ord, Clone {
    let bright: bool = false;
    let bg: bool = false;
  }

  mod presets {
    pub fn with_bright() -> Self {
      Self { bright: true, bg: false }
    }

    pub fn with_background(bright: bool) -> Self {
      Self { bright, bg: true }
    }
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
      if *NO_COLOR {
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
        .saturating_add(if settings.bg { 10 } else { 0 })
        .saturating_add(if settings.bright { 60 } else { 0 })
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

        Self::BrightBlack => Self::return_base(Self::Black, &EffectSettings::with_bright()),
        Self::BrightRed => Self::return_base(Self::Red, &EffectSettings::with_bright()),
        Self::BrightGreen => Self::return_base(Self::Green, &EffectSettings::with_bright()),
        Self::BrightYellow => Self::return_base(Self::Yellow, &EffectSettings::with_bright()),
        Self::BrightBlue => Self::return_base(Self::Blue, &EffectSettings::with_bright()),
        Self::BrightMagenta => Self::return_base(Self::Magenta, &EffectSettings::with_bright()),
        Self::BrightCyan => Self::return_base(Self::Cyan, &EffectSettings::with_bright()),
        Self::BrightWhite => Self::return_base(Self::White, &EffectSettings::with_bright()),

        Self::BlackBackground => Self::return_base(Self::Black, &EffectSettings::with_background(false)),
        Self::RedBackground => Self::return_base(Self::Red, &EffectSettings::with_background(false)),
        Self::GreenBackground => Self::return_base(Self::Green, &EffectSettings::with_background(false)),
        Self::YellowBackground => Self::return_base(Self::Yellow, &EffectSettings::with_background(false)),
        Self::BlueBackground => Self::return_base(Self::Blue, &EffectSettings::with_background(false)),
        Self::MagentaBackground => Self::return_base(Self::Magenta, &EffectSettings::with_background(false)),
        Self::CyanBackground => Self::return_base(Self::Cyan, &EffectSettings::with_background(false)),
        Self::WhiteBackground => Self::return_base(Self::White, &EffectSettings::with_background(false)),

        Self::BrightBlackBackground => Self::return_base(Self::Black, &EffectSettings::with_background(true)),
        Self::BrightRedBackground => Self::return_base(Self::Red, &EffectSettings::with_background(true)),
        Self::BrightGreenBackground => Self::return_base(Self::Green, &EffectSettings::with_background(true)),
        Self::BrightYellowBackground => Self::return_base(Self::Yellow, &EffectSettings::with_background(true)),
        Self::BrightBlueBackground => Self::return_base(Self::Blue, &EffectSettings::with_background(true)),
        Self::BrightMagentaBackground => Self::return_base(Self::Magenta, &EffectSettings::with_background(true)),
        Self::BrightCyanBackground => Self::return_base(Self::Cyan, &EffectSettings::with_background(true)),
        Self::BrightWhiteBackground => Self::return_base(Self::White, &EffectSettings::with_background(true)),
      }
    }

    pub fn to_ansi(self: &Self) -> String {
      format!("\x1b[{value}m", value = self.get_ansi_value())
    }
  }
}
