#![no_main]

use imara_diff::{Algorithm, Diff, IndentHeuristic, IndentLevel, InternedInput};
use libfuzzer_sys::fuzz_target;

use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug)]
struct Input<'a> {
    before: &'a str,
    after: &'a str,
    ident_level: u8,
}

/// Tests postprocessing with different heuristics:
/// - No heuristic
/// - Line heuristic (default indent-based)
/// - Custom indent heuristic with different tab sizes
/// - Validates hunk ranges are valid after postprocessing
fn do_fuzz(
    Input {
        before,
        after,
        ident_level,
    }: Input<'_>,
) {
    let input = InternedInput::new(before, after);

    // Test with different algorithms
    for algorithm in [Algorithm::Histogram, Algorithm::Myers] {
        let mut diff = Diff::compute(algorithm, &input);

        // Test postprocess with no heuristic
        diff.postprocess_no_heuristic(&input);
        let _ = diff.count_additions();
        let _ = diff.count_removals();

        // Test postprocess with line heuristic
        let mut diff2 = Diff::compute(algorithm, &input);
        diff2.postprocess_lines(&input);
        let _ = diff2.count_additions();
        let _ = diff2.count_removals();

        // Test postprocess with custom indent heuristic
        let mut diff3 = Diff::compute(algorithm, &input);
        diff3.postprocess_with_heuristic(
            &input,
            IndentHeuristic::new(|token| {
                IndentLevel::for_ascii_line(
                    input.interner[token].as_bytes().iter().copied(),
                    ident_level,
                )
            }),
        );
        let _ = diff3.count_additions();
        let _ = diff3.count_removals();

        // Verify hunks are valid after postprocessing
        for hunk in diff.hunks() {
            assert!(hunk.before.start <= hunk.before.end);
            assert!(hunk.after.start <= hunk.after.end);
        }
    }
}

fuzz_target!(|input: Input<'_>| {
    do_fuzz(input);
});
