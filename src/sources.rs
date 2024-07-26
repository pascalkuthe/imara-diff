use std::mem::take;
use std::str::from_utf8_unchecked;

use crate::TokenSource;

/// Returns a [`TokenSource`] that uses
/// the lines in `data` as Tokens. The newline seperator (`\r\n` or `\n`) is
/// not included in the emitted tokens.
/// This means that changing the newline seperator from `\r\n` to `\n`
/// (or omitting it fully on the last line) is not detected by [`diff`](crate::diff).
pub fn lines(data: &str) -> Lines<'_, false> {
    Lines(ByteLines(data.as_bytes()))
}

/// Returns a [`TokenSource`] that uses
/// the lines in `data` as Tokens. The newline seperator (`\r\n` or `\n`) is
/// included in the emitted tokens.
/// This means that changing the newline seperator from `\r\n` to `\n`
/// (or omitting it fully on the last line) is  detected by [`diff`](crate::diff).
pub fn lines_with_terminator(data: &str) -> Lines<'_, true> {
    Lines(ByteLines(data.as_bytes()))
}

/// Returns a [`TokenSource`] that uses
/// the lines in `data` as Tokens. A lines is a continous subslice of
/// `data` which does not contain `\n` (or `\r\n`).
/// The newline seperator (`\r\n` or `\n`) is not included in the emitted tokens.
/// This means that changing the newline seperator from `\r\n` to `\n`
/// (or omitting it fully on the last line) is not detected by [`diff`](crate::diff).
pub fn byte_lines_with_terminator(data: &[u8]) -> ByteLines<'_, true> {
    ByteLines(data)
}

/// Returns a [`TokenSource`] that uses
/// the lines in `data` as Tokens. The newline seperator (`\r\n` or `\n`) is
/// included in the emitted tokens.
/// This means that changing the newline seperator from `\r\n` to `\n`
/// (or omitting it fully on the last line) is  detected by [`diff`](crate::diff).
pub fn byte_lines(data: &[u8]) -> ByteLines<'_, false> {
    ByteLines(data)
}

/// By default a line diff is produced for a string
impl<'a> TokenSource for &'a str {
    type Token = &'a str;

    type Tokenizer = Lines<'a, false>;

    fn tokenize(&self) -> Self::Tokenizer {
        lines(self)
    }

    fn estimate_tokens(&self) -> u32 {
        lines_with_terminator(self).estimate_tokens()
    }
}

/// By default a line diff is produced for a bytes
impl<'a> TokenSource for &'a [u8] {
    type Token = Self;
    type Tokenizer = ByteLines<'a, false>;

    fn tokenize(&self) -> Self::Tokenizer {
        byte_lines(self)
    }

    fn estimate_tokens(&self) -> u32 {
        byte_lines(self).estimate_tokens()
    }
}

/// A [`TokenSource`] that returns the lines of a `str` as tokens.
/// See [`lines`] and [`lines_with_terminator`] for details
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Lines<'a, const INCLUDE_LINE_TERMINATOR: bool>(ByteLines<'a, INCLUDE_LINE_TERMINATOR>);

impl<'a, const INCLUDE_LINE_TERMINATOR: bool> Iterator for Lines<'a, INCLUDE_LINE_TERMINATOR> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // safety invariant: this struct may only contain valid utf8
        // dividing valid utf8 bytes by ascii characters always produces valid utf-8
        self.0.next().map(|it| unsafe { from_utf8_unchecked(it) })
    }
}

/// By default a line diff is produced for a string
impl<'a, const INCLUDE_LINE_TERMINATOR: bool> TokenSource for Lines<'a, INCLUDE_LINE_TERMINATOR> {
    type Token = &'a str;

    type Tokenizer = Self;

    fn tokenize(&self) -> Self::Tokenizer {
        *self
    }

    fn estimate_tokens(&self) -> u32 {
        self.0.estimate_tokens()
    }
}

/// A [`TokenSource`] that returns the lines of a byte slice as tokens.
/// See [`byte_lines`] and [`byte_lines_with_terminator`] for details
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ByteLines<'a, const INCLUDE_LINE_TERMINATOR: bool>(&'a [u8]);

impl<'a, const INCLUDE_LINE_TERMINATOR: bool> Iterator for ByteLines<'a, INCLUDE_LINE_TERMINATOR> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let mut saw_carriage_return = false;
        let mut iter = self.0.iter().enumerate();
        let line_len = loop {
            match iter.next() {
                Some((i, b'\n')) => break i + 1,
                None => {
                    return (!self.0.is_empty()).then(|| take(&mut self.0));
                }
                Some((_, &it)) => saw_carriage_return = it == b'\r',
            }
        };
        let (mut line, rem) = self.0.split_at(line_len);
        self.0 = rem;
        if !INCLUDE_LINE_TERMINATOR {
            line = &line[..line_len - 1 - saw_carriage_return as usize];
        }
        Some(line)
    }
}

/// By default a line diff is produced for a string
impl<'a, const INCLUDE_LINE_TERMINATOR: bool> TokenSource
    for ByteLines<'a, INCLUDE_LINE_TERMINATOR>
{
    type Token = &'a [u8];

    type Tokenizer = Self;

    fn tokenize(&self) -> Self::Tokenizer {
        *self
    }

    fn estimate_tokens(&self) -> u32 {
        let len: usize = self.take(20).map(|line| line.len()).sum();
        if len == 0 {
            100
        } else {
            (self.0.len() * 20 / len) as u32
        }
    }
}
