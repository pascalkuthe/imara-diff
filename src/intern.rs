use std::hash::Hash;
use std::ops::Index;

use ahash::RandomState;
use hashbrown::raw::RawTable;

/// A token represented as an interned integer.
///
/// A token represents the smallest possible unit of change during a diff.
/// For text this is usually a line, a word or a single character.
/// All [algorithms](crate::Algorithm) operate on interned tokens instead
/// of using the token data directly.
/// This allows for much better performance by amortizing the cost hashing/equality.
///
/// While you can intern tokens yourself it is strongly recommended to use [`InternedInput`] module.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Token(pub u32);

impl From<u32> for Token {
    fn from(token: u32) -> Self {
        Token(token)
    }
}

impl From<Token> for u32 {
    fn from(token: Token) -> Self {
        token.0
    }
}

pub trait TokenSource {
    type Token: Hash + Eq;
    type Tokenizer: Iterator<Item = Self::Token>;
    fn tokenize(&self) -> Self::Tokenizer;
    fn estimate_tokens(&self) -> u32;
}

/// Two lists of interned [tokens](crate::intern::Token) that can be compared with the [`diff`](crate::diff) function.
///
/// A token represents the smallest possible unit of change during a diff.
/// For text this is usually a line, a word or a single character.
/// All [algorithms](crate::Algorithm) operate on interned tokens instead
/// of using the token data directly.
/// This allows for much better performance by amortizing the cost hashing/equality.
///
/// While you can intern tokens yourself it is strongly recommended to use [`InternedInput`] module.
#[derive(Default)]
pub struct InternedInput<T: Eq + Hash> {
    pub before: Vec<Token>,
    pub after: Vec<Token>,
    pub interner: Interner<T>,
}

impl<T: Eq + Hash> InternedInput<T> {
    pub fn new<I: TokenSource<Token = T>>(before: I, after: I) -> Self {
        let token_estimate_before = before.estimate_tokens() as usize;
        let token_estimate_after = after.estimate_tokens() as usize;
        let mut res = Self {
            before: Vec::with_capacity(token_estimate_before),
            after: Vec::with_capacity(token_estimate_after),
            interner: Interner::new(token_estimate_before + token_estimate_after),
        };
        res.update_before(before.tokenize());
        res.update_after(after.tokenize());
        res
    }

    /// replaces `self.before` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long_running process
    /// consider clearing the interner with [`clear`](crate::intern::Interner::clear).
    pub fn update_before(&mut self, input: impl Iterator<Item = T>) {
        self.before.clear();
        self.before
            .extend(input.map(|token| self.interner.intern(token)));
    }

    /// replaces `self.before` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long_running process
    /// consider clearing the interner with [`clear`](crate::intern::Interner::clear) or
    /// [`erase_tokens_after`](crate::intern::Interner::erase_tokens_after).
    pub fn update_after(&mut self, input: impl Iterator<Item = T>) {
        self.after.clear();
        self.after
            .extend(input.map(|token| self.interner.intern(token)));
    }

    pub fn clear(&mut self) {
        self.before.clear();
        self.after.clear();
        self.interner.clear();
    }
}

/// A hastable based interner that allows
#[derive(Default)]
pub struct Interner<T: Hash + Eq> {
    tokens: Vec<T>,
    table: RawTable<Token>,
    hasher: RandomState,
}

impl<T: Hash + Eq> Interner<T> {
    /// Create an Interner with an intial capacity calculated by calling
    /// [`estimate_tokens`](crate::intern::TokenSource::estimate_tokens) methods of `before` and `after`
    pub fn new_for_token_source<S: TokenSource<Token = T>>(before: &S, after: &S) -> Self {
        Self::new(before.estimate_tokens() as usize + after.estimate_tokens() as usize)
    }

    /// Create an Interner with inital capacity `capacity`.
    pub fn new(capacity: usize) -> Interner<T> {
        Interner {
            tokens: Vec::with_capacity(capacity),
            table: RawTable::with_capacity(capacity),
            hasher: RandomState::new(),
        }
    }

    /// Remove all interned tokens
    pub fn clear(&mut self) {
        self.table.clear_no_drop();
        self.tokens.clear();
    }

    /// Intern `token` and return a the interned integer
    pub fn intern(&mut self, token: T) -> Token {
        let hash = self.hasher.hash_one(&token);
        if let Some(&token) = self
            .table
            .get(hash, |&it| self.tokens[it.0 as usize] == token)
        {
            token
        } else {
            let interned = Token(self.tokens.len() as u32);
            self.table.insert(hash, interned, |&token| {
                self.hasher.hash_one(&self.tokens[token.0 as usize])
            });
            self.tokens.push(token);
            interned
        }
    }

    /// Returns to total number of **distinct** tokens currently interned.
    pub fn num_tokens(&self) -> u32 {
        self.tokens.len() as u32
    }

    /// Erases `first_erased_token` and any tokens interned afterwards from the interner.
    pub fn erase_tokens_after(&mut self, first_erased_token: Token) {
        assert!(first_erased_token.0 <= self.tokens.len() as u32);
        let retained = first_erased_token.0 as usize;
        let erased = self.tokens.len() - retained;
        if retained <= erased {
            self.table.clear_no_drop();
            // safety, we assert that retained is smaller then the table size so the table will never have to grow
            unsafe {
                for (i, token) in self.tokens[0..retained].iter().enumerate() {
                    self.table
                        .insert_no_grow(self.hasher.hash_one(token), Token(i as u32));
                }
            }
        } else {
            for (i, token) in self.tokens[retained..].iter().enumerate() {
                self.table
                    .erase_entry(self.hasher.hash_one(token), |token| {
                        token.0 == (retained + i) as u32
                    });
            }
        }
        self.tokens.truncate(first_erased_token.0 as usize);
    }
}

impl<T: Hash + Eq> Index<Token> for Interner<T> {
    type Output = T;
    fn index(&self, index: Token) -> &Self::Output {
        &self.tokens[index.0 as usize]
    }
}
