use std::ops::Range;

use crate::histogram::lcs::find_lcs;
use crate::histogram::list_pool::{ListHandle, ListPool};
use crate::intern::Token;
use crate::util::{strip_common_postfix, strip_common_prefix};
use crate::{myers, Sink};

mod lcs;
mod list_pool;

const MAX_CHAIN_LEN: u32 = 63;

struct Histogram {
    token_occurances: Vec<ListHandle>,
    pool: ListPool,
}

pub fn diff<S: Sink>(
    mut before: &[Token],
    mut after: &[Token],
    num_tokens: u32,
    mut sink: S,
) -> S::Out {
    let mut histogram = Histogram::new(num_tokens);
    let prefix = strip_common_prefix(&mut before, &mut after);
    strip_common_postfix(&mut before, &mut after);
    histogram.run(before, prefix, after, prefix, &mut sink);
    sink.finish()
}

impl Histogram {
    fn new(num_buckets: u32) -> Histogram {
        Histogram {
            token_occurances: vec![ListHandle::default(); num_buckets as usize],
            pool: ListPool::new(2 * num_buckets),
        }
    }

    fn clear(&mut self) {
        self.pool.clear();
    }

    fn token_occurances(&self, token: Token) -> &[u32] {
        self.token_occurances[token.0 as usize].as_slice(&self.pool)
    }

    fn num_token_occurances(&self, token: Token) -> u32 {
        self.token_occurances[token.0 as usize].len(&self.pool)
    }

    fn populate(&mut self, file: &[Token]) {
        for (i, &token) in file.iter().enumerate() {
            self.token_occurances[token.0 as usize].push(i as u32, &mut self.pool);
        }
    }

    fn run(
        &mut self,
        mut before: &[Token],
        mut before_off: u32,
        mut after: &[Token],
        mut after_off: u32,
        sink: &mut impl Sink,
    ) {
        loop {
            if before.is_empty() {
                if !after.is_empty() {
                    sink.process_change(
                        before_off..before_off,
                        after_off..after_off + after.len() as u32,
                    );
                }
                return;
            } else if after.is_empty() {
                sink.process_change(
                    before_off..before_off + before.len() as u32,
                    after_off..after_off,
                );
                return;
            }

            self.populate(before);
            match find_lcs(before, after, self) {
                // no lcs was found, that means that file1 and file2 two have nothing in common
                Some(lcs) if lcs.len == 0 => {
                    sink.process_change(
                        before_off..before_off + before.len() as u32,
                        after_off..after_off + after.len() as u32,
                    );
                    return;
                }
                Some(lcs) => {
                    self.run(
                        &before[..lcs.before_start as usize],
                        before_off,
                        &after[..lcs.after_start as usize],
                        after_off,
                        sink,
                    );

                    // this is equivalent to (tail) recursion but implement as a loop for efficeny reasons
                    let before_end = lcs.before_start + lcs.len;
                    before = &before[before_end as usize..];
                    before_off += before_end;

                    let after_end = lcs.after_start + lcs.len;
                    after = &after[after_end as usize..];
                    after_off += after_end;
                }
                None => {
                    // we are diffing two extremly large repetitive file
                    // this is a worst case for histogram diff with O(N^2) performance
                    // fallback to myers to maintain linear time complxity
                    myers::diff(
                        before,
                        after,
                        0, // not used by myers
                        |mut before: Range<u32>, mut after: Range<u32>| {
                            before.start += before_off;
                            before.end += before_off;
                            after.start += after_off;
                            after.end += after_off;
                            sink.process_change(before, after)
                        },
                        false,
                    );
                    return;
                }
            }
        }
    }
}
