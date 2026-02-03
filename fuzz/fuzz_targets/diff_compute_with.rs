#![no_main]

use imara_diff::{Algorithm, Diff, Token};
use libfuzzer_sys::fuzz_target;

/// Tests the lower-level `compute_with` API that works directly with Token sequences:
/// - Creating arbitrary token sequences
/// - Computing diffs with all algorithms
/// - Querying individual token states
/// - Iterating through hunks
fn do_fuzz(data: &[u8]) {
    // Test the lower-level compute_with API that works directly with tokens
    if data.len() < 4 {
        return;
    }

    // Use first two bytes to determine before/after lengths
    let before_len = (data[0] as usize % 100).min(data.len() / 2);
    let after_len = (data[1] as usize % 100).min(data.len() / 2);

    // Create token sequences from remaining bytes
    let mut before_tokens = Vec::new();
    let mut after_tokens = Vec::new();

    for i in 0..before_len {
        if i + 2 < data.len() {
            before_tokens.push(Token::from(data[i + 2] as u32 % 256));
        }
    }

    for i in 0..after_len {
        if i + 2 + before_len < data.len() {
            after_tokens.push(Token::from(data[i + 2 + before_len] as u32 % 256));
        }
    }

    // Test all algorithms with compute_with
    for algorithm in [
        Algorithm::Histogram,
        Algorithm::Myers,
        Algorithm::MyersMinimal,
    ] {
        let mut diff = Diff::default();
        diff.compute_with(algorithm, &before_tokens, &after_tokens, 256);

        // Test basic queries
        let _ = diff.count_additions();
        let _ = diff.count_removals();

        // Test is_removed and is_added for valid indices
        for i in 0..before_tokens.len() as u32 {
            let _ = diff.is_removed(i);
        }
        for i in 0..after_tokens.len() as u32 {
            let _ = diff.is_added(i);
        }

        // Test hunks
        for hunk in diff.hunks() {
            let _ = hunk.is_pure_insertion();
            let _ = hunk.is_pure_removal();
        }
    }
}

fuzz_target!(|data: &[u8]| {
    do_fuzz(data);
});
