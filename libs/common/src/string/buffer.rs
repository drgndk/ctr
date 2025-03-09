use std::{slice::Iter, string::FromUtf8Error};

use crate::{console::CONSOLE, env::consts::IS_DEBUG, string::StringV2, struct_gen};

struct_gen! {
  pub struct Buffer use Clone, Ord, Eq, PartialOrd, PartialEq {
    pub(super) let &mut bytes: Vec<u8> = Vec::new();
  }

  impl std::fmt::Display {
    fn fmt(self: &Self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "Buffer<{}>", self.bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join(" "))
    }
  }

  impl From<char> {
    fn from(ch: char) -> Self {
      Self {
        bytes: vec![ch as u8],
      }
    }
  }

  impl From<&str> {
    fn from(string: &str) -> Self {
      Self::from(string.as_bytes().to_vec())
    }
  }

  impl From<String> {
    fn from(string: String) -> Self {
      Self::from(&string)
    }
  }

  impl From<&String> {
    fn from(string: &String) -> Self {
      Self::from(string.as_bytes().to_vec())
    }
  }

  impl From<Vec<u8>> {
    fn from(bytes: Vec<u8>) -> Self {
      Self {
        bytes,
      }
    }
  }

  mod static_functions {
    /// Constructs a new, empty `Buffer` with at least the specified capacity.
    ///
    /// The buffer will be able to hold at least `capacity` bytes without reallocating.
    /// If `capacity` is 0, the buffer will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = Buffer::with_capacity(10);
    ///
    /// assert_eq!(buffer.size(), 0);
    /// assert!(buffer.capacity() >= 10);
    ///
    /// for i in 0..10 {
    ///   buffer.push(i);
    /// }
    /// assert_eq!(buffer.size(), 10);
    /// assert!(buffer.capacity() >= 10);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
      // If the string is longer than the provided capacity, print a warning. As it's now lossy.
      if IS_DEBUG && capacity.eq(&0) {
        CONSOLE.debug("Allocating a buffer with a capacity of <bold>0 bytes</bold>. This is not recommended as it requires a <magenta>buf.try_reserve()</magenta> call on change.");
      }

      Self {
        bytes: Vec::with_capacity(capacity),
      }
    }

    /// Allocates a buffer with the provided string or character, filling up to the specified capacity.
    ///
    /// If the string is empty, it will fill the buffer with null bytes.
    /// If the string is longer than the provided capacity, it will truncate the string.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = Buffer::alloc(10, "Hello, World!");
    /// assert_eq!(buffer.text().unwrap(), StringV2::from("Hello, Wor"));
    /// ```
    ///
    /// ```
    /// let buffer = Buffer::alloc(10, 'a');
    /// assert_eq!(buffer.text().unwrap(), StringV2::from("aaaaaaaaaa"));
    /// ```
    pub fn alloc(capacity: usize, bytes: impl Into<String>) -> Self {
      let str_bytes = bytes.into();
      let mut result = Self::with_capacity(capacity);

      if str_bytes.is_empty() {
        for _ in 0..capacity {
          result.push(0);
        }

        return result;
      }

      let length = str_bytes.len();

      // If the string is longer than the provided capacity, print a warning as it's lossy.
      if IS_DEBUG && length > capacity {
        CONSOLE.suggest(
          format!("Allocating a buffer with a capacity of <bold>{capacity} bytes</bold>, but the provided string is <bold>{length} bytes long</bold>."),
          vec![concat!("Buffer::alloc({{length}}, {str_bytes:?})")]
        );
      }

      loop {
        if length >= capacity {
          break;
        }

        for ch in str_bytes.chars().take(capacity - length) {
          result.push(ch);
        }
      }

      result
    }
  }

  mod conversion {
    /// Returns the buffer as a slice.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    ///
    /// assert_eq!(buffer.as_slice(), &[72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]);
    /// ```
    pub fn as_slice(self: &Self) -> &[u8] {
      self.bytes.as_slice()
    }

    /// Returns the buffer as a `StringV2`.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    ///
    /// assert_eq!(buffer.text().unwrap(), StringV2::from("Hello, World!"));
    /// ```
    ///
    /// # Error
    /// If the buffer contains invalid UTF-8.
    pub fn text(self: &Self) -> Result<StringV2, FromUtf8Error> {
      if self.bytes.is_empty() {
        CONSOLE.suggest("Converting an empty buffer to a string", vec![stringify!(StringV2::new())]);

        return Ok(StringV2::default());
      }

      StringV2::from_utf8(self.bytes.clone())
    }

    /// Converts the buffer to a `Vec<u8>`.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    ///
    /// assert_eq!(buffer.to_vec(), vec![72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]);
    /// ```
    pub fn to_vec(self: &Self) -> Vec<u8> {
      self.bytes.clone()
    }
  }

  mod implementations {
    /// Returns the number of bytes in the buffer.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::alloc(10, "Hello, World!");
    ///
    /// assert_eq!(buffer.size(), 10);
    /// ```
    pub fn size(self: &Self) -> usize {
      self.bytes.len()
    }

    /// Returns the capacity of the buffer.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::with_capacity(10);
    ///
    /// assert!(buffer.capacity() >= 10);
    /// ```
    pub fn capacity(self: &Self) -> usize {
      self.bytes.capacity()
    }

    /// Returns how many times the provided byte appears in the buffer.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    ///
    /// assert_eq!(buffer.count('l'), 3);
    /// ```
    pub fn count(self: &Self, byte: impl Into<char>) -> usize {
      let char_byte = byte.into();
      self.bytes().iter().filter(|&&b| b == char_byte as u8).count()
    }

    /// Gets the byte at the specified index.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    ///
    /// assert_eq!(buffer.byte_at(0), Some(72));
    /// assert_eq!(buffer.byte_at(1), Some(101));
    /// assert_eq!(buffer.byte_at(2), Some(108));
    /// ```
    pub fn byte_at(self: &Self, index: usize) -> Option<u8> {
      self.bytes.get(index).copied()
    }

    /// Returns an iterator over the slice.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    /// let mut iterator = buffer.iter();
    ///
    /// assert_eq!(iterator.next(), Some(&72));
    /// assert_eq!(iterator.next(), Some(&101));
    /// assert_eq!(iterator.next(), Some(&108));
    /// ```
    pub fn iter(self: &Self) -> Iter<u8> {
      self.bytes.iter()
    }

    /// Returns an iterator that allows modifying each value.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Example
    /// ```
    /// let mut buffer = Buffer::from("Hello, World!");
    ///
    /// for elem in buffer.iter_mut() {
    ///   *elem += 2;
    /// }
    ///
    /// assert_eq!(buffer.text().unwrap(), StringV2::from("Jgnnq.\"Yqtnf#"));
    /// ```
    pub fn iter_mut(self: &mut Self) -> std::slice::IterMut<u8> {
      self.bytes.iter_mut()
    }

    /// Pushes a byte into the buffer.
    ///
    /// # Example
    /// ```
    /// let mut buffer = Buffer::with_capacity(10);
    ///
    /// for i in 0..10 {
    ///  buffer.push(i);
    /// }
    ///
    /// // This will panic, as the buffer is full
    /// buffer.push(11);
    /// ```
    ///
    /// # Panics
    /// This function will panic if the buffer is full and you try to push more items.
    ///
    /// # Suggestions
    /// Consider using `Buffer::push_safe()` to avoid panics.
    ///
    /// For larger operations, consider using `Buffer::try_reserve()` before pushing items. To minimize reallocation requests.
    pub fn push(self: &mut Self, byte: impl Into<char>) {
      if self.size().saturating_add(1) > self.capacity() {
        if IS_DEBUG {
          CONSOLE.suggest("$preventspanic Attempted to push a byte into a full buffer.", vec![stringify!(Buffer::push_safe())]);
          return;
        }

        #[cfg(not(debug_assertions))]
        CONSOLE.panic("Attempted to push a byte into a full buffer. Consider using Buffer::push_safe() or Buffer::try_reserve() instead.");
      }

      let char_byte = byte.into();
      self.bytes.push(char_byte as u8);
    }

    /// Safe version of `push` that returns a `Result` instead of panicking.
    ///
    /// This is useful when you want to push single items into the buffer, but don't want to panic if the buffer is full.
    /// For larger operations, consider using `Buffer::try_reserve()` instead.
    ///
    /// # Example
    /// ```
    /// let mut buffer = Buffer::with_capacity(10);
    ///
    /// for i in 0..10 {
    ///  buffer.push(i);
    /// }
    ///
    /// // This tries to reserve one extra byte, if it fails, it returns an error.
    /// // Otherwise, it pushes the byte into the buffer.
    /// buffer.push_safe(11).unwrap();
    /// ```
    ///
    /// # Error
    /// If the buffer cannot reserve additional space.
    pub fn push_safe(self: &mut Self, byte: impl Into<char>) -> Result<(), std::collections::TryReserveError> {
      match self.bytes.try_reserve(1) {
        Ok(_) => {
            self.push(byte);
            Ok(())
        },
        Err(err) => Err(err)
      }
    }

    /// Returns true if the size is 0.
    ///
    /// # Example
    /// ```
    /// let buffer = Buffer::new();
    ///
    /// assert!(buffer.is_empty()); // returns true
    /// ```
    ///
    /// ```
    /// let buffer = Buffer::from("Hello, World!");
    ///
    /// assert!(!buffer.is_empty()); // returns false
    /// ```
    pub fn is_empty(self: &Self) -> bool {
      self.bytes.is_empty()
    }

    /// Clears the buffer, removing all bytes.
    ///
    /// # Example
    /// ```
    /// let mut buffer = Buffer::from("Hello, World!");
    /// buffer.clear();
    ///
    /// assert!(buffer.is_empty());
    /// assert_eq!(buffer.size(), 0);
    /// ```
    pub fn clear(self: &mut Self) {
      self.bytes.clear();
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted in the given `Buffer`.
    ///
    /// # Example
    /// ```
    /// let mut buffer = Buffer::with_capacity(10);
    ///
    /// assert_eq!(buffer.capacity(), 10);
    ///
    /// buffer.try_reserve(10).unwrap();
    ///
    /// assert!(buffer.capacity() >= 20);
    /// ```
    ///
    /// # Error
    /// If the new capacity overflows, or the allocator reports a failure.
    pub fn try_reserve(self: &mut Self, additional: usize) -> Result<(), std::collections::TryReserveError> {
      self.bytes.try_reserve(additional)
    }

    /// Clones and appends all elements in a Buffer to another Buffer instance.
    ///
    /// # Example
    /// ```
    /// let mut buffer = Buffer::from("Hello, ");
    /// let buffer2 = Buffer::from("World!");
    ///
    /// buffer.extend_from_buffer(&buffer2);
    /// ```
    pub fn extend_from_buffer(self: &mut Self, bytes: &Buffer) {
      self.bytes.extend_from_slice(bytes.as_slice());
    }
  }
}
