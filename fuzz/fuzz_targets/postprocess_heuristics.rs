#![no_main]

use libfuzzer_sys::fuzz_target;
use imara_diff::{Algorithm, Diff, InternedInput, IndentHeuristic, IndentLevel};

fuzz_target!(|data: &[u8]| {
    // Test postprocessing with various heuristics
    if data.is_empty() {
        return;
    }
    
    // Split input into before and after
    let split_point = if data.len() > 1 {
        (data[0] as usize % data.len()).min(data.len() - 1)
    } else {
        0
    };
    
    let before_bytes = &data[..split_point];
    let after_bytes = &data[split_point..];
    
    let before = String::from_utf8_lossy(before_bytes);
    let after = String::from_utf8_lossy(after_bytes);
    
    let input = InternedInput::new(before.as_ref(), after.as_ref());
    
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
                IndentLevel::for_ascii_line(input.interner[token].as_bytes().iter().copied(), 4)
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
});
