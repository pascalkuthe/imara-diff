use std::fs::read_to_string;
use std::mem::swap;
use std::path::PathBuf;

use expect_test::{expect, expect_file};

use crate::intern::InternedInput;
use crate::sink::Counter;
use crate::{diff, Algorithm, UnifiedDiffBuilder};

#[test]
fn replace() {
    let before = r#"fn foo() -> Bar{
    let mut foo = 2.0;
    foo *= 100 / 2;
    println!("hello world")        
}"#;

    let after = r#"const TEST: i32 = 0;
fn foo() -> Bar{
    let mut foo = 2.0;
    foo *= 100 / 2;
    println!("hello world");        
    println!("hello foo {TEST}");        
}
    
"#;
    let input = InternedInput::new(before, after);
    for algorithm in Algorithm::ALL {
        println!("{algorithm:?}");
        let diff = diff(algorithm, &input, UnifiedDiffBuilder::new(&input));
        expect![[r#"
            @@ -1,5 +1,8 @@
            +const TEST: i32 = 0;
             fn foo() -> Bar{
                 let mut foo = 2.0;
                 foo *= 100 / 2;
            -    println!("hello world")        
            +    println!("hello world");        
            +    println!("hello foo {TEST}");        
             }
            +    
        "#]]
        .assert_eq(&diff);
    }
}

#[test]
fn identical_files() {
    let file = r#"fn foo() -> Bar{
    let mut foo = 2.0;
    foo *= 100 / 2;
}"#;

    for algorithm in Algorithm::ALL {
        println!("{algorithm:?}");
        let input = InternedInput::new(file, file);
        let diff = diff(algorithm, &input, UnifiedDiffBuilder::new(&input));
        assert_eq!(diff, "");
    }
}

#[test]
fn simple_insert() {
    let before = r#"fn foo() -> Bar{
    let mut foo = 2.0;
    foo *= 100 / 2;
}"#;

    let after = r#"fn foo() -> Bar{
    let mut foo = 2.0;
    foo *= 100 / 2;
    println("hello world")
}"#;

    let mut input = InternedInput::new(before, after);
    for algorithm in Algorithm::ALL {
        println!("{algorithm:?}");
        let res = diff(algorithm, &input, UnifiedDiffBuilder::new(&input));
        expect![[r#"
          @@ -1,4 +1,5 @@
           fn foo() -> Bar{
               let mut foo = 2.0;
               foo *= 100 / 2;
          +    println("hello world")
           }
          "#]]
        .assert_eq(&res);

        swap(&mut input.before, &mut input.after);

        let res = diff(algorithm, &input, UnifiedDiffBuilder::new(&input));
        expect![[r#"
            @@ -1,5 +1,4 @@
             fn foo() -> Bar{
                 let mut foo = 2.0;
                 foo *= 100 / 2;
            -    println("hello world")
             }
            "#]]
        .assert_eq(&res);

        swap(&mut input.before, &mut input.after);
    }
}

pub fn project_root() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR");
    let mut res = PathBuf::from(dir);
    while !res.join("README.md").exists() {
        res = res
            .parent()
            .expect("reached fs root without finding project root")
            .to_owned()
    }
    res
}

#[test]
#[cfg(not(miri))]
fn hand_checked_udiffs() {
    for algorithm in Algorithm::ALL {
        println!("{algorithm:?}");
        let test_dir = project_root().join("tests");
        let file = "helix_syntax.rs";
        let path_before = test_dir.join(format!("{file}.before"));
        let path_after = test_dir.join(format!("{file}.after"));
        let path_diff = test_dir.join(format!("{file}.{algorithm:?}.diff"));
        let before = read_to_string(path_before).unwrap();
        let after = read_to_string(path_after).unwrap();
        let input = InternedInput::new(&*before, &*after);
        let diff = diff(algorithm, &input, UnifiedDiffBuilder::new(&input));
        expect_file![path_diff].assert_eq(&diff);
    }
}

#[test]
#[cfg(not(miri))]
fn complex_diffs() {
    for algorithm in Algorithm::ALL {
        println!("{algorithm:?}");
        let test_dir = project_root().join("tests");
        for (file1, file2) in [
            ("test1.json", "test2.json"),
            ("helix_syntax.rs.Histogram.diff", "helix_syntax.rs.after"),
        ] {
            let path_before = test_dir.join(file1);
            let path_diff = test_dir.join(file2);
            let before = read_to_string(path_before).unwrap();
            let after = read_to_string(path_diff).unwrap();
            let input = InternedInput::new(&*before, &*after);
            let res = diff(algorithm, &input, Counter::default());
            println!("{}", res.total())
        }
    }
}
