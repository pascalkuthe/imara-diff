use ahash::RandomState;
use hashbrown::raw::RawTable;
use std::hash::Hash;
use std::ops::Index;

/// A token represented as an interned integer.
///
/// A token represents the smallest possible unit of change during a diff.
/// For text this is usually a line, a word or a single character.
/// All [algorithms](crate::Algorithm) operate on interned tokens instead
/// of using the token data directly.
/// This allows for much better performance by amortizing the cost hashing/equality.
///
/// While you can intern tokens yourself it is strongly recommended to use [`InternedInput`](crate::intern::InternedInput) module.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Token(pub u32);

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
/// While you can intern tokens yourself it is strongly recommended to use [`InternedInput`](crate::intern::InternedInput) module.
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

    pub fn update_before(&mut self, file: impl Iterator<Item = T>) {
        self.before.clear();
        self.before
            .extend(file.map(|token| self.interner.intern(token)));
    }

    pub fn update_after(&mut self, file: impl Iterator<Item = T>) {
        self.after.clear();
        self.after
            .extend(file.map(|token| self.interner.intern(token)));
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
    ///
    pub fn new_for_token_source<S: TokenSource<Token = T>>(file1: &S, file2: &S) -> Self {
        Self::new(file1.estimate_tokens() as usize + file2.estimate_tokens() as usize)
    }

    pub fn new(files_len: usize) -> Self {
        Interner {
            tokens: Vec::with_capacity(files_len),
            table: RawTable::with_capacity(files_len),
            hasher: RandomState::new(),
        }
    }

    pub fn clear(&mut self) {
        self.table.clear_no_drop();
        self.tokens.clear();
    }

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

    pub fn num_tokens(&self) -> u32 {
        self.tokens.len() as u32
    }
}

impl<T: Hash + Eq> Index<Token> for Interner<T> {
    type Output = T;
    fn index(&self, index: Token) -> &Self::Output {
        &self.tokens[index.0 as usize]
    }
}
