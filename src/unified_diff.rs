/// Defines the size of the context printed before and after each change.
///
/// Similar to the -U option in git diff or gnu-diff. If the context overlaps
/// with previous or next change, the context gets reduced accordingly.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct ContextSize {
    /// Defines the size of the context printed before and after each change.
    symmetrical: u32,
}

impl Default for ContextSize {
    fn default() -> Self {
        ContextSize::symmetrical(3)
    }
}

/// Instantiation
impl ContextSize {
    /// Create a symmetrical context with `n` lines before and after a changed hunk.
    pub fn symmetrical(n: u32) -> Self {
        ContextSize { symmetrical: n }
    }
}

pub(super) mod _impl {
    use std::fmt::{Display, Write};
    use std::ops::Range;

    use super::ContextSize;
    use crate::intern::{InternedInput, Interner, Token};
    use crate::Sink;

    /// A [`Sink`] that creates a textual diff
    /// in the format typically output by git or gnu-diff if the `-u` option is used
    pub struct UnifiedDiffBuilder<'a, W, T>
    where
        W: Write,
        T: Display,
    {
        before: &'a [Token],
        after: &'a [Token],
        interner: &'a Interner<T>,

        pos: u32,
        before_hunk_start: u32,
        after_hunk_start: u32,
        before_hunk_len: u32,
        after_hunk_len: u32,
        /// Symmetrical context before and after the changed hunk.
        ctx_size: u32,

        buffer: String,
        dst: W,
    }

    impl<'a, T> UnifiedDiffBuilder<'a, String, T>
    where
        T: Display,
    {
        /// Create a new `UnifiedDiffBuilder` for the given `input`,
        /// displaying `context_size` lines of context around each change,
        /// that will return a [`String`].
        pub fn new(input: &'a InternedInput<T>, context_size: ContextSize) -> Self {
            Self {
                before_hunk_start: 0,
                after_hunk_start: 0,
                before_hunk_len: 0,
                after_hunk_len: 0,
                buffer: String::with_capacity(8),
                dst: String::new(),
                interner: &input.interner,
                before: &input.before,
                after: &input.after,
                pos: 0,
                ctx_size: context_size.symmetrical,
            }
        }
    }

    impl<'a, W, T> UnifiedDiffBuilder<'a, W, T>
    where
        W: Write,
        T: Display,
    {
        /// Create a new `UnifiedDiffBuilder` for the given `input`,
        /// displaying `context_size` lines of context around each change,
        /// that will writes it output to the provided implementation of [`Write`].
        pub fn with_writer(
            input: &'a InternedInput<T>,
            writer: W,
            context_size: Option<u32>,
        ) -> Self {
            Self {
                before_hunk_start: 0,
                after_hunk_start: 0,
                before_hunk_len: 0,
                after_hunk_len: 0,
                buffer: String::with_capacity(8),
                dst: writer,
                interner: &input.interner,
                before: &input.before,
                after: &input.after,
                pos: 0,
                ctx_size: context_size.unwrap_or(3),
            }
        }

        fn print_tokens(&mut self, tokens: &[Token], prefix: char) {
            for &token in tokens {
                writeln!(&mut self.buffer, "{prefix}{}", self.interner[token]).unwrap();
            }
        }

        fn flush(&mut self) {
            if self.before_hunk_len == 0 && self.after_hunk_len == 0 {
                return;
            }

            let end = (self.pos + self.ctx_size).min(self.before.len() as u32);
            self.update_pos(end, end);

            writeln!(
                &mut self.dst,
                "@@ -{},{} +{},{} @@",
                self.before_hunk_start + 1,
                self.before_hunk_len,
                self.after_hunk_start + 1,
                self.after_hunk_len,
            )
            .unwrap();
            write!(&mut self.dst, "{}", &self.buffer).unwrap();
            self.buffer.clear();
            self.before_hunk_len = 0;
            self.after_hunk_len = 0
        }

        fn update_pos(&mut self, print_to: u32, move_to: u32) {
            self.print_tokens(&self.before[self.pos as usize..print_to as usize], ' ');
            let len = print_to - self.pos;
            self.pos = move_to;
            self.before_hunk_len += len;
            self.after_hunk_len += len;
        }
    }

    impl<W, T> Sink for UnifiedDiffBuilder<'_, W, T>
    where
        W: Write,
        T: Display,
    {
        type Out = W;

        fn process_change(&mut self, before: Range<u32>, after: Range<u32>) {
            if ((self.pos == 0) && (before.start - self.pos > self.ctx_size))
                || (before.start - self.pos > 2 * self.ctx_size)
            {
                self.flush();
                self.pos = before.start - self.ctx_size;
                self.before_hunk_start = self.pos;
                self.after_hunk_start = after.start - self.ctx_size;
            }
            self.update_pos(before.start, before.end);
            self.before_hunk_len += before.end - before.start;
            self.after_hunk_len += after.end - after.start;
            self.print_tokens(
                &self.before[before.start as usize..before.end as usize],
                '-',
            );
            self.print_tokens(&self.after[after.start as usize..after.end as usize], '+');
        }

        fn finish(mut self) -> Self::Out {
            self.flush();
            self.dst
        }
    }
}
