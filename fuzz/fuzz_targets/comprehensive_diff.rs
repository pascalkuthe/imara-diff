#![no_main]

use imara_diff::{Algorithm, Diff, InternedInput};
use libfuzzer_sys::fuzz_target;

use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug)]
struct Input<'a> {
    before: &'a [u8],
    after: &'a [u8],
}

/// Tests all three diff algorithms (Myers, Histogram, MyersMinimal) with:
/// - Computing diffs on arbitrary string inputs
/// - Postprocessing with no heuristic and line heuristic
/// - Unified diff printing
/// - Basic queries (count_additions, count_removals, is_added, is_removed)
/// - Hunks iteration
fn do_fuzz(Input { before, after }: Input<'_>) {
    // Create interned input
    let input = InternedInput::new(before, after);

    // Test all three diff algorithms
    for algorithm in [
        Algorithm::Histogram,
        Algorithm::Myers,
        Algorithm::MyersMinimal,
    ] {
        // Compute diff
        let mut diff = Diff::compute(algorithm, &input);

        // Test basic queries
        let _ = diff.count_additions();
        let _ = diff.count_removals();

        // Test hunks iteration
        for hunk in diff.hunks() {
            let _ = hunk.is_pure_insertion();
            let _ = hunk.is_pure_removal();
            let _ = hunk.invert();
        }

        // Test postprocessing with no heuristic
        diff.postprocess_no_heuristic(&input);

        // Test postprocessing with line heuristic
        diff.postprocess_lines(&input);
    }
}

fuzz_target!(|input: Input<'_>| {
    do_fuzz(input);
});
