pub mod ansi;
pub mod buffer;

use std::{fmt::{Display, Formatter}, slice::{Iter, IterMut}, string::FromUtf8Error, vec::IntoIter};

use ansi::{Effect, EffectArray};
use crate::{console::CONSOLE, enum_gen, struct_gen};
use buffer::Buffer;

#[allow(dead_code)]
const UTF_QUESTIONMARK: &str = "\u{FFFD}";

enum_gen! {
  enum Tags {
    PreventsPanic
  }

  mod implementations {
    pub fn to_ansi(self: &Self) -> Option<StringV2> {
      // They should only be used in debug modes to prevent misuse/unwanted behavior.
      match self {
        #[cfg(debug_assertions)]
        Self::PreventsPanic => Some(StringV2::from("<redbackground><red>[</red><bold>THIS MESSAGE PREVENTS A PANIC!</bold><red>]</red></redbackground>").render_ansi()),
        #[allow(unreachable_patterns)]
        _ => None
      }
    }
  }
}

struct_gen! {
  pub struct StringV2 use PartialEq, PartialOrd, Eq, Ord {
    pub(super) let &mut buffer: Buffer = Buffer::new();
    let global_effects: EffectArray = EffectArray::new();
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
        global_effects: EffectArray::new(),
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
        global_effects: EffectArray::new(),
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
        return Self::new();
      }

      Self::from(s.as_bytes().to_vec())
    }
  }

  impl From<String> {
    fn from(s: String) -> Self {
      if s.is_empty() {
        return Self::new();
      }

      Self::from(s.as_bytes().to_vec())
    }
  }

  impl From<&String> {
    fn from(s: &String) -> Self {
      if s.is_empty() {
        return Self::new();
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
    #[inline]
    fn fmt(self: &Self, f: &mut Formatter) -> std::fmt::Result {
      write!(f, "{}", self.render_ansi().to_string())
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
        global_effects: EffectArray::new(),
      }
    }

    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
      match String::from_utf8(vec.clone()) {
        Ok(..) => Ok(Self {
          buffer: Buffer::from(vec),
          global_effects: EffectArray::new(),
        }),
        Err(error) => Err(FromUtf8Error::from(error)),
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
        global_effects: EffectArray::new(),
      }
    }

    #[doc = "Sets a global effect: \n - `Effect::Bold`"]
    pub fn bold(self: &Self) -> Self {
      self.push_effect(Effect::Bold)
    }

    #[doc = "Sets a global effect: \n - `Effect::Italic`"]
    pub fn italic(self: &Self) -> Self {
      self.push_effect(Effect::Italic)
    }

    #[doc = "Sets a global effect: \n - `Effect::Underline`"]
    pub fn underline(self: &Self) -> Self {
      self.push_effect(Effect::Underline)
    }

    #[doc = "Sets a global effect: \n - `Effect::Blink`"]
    pub fn blink(self: &Self) -> Self {
      self.push_effect(Effect::Blink)
    }

    #[doc = "Sets a global effect: \n - `Effect::Inverse`"]
    pub fn inverse(self: &Self) -> Self {
      self.push_effect(Effect::Inverse)
    }

    #[doc = "Sets a global effect: \n - `Effect::Hidden`"]
    pub fn hidden(self: &Self) -> Self {
      self.push_effect(Effect::Hidden)
    }

    #[doc = "Sets a global effect: \n - `Effect::Strikethrough`"]
    pub fn strike(self: &Self) -> Self {
      self.push_effect(Effect::Strikethrough)
    }

    #[doc = "Pushes a new global effect."]
    pub fn push_effect(self: &Self, effect: impl Into<Effect>) -> Self {
      Self {
        buffer: self.buffer.clone(),
        global_effects: {
          let mut styles = self.global_effects.clone();
          styles.push(effect.into());
          styles
        },
      }
    }
  }

  mod ansi_implementations {
    pub fn render_ansi(self: &Self) -> Self {
      if std::env::var("NO_COLOR").is_ok() {
        return self.strip_ansi();
      }

      let mut result = Self::new();

      if self.bytes().is_empty() {
        return result;
      }

      let mut active_styles = self.global_effects().effects().clone();

      let mut tag_stack: Vec<String> = Vec::new();

      for style in &active_styles {
        result.push_str(style.to_ansi());
        tag_stack.push(style.clone().to_string());
      }

      let mut iter = self.chars().peekable();
      while let Some(ch) = iter.next() {
        match ch {
          ':' => {
            if let Some(&next_ch) = iter.peek() {
              if next_ch == ':' {
                result.push_str("::");
                iter.next();
                continue;
              }
            }

            let mut code = String::new();
            let mut ended = false;
            while let Some(&next_ch) = iter.peek() {
              if next_ch == ':' {
                ended = true;
                iter.next();
                break;
              }

              iter.next();
              if next_ch == ' ' {
                ended = true;
                result.push_str(&format!(":{code} "));
                code.clear();
                break;
              }

              code.push(next_ch);
            }

            if !ended {
              if code.is_empty() {
                continue;
              }

              result.push_str(&format!(":{code}"));
              continue;
            }

            if code.is_empty() {
              continue;
            }

            match Tags::try_from(code.as_str()) {
              Some(code_enum) => {
                match code_enum.to_ansi() {
                  Some(ansi) => result.push_str(ansi.to_string()),
                  None => result.push_str(&format!(":{code}:"))
                }
              },
              None => result.push_str(&format!(":{code}:"))
            }
          },
          '<' => {
            if let Some(next) = iter.peek() {
              if next == &'>' {
                result.push_str("<>");
                continue;
              }

              if next == &'<' {
                result.push('<');
                while let Some(&next_ch) = iter.peek() {
                  if next_ch != '<' {
                    break;
                  }
                  iter.next();
                }
                continue;
              }
            }

            let mut tag = String::new();
            let mut is_closing_tag = false;

            while let Some(&next_ch) = iter.peek() {
              iter.next();
              if next_ch == '>' {
                break;
              }

              if next_ch == '/' && tag.is_empty() {
                is_closing_tag = true;
              } else {
                tag.push(next_ch);
              }
            }

            if tag.is_empty() {
              continue;
            }

            if is_closing_tag {
              let last_pos = tag_stack.iter().rposition(|t| *t == tag);
              if let Some(last_effect) = active_styles.last() {
                if let Some(effect) = Effect::try_from(&tag) {
                  if last_effect.ne(&effect) {
                    if last_pos.is_none() {
                      CONSOLE.panic(format!("You're trying to close the tag </{tag}> before opening it.\nWhich is illegal"));
                    }

                    let last_tag = last_effect.clone().to_string().to_lowercase();
                    CONSOLE.panic(format!("You're trying to close the tag </{tag}> before </{last_tag}>.\nWhich is illegal"));
                  }
                }
              }

              if let Some(pos) = last_pos {
                tag_stack.truncate(pos);
                active_styles.truncate(pos);

                result.push_str(Effect::Reset);
                for style in &active_styles {
                  result.push_str(style.to_ansi());
                }
              } else {
                result.push_str(&format!("</{tag}>"));
              }
            } else {
              match Effect::try_from(&tag) {
                Some(effect) => {
                  active_styles.push(effect.clone());
                  tag_stack.push(tag.clone());
                  result.push_str(effect.to_ansi());
                }
                None => {
                  result.push_str(&format!("<{tag}>"));
                }
              }
            }
          },
          _ => {
            if ch == '>' {
              if let Some(next) = iter.peek() {
                if next == &'>' {
                  result.push('>');
                  while let Some(&next_ch) = iter.peek() {
                    if next_ch != '>' {
                      break;
                    }
                    iter.next();
                  }
                }
              }
            } else {
              result.push(ch);
            }
          }
        }
      }

      if !active_styles.is_empty() {
        while let Some(_) = tag_stack.pop() {
          if let Some(_) = active_styles.pop() {
            result.push_str(Effect::Reset);
            for style in &active_styles {
              result.push_str(style.to_ansi());
            }
          }
        }
      }

      result.push_str(Effect::Reset);
      result
    }

    pub fn strip_ansi(self: &Self) -> Self {
      let mut result = Self::new();
      if self.bytes().is_empty() {
        return result;
      }

      let mut iter = self.chars().peekable();
      while let Some(ch) = iter.next() {
        match ch {
          ':' => {
            let mut code = String::new();
            while let Some(&next_ch) = iter.peek() {
              if next_ch == ':' {
                iter.next();
                break;
              }
              code.push(next_ch);
              iter.next();
            }

            match Tags::try_from(code.as_str()) {
              Some(code_enum) => {
                match code_enum.to_ansi() {
                  Some(ansi) => result.push_str(ansi.strip_ansi().to_string()),
                  None => result.push_str(&format!(":{code}:"))
                }
              },
              None => result.push_str(&format!(":{code}:"))
            }
          },
          '<' => {
            if let Some(next) = iter.peek() {
              if next == &'<' {
                result.push('<');
                iter.next();
                continue;
              }
            }

            let mut tag = String::new();
            while let Some(&next_ch) = iter.peek() {
              iter.next();
              if next_ch == '>' {
                break;
              }

              if !(next_ch == '/' && tag.is_empty()) {
                tag.push(next_ch);
              }
            }

            if let None = Effect::try_from(&tag) {
              result.push_str(&format!("<{tag}>"));
            }
          },
          _ => {
            if ch == '>' {
              if let Some(next) = iter.peek() {
                if next == &'>' {
                  result.push('>');
                  iter.next();
                }
              }
            } else {
              result.push(ch);
            }
          }
        }
      }

      result
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

      self.bytes().windows(needle.bytes().len()).any(|window| window == needle.as_bytes())
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
        return Self::new();
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
