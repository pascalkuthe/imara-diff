#![no_main]

use libfuzzer_sys::fuzz_target;
use imara_diff::{Algorithm, Diff, InternedInput, BasicLineDiffPrinter, UnifiedDiffConfig};

/// Valid prefixes for unified diff output lines
const VALID_DIFF_LINE_PREFIXES: [char; 4] = [' ', '+', '-', '@'];

fuzz_target!(|data: &[u8]| {
    // Test unified diff printing extensively
    {
        if data.is_empty() {
            return;
        }
        
        // Split input into before, after, and context_len
        let split1 = if data.len() > 2 {
            (data[0] as usize % data.len()).min(data.len() - 1)
        } else {
            0
        };
        
        let split2 = if data.len() > split1 + 1 {
            split1 + ((data[split1] as usize % (data.len() - split1)).max(1))
        } else {
            data.len()
        };
        
        let before_bytes = &data[..split1];
        let after_bytes = &data[split1..split2];
        
        // Use remaining byte for context_len (0-10)
        let context_len = if split2 < data.len() {
            data[split2] as u32 % 11
        } else {
            3
        };
        
        let before = String::from_utf8_lossy(before_bytes);
        let after = String::from_utf8_lossy(after_bytes);
        
        let input = InternedInput::new(before.as_ref(), after.as_ref());
        
        // Test with different algorithms
        for algorithm in [Algorithm::Histogram, Algorithm::Myers, Algorithm::MyersMinimal] {
            let mut diff = Diff::compute(algorithm, &input);
            
            // Postprocess before printing
            diff.postprocess_lines(&input);
            
            // Create printer and config
            let printer = BasicLineDiffPrinter(&input.interner);
            let mut config = UnifiedDiffConfig::default();
            config.context_len(context_len);
            
            // Generate unified diff - should not panic
            let unified = diff.unified_diff(&printer, config, &input);
            let output = unified.to_string();
            
            // Basic sanity checks on output
            // It should be valid UTF-8 (already guaranteed by to_string)
            // Lines should start with valid diff prefixes
            for line in output.lines() {
                if !line.is_empty() {
                    let first_char = line.chars().next().unwrap();
                    // Should be a valid diff line prefix
                    assert!(
                        VALID_DIFF_LINE_PREFIXES.contains(&first_char),
                        "Invalid diff line prefix: '{}' in line: '{}'", first_char, line
                    );
                }
            }
        }
    }
});
