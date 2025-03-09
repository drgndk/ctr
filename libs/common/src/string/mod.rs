pub mod ansi;
pub mod buffer;
use std::{
  fmt::{Display, Formatter},
  slice::{Iter, IterMut},
  string::FromUtf8Error,
  vec::IntoIter,
};

use ansi::{Effect, EffectArray};
use buffer::Buffer;
use quick_xml::{events::Event, Reader};

use crate::{console::CONSOLE, enum_gen, env::consts::NO_COLOR, struct_gen};
enum_gen! {
  enum Tags {
    PreventsPanic
  }

  mod implementations {
    pub fn to_ansi(self: &Self) -> Option<StringV2> {
      // They should only be used in debug modes to prevent misuse/unwanted behavior.
      match self {
        #[cfg(debug_assertions)]
        Self::PreventsPanic => Some(StringV2::from("<redbackground><red>[</red><bold>THIS MESSAGE PREVENTS A PANIC!</bold><red>]</red></redbackground>").render()),

        #[allow(unreachable_patterns)]
        _ => None
      }
    }
  }
}
struct_gen! {
  pub struct StringV2 use PartialEq, PartialOrd, Eq, Ord {
    pub(super) let &mut buffer: Buffer = Buffer::default();
    let global_effects: EffectArray = EffectArray::default();
  }

  impl PartialEq<String> {
    fn eq(self: &Self, other: &String) -> bool {
      self.to_string() == *other
    }
  }

  impl PartialEq<str> {
    fn eq(self: &Self, other: &str) -> bool {
      self.to_string().as_str() == other
    }
  }

  impl From<char> {
    fn from(ch: char) -> Self {
      Self {
        buffer: Buffer::from(ch),
        global_effects: EffectArray::default(),
      }
    }
  }

  impl From<Vec<u8>> {
    fn from(bytes: Vec<u8>) -> Self {
      Self::from(Buffer::from(bytes))
    }
  }

  impl From<&[u8]> {
    fn from(byte_slice: &[u8]) -> Self {
      Self::from(byte_slice.to_vec())
    }
  }

  impl From<Buffer> {
    fn from(buffer: Buffer) -> Self {
      Self {
        buffer,
        global_effects: EffectArray::default(),
      }
    }
  }

  impl From<&Buffer> {
    fn from(buffer: &Buffer) -> Self {
      Self::from(buffer.clone())
    }
  }

  impl From<&str> {
    fn from(s: &str) -> Self {
      if s.is_empty() {
        return Self::default();
      }

      Self::from(s.as_bytes().to_vec())
    }
  }

  impl From<String> {
    fn from(s: String) -> Self {
      if s.is_empty() {
        return Self::default();
      }

      Self::from(s.as_bytes().to_vec())
    }
  }

  impl From<&String> {
    fn from(s: &String) -> Self {
      if s.is_empty() {
        return Self::default();
      }

      Self::from(s.as_bytes().to_vec())
    }
  }

  impl Into<String> {
    fn into(self: Self) -> String {
      self.to_string()
    }
  }

  impl Display {
    fn fmt(self: &Self, f: &mut Formatter) -> std::fmt::Result {
      write!(f, "{}", self.render().to_string())
    }
  }

  impl Clone {
    fn clone(self: &StringV2) -> Self {
      Self {
        buffer: self.buffer.clone(),
        global_effects: self.global_effects.clone(),
      }
    }

    fn clone_from(self: &mut StringV2, source: &StringV2) {
      self.buffer.clone_from(&source.buffer);
    }
  }

  mod constructor {
    pub fn with_capacity(capacity: usize) -> Self {
      Self {
        buffer: Buffer::with_capacity(capacity),
        global_effects: EffectArray::default(),
      }
    }

    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
      match String::from_utf8(vec.clone()) {
        Ok(..) => Ok(Self {
          buffer: Buffer::from(vec),
          global_effects: EffectArray::default(),
        }),
        Err(error) => Err(error),
      }
    }

    pub fn to_owned(self: &Self) -> Self {
      self.clone()
    }
  }

  mod buffer_compatibility {
    pub fn len(self: &Self) -> usize {
      self.buffer.size()
    }

    pub fn push(self: &mut Self, byte: impl Into<char>) {
      if let Err(err) = self.buffer_mut().push_safe(byte.into() as u8) {
        CONSOLE.panic(format!("{err}"))
      }
    }

    pub fn push_slice(self: &mut Self, slice: &[u8]) {
      for byte in slice {
        if let Err(err) = self.buffer_mut().push_safe(*byte) {
          CONSOLE.panic(format!("{err}"))
        }
      }
    }

    pub fn iter(self: &Self) -> Iter<u8> {
      self.buffer.iter()
    }

    pub fn iter_mut(self: &mut Self) -> IterMut<u8> {
      self.buffer.iter_mut()
    }

    pub fn is_empty(self: &Self) -> bool {
      self.buffer.is_empty()
    }

    pub fn clear(self: &mut Self) -> &mut Self {
      self.buffer_mut().clear();
      self
    }

    pub fn bytes(self: &Self) -> &Vec<u8> {
      self.buffer.bytes()
    }

    pub fn bytes_mut(self: &mut Self) -> &mut Vec<u8> {
      self.buffer.bytes_mut()
    }
  }

  mod bytes {
    pub fn is_whitespace(self: &Self) -> bool {
      self.iter().all(|&b| b.is_ascii_whitespace())
    }
  }

  mod conversions {
    pub fn to_string(self: &Self) -> String {
      String::from_utf8_lossy(self.bytes().as_slice()).to_string()
    }

    pub fn chars(self: &Self) -> IntoIter<char> {
      let string = String::from_utf8(self.bytes().clone())
        .expect("Invalid UTF-8 sequence");

      string.chars().collect::<Vec<char>>().into_iter()
    }
  }

  mod ansi_styles {
    #[doc = "Reset all styles and effects."]
    pub fn reset(self: &Self) -> Self {
      Self {
        buffer: self.buffer.clone(),
        global_effects: EffectArray::default(),
      }
    }

    #[doc = "Sets a global effect: \n - `Effect::Bold`"]
    pub fn bold(self: &mut Self) -> Self {
      self.push_effect(Effect::Bold)
    }

    #[doc = "Sets a global effect: \n - `Effect::Italic`"]
    pub fn italic(self: &mut Self) -> Self {
      self.push_effect(Effect::Italic)
    }

    #[doc = "Sets a global effect: \n - `Effect::Underline`"]
    pub fn underline(self: &mut Self) -> Self {
      self.push_effect(Effect::Underline)
    }

    #[doc = "Sets a global effect: \n - `Effect::Blink`"]
    pub fn blink(self: &mut Self) -> Self {
      self.push_effect(Effect::Blink)
    }

    #[doc = "Sets a global effect: \n - `Effect::Inverse`"]
    pub fn inverse(self: &mut Self) -> Self {
      self.push_effect(Effect::Inverse)
    }

    #[doc = "Sets a global effect: \n - `Effect::Hidden`"]
    pub fn hidden(self: &mut Self) -> Self {
      self.push_effect(Effect::Hidden)
    }

    #[doc = "Sets a global effect: \n - `Effect::Strikethrough`"]
    pub fn strike(self: &mut Self) -> Self {
      self.push_effect(Effect::Strikethrough)
    }

    #[doc = "Pushes a new global effect."]
    pub fn push_effect(self: &mut Self, effect: impl Into<Effect>) -> Self {
      self.global_effects.push(effect.into());
      self.clone()
    }
  }

  mod ansi_implementations {
    fn parse_xml(self: &Self, strip_styling: bool) -> Self {
      let mut result = Self::default();

      if self.bytes().is_empty() {
        return result;
      }

      let txt = self.to_string();
      let mut xml_reader = Reader::from_str(&txt);
      let mut buf = Vec::new();

      let mut active_styles = self.global_effects().clone();

      if strip_styling {
        result.push_str(active_styles.to_ansi());
      }

      while let Ok(event) = xml_reader.read_event_into(&mut buf) {
        match event {
          Event::Eof => break,
          Event::Start(e) => {
            if !strip_styling {
              continue;
            }

            let tag_name = StringV2::from(e.name().as_ref());
            match Effect::try_from(tag_name.clone()) {
              Some(effect) => {
                active_styles.push(effect.clone());
                result.push_str(active_styles.to_ansi());
              },
              None => {
                if tag_name.len() == 2 && tag_name.starts_with('h') {
                  if let Some(level) = tag_name.chars().nth(1) {
                    if level.is_digit(10) && (1..=6).contains(&(level.to_digit(10).unwrap_or_else(|| CONSOLE.panic("Failed to convert level to digit. Ensure the level is between 1 and 6.")))) {
                      active_styles.push(Effect::MagentaBackground);
                      result.push_str(active_styles.to_ansi());
                      result.push_str(" # ");
                    }
                  }
                } else {
                  result.push_str(format!("<{tag_name}>"));
                }
              }
            }
          },
          Event::Text(text) => {
            let mut text = match text.unescape() {
              Ok(text) => text.into_owned(),
              _ => String::from_utf8_lossy(text.as_ref()).into_owned()
            };

            // Split the text into words and check for $variables
            let words: Vec<String> = text.split_whitespace().map(String::from).collect();
            for word in words {
              if word.starts_with('$') {
                let variable = &word[1..];
                if let Some(preset_tag) = Tags::try_from(variable) {
                  match preset_tag.to_ansi() {
                    Some(ansi) => text = text.replace(&format!("${}", variable), &ansi.to_string()),
                    None => {}
                  }
                }
              }
            }

            {
              let mut result = String::new();
              let mut start = 0;
              while let Some(start_idx) = text[start..].find("==") {
                result.push_str(&text[start..start + start_idx]);
                start += start_idx + 2;
                if let Some(end_idx) = text[start..].find("==") {
                  let mut highlight = StringV2::default().push_effect(Effect::BrightYellowBackground).push_effect(Effect::Black).push_effect(Effect::Bold);

                  let borders = "<brightyellow>==</brightyellow>";
                  highlight.push_str(borders);
                  highlight.push_str(&text[start..start + end_idx]);
                  highlight.push_str(borders);
                  result.push_str(&highlight.parse_xml(strip_styling).to_string());
                  start += end_idx + 2;
                } else {
                  result.push_str("==");
                  break;
                }
              }
              result.push_str(&text[start..]);
              text = result;
            }

            result.push_str(text);
          },
          Event::End(e) => {
            if !strip_styling {
              continue;
            }

            let tag_name = StringV2::from(e.name().as_ref());

            match Effect::try_from(tag_name.clone()) {
              Some(effect) => {
                let last_tag = active_styles.iter().rposition(|t| *t == effect);
                if let Some(last_effect) = active_styles.last() {
                  let last_effect_name = last_effect.clone().to_string().to_lowercase();

                  if last_effect.ne(&effect) {
                    CONSOLE.panic(
                      if last_tag.is_none() {
                        format!("Cannot close <</{tag_name}>> which is not open")
                      } else {
                        format!("Cannot close <</{tag_name}>> as the last tag was <<{last_effect_name}>>.")
                      }
                    );
                  }
                }

                if let Some(pos) = last_tag {
                  active_styles.truncate(pos);

                  result.push_str(Effect::Reset);
                  result.push_str(active_styles.to_ansi());
                }
              },
              None => {
                if tag_name.len() == 2 && tag_name.starts_with('h') {
                  if let Some(level) = tag_name.chars().nth(1) {
                    if level.is_digit(10) && (1..=6).contains(&(level.to_digit(10).unwrap_or_else(|| CONSOLE.panic("Failed to convert level to digit. Ensure the level is between 1 and 6.")))) {
                      active_styles.retain(|effect: &Effect| *effect != Effect::MagentaBackground);
                      result.push(' ');
                      result.push_str(Effect::Reset);
                      result.push_str(active_styles.to_ansi());
                    }
                  }
                } else {
                  result.push_str(format!("</{tag_name}>"));
                }
              }
            }
          },
          _ => result.push_str(String::from_utf8_lossy(&buf).into_owned())
        }
        buf.clear();
      }

      if strip_styling && !active_styles.is_empty() {
        result.push_str(Effect::Reset);
        result.push_str(active_styles.to_ansi());
      }

      if strip_styling {
        result.push_str(Effect::Reset);
      }
      result
    }

    pub fn render(self: &Self) -> Self {
      // If NO_COLOR is set, don't render any styles
      self.parse_xml(!&*NO_COLOR)
    }

    pub fn strip_styling(self: &Self) -> Self {
      self.parse_xml(false)
    }
  }

  mod implementations {
    pub fn nearest(self: &Self, args: Vec<String>) -> Option<String> {
      let self_str = String::from_utf8(self.bytes().clone())
        .expect("Invalid UTF-8 sequence");
      let mut nearest_word: Option<String> = None;
      let mut min_distance = self.buffer.bytes().capacity().min(usize::MAX);

      for arg in args {
        let distance = levenshtein::levenshtein(&self_str, &arg);
        if distance < min_distance {
          min_distance = distance;
          nearest_word = Some(arg);
        }
      }

      if min_distance == 0 || min_distance <= 2 {
        nearest_word
      } else {
        None
      }
    }

    pub fn append(self: &mut Self, other: &Self) {
      self.buffer_mut().extend_from_buffer(other.buffer());
    }

    pub fn char_at(self: &Self, index: usize) -> Option<char> {
      self.buffer().byte_at(index).map(char::from)
    }

    pub fn concat(self: &Self, other: &Self) -> Self {
      let mut buffer = self.buffer.clone();
      buffer.extend_from_buffer(other.buffer());
      Self {
        buffer,
        global_effects: self.global_effects.clone(),
      }
    }

    pub fn contains(self: &Self, c: char) -> bool {
      self.index_of(c).is_some()
    }

    pub fn ends_with(self: &Self, suffix: impl Into<String>) -> bool {
      self.bytes().ends_with(suffix.into().as_bytes())
    }

    pub fn includes(self: &Self, needle: impl Into<String>) -> bool {
      let needle = needle.into();
      if needle.trim().is_empty() || needle.len() > self.len() {
        return needle.eq(&self.to_string());
      }

      self.bytes().windows(needle.len()).any(|window| window == needle.as_bytes())
    }

    pub fn index_of(self: &Self, c: char) -> Option<usize> {
      self.iter().position(|&b| b == c as u8)
    }

    pub fn last_index_of(self: &Self, c: char) -> Option<usize> {
      self.iter().rposition(|&b| b == c as u8)
    }

    #[allow(unused_variables)]
    pub fn matches(self: &Self, pattern: impl Into<String>) -> bool {
      unimplemented!("StringV2::matches is not implemented yet.");
    }

    #[allow(unused_variables)]
    pub fn matches_all(self: &Self, pattern: impl Into<String>) -> bool {
      unimplemented!("StringV2::matches_all is not implemented yet.");
    }

    pub fn pad_end(self: &Self, target_length: usize, pad_string: impl Into<String>) -> Self {
      let pad_string = pad_string.into();

      if self.len() >= target_length {
        return self.clone();
      }

      let mut vec = Vec::with_capacity(target_length);
      vec.extend_from_slice(&self.bytes().to_vec());
      vec.extend_from_slice(pad_string.as_bytes());
      Self::from(vec)
    }

    pub fn pad_start(self: &Self, target_length: usize, pad_string: impl Into<String>) -> Self {
      let pad_string = pad_string.into();

      if self.len() >= target_length {
        return self.clone();
      }

      let mut vec = Vec::with_capacity(target_length);
      vec.extend_from_slice(pad_string.as_bytes());
      vec.extend_from_slice(&self.bytes().to_vec());
      Self::from(vec)
    }

    pub fn position(self: &Self, string: impl Into<String>) -> Option<usize> {
      let string = string.into();
      self.bytes().windows(string.len()).position(|window| window == string.as_bytes())
    }

    pub fn repeat(self: &Self, count: usize) -> Self {
      let mut vec = Vec::with_capacity(self.len() * count);
      for _ in 0..count {
        vec.extend_from_slice(&self.bytes().to_vec());
      }
      Self::from(vec)
    }

    pub fn replace(self: &Self, from: impl Into<String>, to: impl Into<String>) -> Self {
      let from = from.into();
      let to = to.into();

      if from.is_empty() {
        return self.clone();
      }

      let mut vec = Vec::with_capacity(self.len());
      let mut i = 0;
      if let Some(index) = self.bytes().windows(from.len()).position(|window| window == from.as_bytes()) {
        vec.extend_from_slice(&self.bytes()[i..i + index]);
        vec.extend_from_slice(to.as_bytes());
        i += index + from.len();
        vec.extend_from_slice(&self.bytes()[i..]);
      } else {
        vec.extend_from_slice(&self.bytes()[i..]);
      }

      Self::from(vec)
    }

    pub fn replace_all(self: &Self, from: impl Into<String>, to: impl Into<String>) -> Self {
      let from = from.into();
      let to = to.into();

      if from.is_empty() {
        return self.clone();
      }

      let mut last = self.clone();
      let mut result = self.clone();
      while result.eq(&last) {
        result = result.replace(&from, &to);

        if result.eq(&last) {
          break;
        }

        last = result.clone();
      }

      result
    }

    #[allow(unused_variables)]
    pub fn search(self: &Self, pattern: impl Into<String>) -> Option<usize> {
      unimplemented!("StringV2::search is not implemented yet.");
    }

    pub fn slice(self: &Self, start: usize, end: Option<usize>) -> Self {
      let end = end.unwrap_or_else(|| self.len());
      let start = start.min(self.len());
      let end = end.min(self.len());
      if start >= end {
        return Self::default();
      }

      Self::from(self.bytes()[start..end].to_vec())
    }

    pub fn split(self: &Self, separator: impl Into<String>) -> Vec<Self> {
      let separator = separator.into();
      let mut vec = Vec::new();
      let mut last = 0;
      for (i, _) in self.bytes().windows(separator.len()).enumerate() {
        if &self.bytes()[i..i + separator.len()] == separator.as_bytes() {
          vec.push(self.slice(last, Some(i)));
          last = i + separator.len();
        }
      }

      vec.push(self.slice(last, None));
      vec
    }

    pub fn starts_with(self: &Self, prefix: impl Into<String>) -> bool {
      self.bytes().starts_with(prefix.into().as_bytes())
    }

    pub fn substring(self: &Self, start: usize, end: usize) -> Self {
      self.slice(start, Some(end))
    }

    pub fn to_lowercase(self: &Self) -> Self {
      Self::from(self.bytes().to_ascii_lowercase())
    }

    pub fn to_uppercase(self: &Self) -> Self {
      Self::from(self.bytes().to_ascii_uppercase())
    }

    pub fn trim(self: &Self) -> Self {
      let start = self.iter().position(|&b| b != b' ').unwrap_or(0);
      let end = self
        .iter()
        .rposition(|&b| b != b' ')
        .map_or(0, |pos| pos + 1);

      self.substring(start, end)
    }

    pub fn trim_end(self: &Self) -> Self {
      let end = self
        .iter()
        .rposition(|&b| b != b' ')
        .map_or(0, |pos| pos + 1);

      self.substring(0, end)
    }

    pub fn trim_start(self: &Self) -> Self {
      let start = self.iter().position(|&b| b != b' ').unwrap_or(0);

      self.substring(start, self.len())
    }
  }

  mod compatibility {
    pub fn push_str(self: &mut Self, s: impl Into<String>) {
      let string = s.into();
      match self.buffer_mut().try_reserve(string.len()) {
        Ok(_) => self.bytes_mut().extend_from_slice(string.as_bytes()),
        Err(err) => CONSOLE.panic(format!("{err}"))
      }
    }
  }
}
