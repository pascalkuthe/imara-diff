use crate::{Algorithm, BasicLineDiffPrinter, Diff, InternedInput, UnifiedDiffConfig};
use expect_test::expect;

#[test]
fn myers_is_even() {
    let before = "a\nb\nx\nx\ny\n";
    let after = "b\na\nx\ny\nx\n";

    cov_mark::check!(EVEN_SPLIT);
    // if the check for is_odd incorrectly always true then we take a fastpath
    // when we shouldn't, which always leads to infinite iterations/recursion
    // still we check the number of iterations here in case the search
    // is buggy in more subtle ways
    cov_mark::check_count!(SPLIT_SEARCH_ITER, 15);
    let input = InternedInput::new(before, after);
    let diff = Diff::compute(Algorithm::Myers, &input);
    expect![[r#"
        @@ -1,5 +1,5 @@
        -a
         b
        -x
        +a
         x
         y
        +x
    "#]]
    .assert_eq(
        &diff
            .unified_diff(
                &BasicLineDiffPrinter(&input.interner),
                UnifiedDiffConfig::default(),
                &input,
            )
            .to_string(),
    );
}

#[test]
fn myers_is_odd() {
    let before = "a\nb\nx\ny\nx\n";
    let after = "b\na\nx\ny\n";

    cov_mark::check!(ODD_SPLIT);
    // if the check for odd doesn't work then
    // we still find the correct result but the number of search
    // iterations increases
    cov_mark::check_count!(SPLIT_SEARCH_ITER, 9);
    let input = InternedInput::new(before, after);
    let diff = Diff::compute(Algorithm::Myers, &input);
    expect![[r#"
        @@ -1,5 +1,4 @@
        -a
         b
        +a
         x
         y
        -x
    "#]]
    .assert_eq(
        &diff
            .unified_diff(
                &BasicLineDiffPrinter(&input.interner),
                UnifiedDiffConfig::default(),
                &input,
            )
            .to_string(),
    );
}

#[test]
fn cancel_unset_matches_compute() {
    // With a cancel flag that is never set, the cancellable path must produce
    // exactly the same edit script as the normal path.
    let before = "a\nb\nc\nd\ne\nf\n";
    let after = "a\nX\nc\nd\nY\nf\nZ\n";
    let input = InternedInput::new(before, after);
    let never = std::sync::atomic::AtomicBool::new(false);
    for algorithm in [
        Algorithm::Histogram,
        Algorithm::Myers,
        Algorithm::MyersMinimal,
    ] {
        let plain = Diff::compute(algorithm, &input);
        let cancellable = Diff::compute_cancellable(algorithm, &input, &never);
        assert_eq!(
            plain.count_removals(),
            cancellable.count_removals(),
            "{algorithm:?}: removals differ"
        );
        assert_eq!(
            plain.count_additions(),
            cancellable.count_additions(),
            "{algorithm:?}: additions differ"
        );
    }
}
