use crate::intern::Token;
use crate::myers::sqrt;
use crate::util::{strip_common_postfix, strip_common_prefix};

pub fn preprocess(
    mut file1: &[Token],
    mut file2: &[Token],
) -> (PreprocessedFile, PreprocessedFile) {
    let common_prefix = strip_common_prefix(&mut file1, &mut file2);
    strip_common_postfix(&mut file1, &mut file2);
    let (hdiff1, hdiff2) = token_occurrences(file1, file2);
    let file1 = PreprocessedFile::new(common_prefix, &hdiff1, file1);
    let file2 = PreprocessedFile::new(common_prefix, &hdiff2, file2);
    (file1, file2)
}

/// computes how
fn token_occurrences(file1: &[Token], file2: &[Token]) -> (Vec<Occurances>, Vec<Occurances>) {
    const MAX_EQLIMIT: u32 = 1024;

    // compute the limit after which tokens are treated as `Occurances::COMMON`
    let eqlimit1 = sqrt(file1.len()).min(MAX_EQLIMIT);
    let eqlimit2 = sqrt(file2.len()).min(MAX_EQLIMIT);

    // first collect how often each token occurs in a file
    let mut occurances1 = Vec::new();
    for token in file1 {
        let bucket = token.0 as usize;
        if bucket >= occurances1.len() {
            occurances1.resize(bucket + 1, 0u32);
        }
        occurances1[bucket] += 1;
    }

    // do the same thing for
    let mut occurances2 = Vec::new();
    let token_occurances2: Vec<_> = file2
        .iter()
        .map(|token| {
            let bucket = token.0 as usize;
            if bucket >= occurances2.len() {
                occurances2.resize(bucket + 1, 0);
            }
            occurances2[bucket] += 1;
            let occurances1 = *occurances1.get(bucket).unwrap_or(&0);
            Occurances::from_occurances(occurances1, eqlimit2)
        })
        .collect();

    let token_occurances1: Vec<_> = file1
        .iter()
        .map(|token| {
            let bucket = token.0 as usize;
            let occurances2 = *occurances2.get(bucket).unwrap_or(&0);
            Occurances::from_occurances(occurances2, eqlimit1)
        })
        .collect();

    (token_occurances1, token_occurances2)
}

#[derive(Clone, Copy, Debug)]
enum Occurances {
    /// Token does not occur in this file
    None,
    /// Token occurs at least once
    Some,
    /// Token occurs very frequently (exact number depends on file size).
    /// Such a tokens are usually empty lines or braces and are often not meaningful to a diff
    Common,
}

impl Occurances {
    pub fn from_occurances(occurances: u32, eqlimit: u32) -> Occurances {
        if occurances == 0 {
            Occurances::None
        } else if occurances >= eqlimit {
            Occurances::Common
        } else {
            Occurances::Some
        }
    }
}

#[derive(Debug)]
pub struct PreprocessedFile {
    pub offset: u32,
    pub is_changed: Vec<bool>,
    pub indices: Vec<u32>,
    pub tokens: Vec<Token>,
}

impl PreprocessedFile {
    fn new(offset: u32, token_diff: &[Occurances], tokens: &[Token]) -> PreprocessedFile {
        let mut changed = vec![false; tokens.len()];
        let (tokens, indices) = prune_unmatched_tokens(tokens, token_diff, &mut changed);
        PreprocessedFile {
            offset,
            is_changed: changed,
            indices,
            tokens,
        }
    }
}

fn prune_unmatched_tokens(
    file: &[Token],
    token_status: &[Occurances],
    changed: &mut [bool],
) -> (Vec<Token>, Vec<u32>) {
    assert_eq!(token_status.len(), file.len());
    file.iter()
        .zip(token_status)
        .enumerate()
        .filter_map(|(i, (&token, &status))| {
            let prune = match status {
                Occurances::None => true,
                Occurances::Some => false,
                Occurances::Common => should_prune_common_line(token_status, i),
            };
            if prune {
                changed[i] = true;
                None
            } else {
                Some((token, i as u32))
            }
        })
        .unzip()
}

// TODO do not unnecessarily rescan lines
fn should_prune_common_line(token_status: &[Occurances], pos: usize) -> bool {
    const WINDOW_SIZE: usize = 100;

    let mut unmatched_before = 0;
    let mut common_before = 0;

    let start = if pos > WINDOW_SIZE { WINDOW_SIZE } else { 0 };
    for status in token_status[start..pos].iter().rev() {
        match status {
            Occurances::None => {
                unmatched_before += 1;
            }
            Occurances::Common => {
                common_before += 1;
            }
            Occurances::Some => break,
        }
    }

    if unmatched_before == 0 {
        return false;
    }

    let end = token_status.len().min(pos + WINDOW_SIZE);
    let mut unmatched_after = 0;
    let mut common_after = 0;
    for status in token_status[pos..end].iter() {
        match status {
            Occurances::None => {
                unmatched_after += 1;
            }
            Occurances::Common => {
                common_after += 1;
            }
            Occurances::Some => break,
        }
    }

    if unmatched_after == 0 {
        return false;
    }

    let common = common_before + common_after;
    let unmatched = unmatched_before + unmatched_after;

    unmatched > 3 * common
}
