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
