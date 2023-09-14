//! Imara-diff is a solid (imara in swahili) diff library for rust.
//! Solid refers to the fact that imara-diff provides very good runtime performance even
//! in pathologic cases so that your application never appears to freeze while waiting on a diff.
//! The performance improvements are achieved using battle tested heuristics used in gnu-diff and git
//! that are known to yield fast runtime and performance.
//!
//! Imara-diff is also designed to be flexible so that it can be used with arbitrary collections and
//! not just lists and strings and even allows reusing large parts of the computation when
//! comparing the same file to multiple different files.
//!
//! Imara-diff provides two diff algorithms:
//!
//! * The linear-space variant of the well known [**myer** algorithm](http://www.xmailserver.org/diff2.pdf)
//! * The **Histogram** algorithm which variant of the patience diff algorithm.
//!
//! Myers algorithm has been enhanced with preprocessing and multiple heuristics to ensure fast runtime in pathological
//! cases to avoid quadratic time complexity and closely matches the behaviour of gnu-diff and git.
//! The Histogram algorithm was originally ported from git but has been heavily optimized.
//! The **Histogram algorithm outperforms Myers diff** by 10% - 100% across a **wide variety of workloads**.
//!
//! Imara-diffs algorithms have been benchmarked over a wide variety of real-world code.
//! For example while comparing multiple different linux kernel it performs up to 30 times better than the `similar` crate:
#![cfg_attr(doc, doc=concat!("<img width=\"600\" class=\"figure\" src=\"data:image/svg+xml;base64,", include_str!("../plots/linux_comparison.svg.base64"), "\"></img>"))]
//!
//! # Api Overview
//!
//! Imara-diff provides the [`UnifiedDiffBuilder`](crate::UnifiedDiffBuilder) for building
//! a human-readable diff similar to the output of `git diff` or `diff -u`.
//! This makes building a tool similar to gnu diff easy:
//!
//! ```
//! use imara_diff::intern::InternedInput;
//! use imara_diff::{diff, Algorithm, UnifiedDiffBuilder};
//!
//! let before = r#"fn foo() -> Bar {
//!     let mut foo = 2;
//!     foo *= 50;
//!     println!("hello world")
//! }"#;
//!
//! let after = r#"// lorem ipsum
//! fn foo() -> Bar {
//!     let mut foo = 2;
//!     foo *= 50;
//!     println!("hello world");
//!     println!("{foo}");
//! }
//! // foo
//! "#;
//!
//! let input = InternedInput::new(before, after);
//! let diff = diff(Algorithm::Histogram, &input, UnifiedDiffBuilder::new(&input));
//! assert_eq!(
//!     diff,
//!     r#"@@ -1,5 +1,8 @@
//! +// lorem ipsum
//!  fn foo() -> Bar {
//!      let mut foo = 2;
//!      foo *= 50;
//! -    println!("hello world")
//! +    println!("hello world");
//! +    println!("{foo}");
//!  }
//! +// foo
//! "#
//! );
//! ```
//!
//! If you want to process the diff in some way you can provide your own implementation of [`Sink`](crate::sink::Sink).
//! For closures [`Sink`](crate::sink::Sink) is already implemented, so simple [`Sink`]s can be easily added:
//!
//! ```
//! use std::ops::Range;
//!
//! use imara_diff::intern::InternedInput;
//! use imara_diff::{diff, Algorithm, UnifiedDiffBuilder};
//!
//! let before = r#"fn foo() -> Bar {
//!     let mut foo = 2;
//!     foo *= 50;
//!     println!("hello world")
//! }"#;
//!
//! let after = r#"// lorem ipsum
//! fn foo() -> Bar {
//!     let mut foo = 2;
//!     foo *= 50;
//!     println!("hello world");
//!     println!("{foo}");
//! }
//! // foo
//! "#;
//!
//! let mut insertions = Vec::new();
//! let mut removals = Vec::new();
//! let mut replacements = Vec::new();
//!
//! let input = InternedInput::new(before, after);
//! let sink = |before: Range<u32>, after: Range<u32>| {
//!     let hunk_before: Vec<_> = input.before[before.start as usize..before.end as usize]
//!         .iter()
//!         .map(|&line| input.interner[line])
//!         .collect();
//!     let hunk_after: Vec<_> = input.after[after.start as usize..after.end as usize]
//!         .iter()
//!         .map(|&line| input.interner[line])
//!         .collect();
//!     if hunk_after.is_empty() {
//!         removals.push(hunk_before)
//!     } else if hunk_before.is_empty() {
//!         insertions.push(hunk_after)
//!     } else {
//!         replacements.push((hunk_before, hunk_after))
//!     }
//! };
//! let diff = diff(Algorithm::Histogram, &input, sink);
//! assert_eq!(&insertions, &[vec!["// lorem ipsum"], vec!["// foo"]]);
//! assert!(removals.is_empty());
//! assert_eq!(
//!     &replacements,
//!     &[(
//!         vec!["    println!(\"hello world\")"],
//!         vec!["    println!(\"hello world\");", "    println!(\"{foo}\");"]
//!     )]
//! );
//! ```
//!
//! For `&str` and `&[u8]` imara-diff will compute a line diff by default.
//! To perform diffs of different tokenizations and collections you can implement the [`TokenSource`](crate::intern::TokenSource) trait.
//! For example the imara-diff provides an alternative tokenziser for line-diffs that includes the line terminator in the line:
//!
//! ```
//! use imara_diff::intern::InternedInput;
//! use imara_diff::sink::Counter;
//! use imara_diff::sources::lines_with_terminator;
//! use imara_diff::{diff, Algorithm, UnifiedDiffBuilder};
//!
//! let before = "foo";
//! let after = "foo\n";
//!
//! let input = InternedInput::new(before, after);
//! let changes = diff(Algorithm::Histogram, &input, Counter::default());
//! assert_eq!(changes.insertions, 0);
//! assert_eq!(changes.removals, 0);
//!
//! let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
//! let changes = diff(Algorithm::Histogram, &input, Counter::default());
//! assert_eq!(changes.insertions, 1);
//! assert_eq!(changes.removals, 1);
//! ```

use std::hash::Hash;

#[cfg(feature = "unified_diff")]
pub use unified_diff::UnifiedDiffBuilder;

use crate::intern::{InternedInput, Token, TokenSource};
pub use crate::sink::Sink;
mod histogram;
pub mod intern;
mod myers;
pub mod sink;
pub mod sources;
#[cfg(feature = "unified_diff")]
mod unified_diff;
mod util;

#[cfg(test)]
mod tests;

/// `imara-diff` supports multiple different algorithms
/// for computing an edit sequence.
/// These algorithms have different performance and all produce different output.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Algorithm {
    /// A variation of the [`patience` diff algorithm described by Bram Cohen's blog post](https://bramcohen.livejournal.com/73318.html)
    /// that uses a histogram to find the least common LCS.
    /// Just like the `patience` diff algorithm, this algorithm usually produces
    /// more human readable output then myers algorithm.
    /// However compared to the `patience` diff algorithm (which is slower then myers algorithm),
    /// the Histogram algorithm performs much better.
    ///
    /// The implementation here was originally ported from `git` but has been significantly
    /// modified to improve performance.
    /// As a result it consistently **performs better then myers algorithm** (5%-100%) over
    /// a wide variety of test data. For example a benchmark of diffing linux kernel commits is shown below:
    #[cfg_attr(doc, doc=concat!("<img width=\"600\" class=\"figure\" src=\"data:image/svg+xml;base64,", include_str!("../plots/linux_speedup.svg.base64"), "\"></img>"))]
    ///
    /// For pathological subsequences that only contain highly repeating tokens (64+ occurrences)
    /// the algorithm falls back on Myers algorithm (with heuristics) to avoid quadratic behavior.
    ///
    /// Compared to Myers algorithm, the Histogram diff algorithm is more focused on providing
    /// human readable diffs instead of minimal diffs. In practice this means that the edit-sequences
    /// produced by the histogram diff are often longer then those produced by Myers algorithm.
    ///
    /// The heuristic used by the histogram diff does not work well for inputs with small (often repeated)
    /// tokens. For example **character diffs do not work well** as most (english) text is madeup of
    /// a fairly small set of characters. The `Histogram` algorithm will automatically these cases and
    /// fallback to Myers algorithm. However this detection has a nontrivial overhead, so
    /// if its known upfront that the sort of tokens is very small `Myers` algorithm should
    /// be used instead.
    Histogram,
    /// An implementation of the linear space variant of
    /// [Myers  `O((N+M)D)` algorithm](http://www.xmailserver.org/diff2.pdf).
    /// The algorithm is enhanced with preprocessing that removes
    /// tokens that don't occur in the other file at all.
    /// Furthermore two heuristics to the middle snake search are implemented
    /// that ensure reasonable runtime (mostly linear time complexity) even for large files.
    ///
    /// Due to the divide and conquer nature of the algorithm
    /// the edit sequenced produced are still fairly small even when the middle snake
    /// search is aborted by a heuristic.
    /// However, the produced edit sequences are not guaranteed to be fully minimal.
    /// If that property is vital to you, use the `MyersMinimal` algorithm instead.
    ///
    /// The implementation (including the preprocessing) are mostly
    /// ported from `git` and `gnu-diff` where Myers algorithm is used
    /// as the default diff algorithm.
    /// Therefore the used heuristics have been heavily battle tested and
    /// are known to behave well over a large variety of inputs
    Myers,
    /// Same as `Myers` but the early abort heuristics are disabled to guarantee
    /// a minimal edit sequence.
    /// This can mean significant slowdown in pathological cases.
    MyersMinimal,
}

impl Algorithm {
    #[cfg(test)]
    const ALL: [Self; 2] = [Algorithm::Histogram, Algorithm::Myers];
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::Histogram
    }
}

/// Computes an edit-script that transforms `input.before` into `input.after` using
/// the specified `algorithm`
/// The edit-script is passed to `sink.process_change` while it is produced.
pub fn diff<S: Sink, T: Eq + Hash>(
    algorithm: Algorithm,
    input: &InternedInput<T>,
    sink: S,
) -> S::Out {
    diff_with_tokens(
        algorithm,
        &input.before,
        &input.after,
        input.interner.num_tokens(),
        sink,
    )
}

/// Computes an edit-script that transforms `before` into `after` using
/// the specified `algorithm`
/// The edit-script is passed to `sink.process_change` while it is produced.
pub fn diff_with_tokens<S: Sink>(
    algorithm: Algorithm,
    before: &[Token],
    after: &[Token],
    num_tokens: u32,
    sink: S,
) -> S::Out {
    assert!(
        before.len() < i32::MAX as usize,
        "imara-diff only supports up to {} tokens",
        i32::MAX
    );
    assert!(
        after.len() < i32::MAX as usize,
        "imara-diff only supports up to {} tokens",
        i32::MAX
    );
    match algorithm {
        Algorithm::Histogram => histogram::diff(before, after, num_tokens, sink),
        Algorithm::Myers => myers::diff(before, after, num_tokens, sink, false),
        Algorithm::MyersMinimal => myers::diff(before, after, num_tokens, sink, true),
    }
}
