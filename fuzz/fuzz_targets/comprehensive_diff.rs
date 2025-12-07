#![no_main]

use libfuzzer_sys::fuzz_target;
use imara_diff::{Algorithm, Diff, InternedInput};

fuzz_target!(|data: &[u8]| {
    // Split the input data into two parts for before and after strings
    // We use a simple strategy: if data is empty, use empty strings
    // otherwise find a split point to create before/after
    
    if data.is_empty() {
        return;
    }
    
    // Use the first byte as a split point indicator (modulo the length)
    let split_point = if data.len() > 1 {
        (data[0] as usize % data.len()).min(data.len() - 1)
    } else {
        0
    };
    
    let before_bytes = &data[..split_point];
    let after_bytes = &data[split_point..];
    
    // Convert to strings, replacing invalid UTF-8 with replacement character
    let before = String::from_utf8_lossy(before_bytes);
    let after = String::from_utf8_lossy(after_bytes);
    
    // Create interned input
    let input = InternedInput::new(before.as_ref(), after.as_ref());
    
    // Test all three diff algorithms
    for algorithm in [Algorithm::Histogram, Algorithm::Myers, Algorithm::MyersMinimal] {
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
        
        // Test unified diff printing
        {
            use imara_diff::{BasicLineDiffPrinter, UnifiedDiffConfig};
            let printer = BasicLineDiffPrinter(&input.interner);
            let config = UnifiedDiffConfig::default();
            let unified = diff.unified_diff(&printer, config, &input);
            let _ = unified.to_string();
        }
    }
});
