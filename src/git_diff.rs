use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Range;

use crate::intern::{InternedInput, Interner, Token};
use crate::Sink;

// Explanation for the following numbers can be found here:
// https://github.com/git/git/blob/324fbaab88126196bd42e7fa383ee94e165d61b5/xdiff/xdiffi.c#L535
const MAX_INDENT: u8 = 200;
const MAX_BLANKS: i16 = 20;
const INDENT_WEIGHT: i16 = 60;
const INDENT_HEURISTIC_MAX_SLIDING: usize = 100;

const START_OF_FILE_PENALTY: i16 = 1;
const END_OF_FILE_PENALTY: i16 = 21;
const TOTAL_BLANK_WEIGHT: i16 = -30;
const POST_BLANK_WEIGHT: i16 = 6;
const RELATIVE_INDENT_PENALTY: i16 = -4;
const RELATIVE_INDENT_WITH_BLANK_PENALTY: i16 = 10;
const RELATIVE_OUTDENT_PENALTY: i16 = 24;
const RELATIVE_OUTDENT_WITH_BLANK_PENALTY: i16 = 17;
const RELATIVE_DEDENT_PENALTY: i16 = 23;
const RELATIVE_DEDENT_WITH_BLANK_PENALTY: i16 = 17;

#[derive(PartialEq, Debug)]
enum ChangeType {
    Added,
    RemovedAbove,
    RemovedBelow,
    Modified,
}

#[derive(PartialEq)]
struct SplitScore {
    effective_indent: i16,
    penalty: i16,
}

impl PartialOrd for SplitScore {
    // A score is considered "Greater" if it is equal or less than 0
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let indent_penalty = match self.effective_indent.cmp(&other.effective_indent) {
            Ordering::Greater => INDENT_WEIGHT,
            Ordering::Less => -INDENT_WEIGHT,
            Ordering::Equal => 0,
        };

        Some(
            (indent_penalty + (self.penalty - other.penalty))
                .cmp(&0)
                .reverse(),
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct ChangeGroup {
    before: Range<usize>,
    after: Range<usize>,
    change_type: ChangeType,
}

/// A [`Sink`] that creates a diff like git would
pub struct GitDiff<'a, T>
where
    T: Display,
{
    before: &'a [Token],
    after: &'a [Token],
    interner: &'a Interner<T>,
    changes: Vec<ChangeGroup>,
}

// Calculate the indentation of a single line
fn get_indent(s: String) -> Option<u8> {
    let mut indent = 0;

    for char in s.chars() {
        if !char.is_whitespace() {
            return Some(indent);
        } else if char == ' ' {
            indent += 1;
        } else if char == '\t' {
            indent += 8 - indent % 8;
        }

        if indent >= MAX_INDENT {
            return Some(MAX_INDENT);
        }
    }

    None
}

fn measure_and_score_change<T: Display>(
    lines: &[Token],
    split: usize,
    interner: &Interner<T>,
    score: &mut SplitScore,
) {
    // Gather information about the surroundings of the change
    let end_of_file = split >= lines.len();
    let mut indent: Option<u8> = if split >= lines.len() {
        None
    } else {
        get_indent(interner[lines[split]].to_string())
    };
    let mut pre_blank = 0;
    let mut pre_indent: Option<u8> = None;
    let mut post_blank = 0;
    let mut post_indent: Option<u8> = None;

    for line in (0..=split.saturating_sub(1)).rev() {
        pre_indent = get_indent(interner[lines[line]].to_string());
        if pre_indent.is_none() {
            pre_blank += 1;
            if pre_blank == MAX_BLANKS {
                pre_indent = Some(0);
                break;
            }
        }
    }
    for line in split + 1..lines.len() {
        post_indent = get_indent(interner[lines[line]].to_string());
        if post_indent.is_none() {
            post_blank += 1;
            if post_blank == MAX_BLANKS {
                post_indent = Some(0);
                break;
            }
        }
    }

    // Calculate score of the currently applied split
    post_blank = if indent.is_none() { 1 + post_blank } else { 0 };
    let total_blank = pre_blank + post_blank;
    if indent.is_none() {
        indent = post_indent;
    }
    let any_blanks = total_blank != 0;

    if pre_indent.is_none() && pre_blank == 0 {
        score.penalty += START_OF_FILE_PENALTY;
    }

    if end_of_file {
        score.penalty += END_OF_FILE_PENALTY;
    }

    score.penalty += TOTAL_BLANK_WEIGHT * total_blank;
    score.penalty += POST_BLANK_WEIGHT * post_blank;

    score.effective_indent += if let Some(indent) = indent {
        indent as i16
    } else {
        -1
    };

    if indent.is_none() || pre_indent.is_none() || indent == pre_indent {
    } else if indent > pre_indent {
        score.penalty += if any_blanks {
            RELATIVE_INDENT_WITH_BLANK_PENALTY
        } else {
            RELATIVE_INDENT_PENALTY
        };
    } else if post_indent.is_some() && post_indent > indent {
        score.penalty += if any_blanks {
            RELATIVE_OUTDENT_WITH_BLANK_PENALTY
        } else {
            RELATIVE_OUTDENT_PENALTY
        };
    } else {
        score.penalty += if any_blanks {
            RELATIVE_DEDENT_WITH_BLANK_PENALTY
        } else {
            RELATIVE_DEDENT_PENALTY
        };
    }
}

impl<'a, T> GitDiff<'a, T>
where
    T: Display,
{
    pub fn new(input: &'a InternedInput<T>) -> Self {
        Self {
            before: &input.before,
            after: &input.after,
            interner: &input.interner,
            changes: Vec::new(),
        }
    }
}

impl<T> Sink for GitDiff<'_, T>
where
    T: Display,
{
    type Out = Vec<ChangeGroup>;

    fn process_change(&mut self, before: Range<u32>, after: Range<u32>) {
        if before.is_empty() && !after.is_empty() {
            self.changes.push(ChangeGroup {
                before: before.start as usize..before.end as usize,
                after: after.start as usize..after.end as usize,
                change_type: ChangeType::Added,
            });
        } else if after.is_empty() && !before.is_empty() {
            if after.start == 0 {
                self.changes.push(ChangeGroup {
                    before: before.start as usize..before.end as usize,
                    after: after.start as usize..after.end as usize,
                    change_type: ChangeType::RemovedAbove,
                });
            } else {
                self.changes.push(ChangeGroup {
                    before: before.start as usize..before.end as usize,
                    after: after.start as usize..after.end as usize,
                    change_type: ChangeType::RemovedBelow,
                });
            }
        } else {
            self.changes.push(ChangeGroup {
                before: before.start as usize..before.end as usize,
                after: after.start as usize..after.end as usize,
                change_type: ChangeType::Modified,
            });
        };
    }

    fn finish(mut self) -> Self::Out {
        if self.changes.is_empty() {
            return self.changes;
        }
        let mut shift: usize;

        for change in self.changes.iter_mut() {
            // Skip one liner changes
            if change.after.is_empty() {
                continue;
            }

            // Move this change up by one line if the line before the change and the last line in
            // the change are equal
            loop {
                if change.after.start > 0
                    && self.after[change.after.start - 1] == self.after[change.after.end - 1]
                {
                    change.after.start -= 1;
                    change.after.end -= 1;
                } else {
                    break;
                }
            }

            shift = change.after.end;

            // Move this change down by one line if the first line in the change the line after the
            // change are equal
            loop {
                if change.after.end < self.after.len()
                    && self.after[change.after.start] == self.after[change.after.end]
                {
                    change.after.start += 1;
                    change.after.end += 1;
                } else {
                    break;
                }
            }

            let mut best_shift: Option<usize> = None;
            let mut best_score = SplitScore {
                effective_indent: 0,
                penalty: 0,
            };

            if change.after.end.saturating_sub(change.after.len()) > shift {
                shift = change.after.end - change.after.len();
            }

            if change
                .after
                .end
                .saturating_sub(INDENT_HEURISTIC_MAX_SLIDING)
                > shift
            {
                shift = change.after.end - INDENT_HEURISTIC_MAX_SLIDING;
            }

            while shift <= change.after.end {
                let mut score = SplitScore {
                    effective_indent: 0,
                    penalty: 0,
                };

                measure_and_score_change(self.after, shift, self.interner, &mut score);
                measure_and_score_change(
                    self.after,
                    shift - change.after.len(),
                    self.interner,
                    &mut score,
                );

                if best_shift.is_none() || score > best_score {
                    best_score = score;
                    best_shift = Some(shift);
                }
                shift += 1;
            }

            if let Some(best_shift) = best_shift {
                while change.after.end > best_shift {
                    loop {
                        if change.after.start > 0
                            && self.after[change.after.start - 1]
                                == self.after[change.after.end - 1]
                        {
                            change.after.start -= 1;
                            change.after.end -= 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        self.changes
    }
}

#[test]
fn git_diff_test() {
    let before = r#"struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    // width and height of a rectangle can be different
    let rect1 = Rectangle { width: 30, height: 50 };

    println!(
        "The area of the rectangle is {} square pixels.",
        area(&rect1)
    );
}

fn area(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}
"#;

    let after = r#"/// A rectangle. First line is changed to prevent a regression of #1869
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };

    println!(
        "The perimeter of the rectangle is {} pixels.",
        perimeter(&rect1)
    );
    println!("This line contains invalid utf8";
}

fn area(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}

fn perimeter(rectangle: &Rectangle) -> u32 {
    (rectangle.width + rectangle.height) * 2
}
"#;
    use crate::git_diff::ChangeType;

    let input = InternedInput::new(before, after);
    let diff = crate::diff(crate::Algorithm::Histogram, &input, GitDiff::new(&input));
    assert_eq!(
        diff,
        vec![
            ChangeGroup {
                before: 0..0,
                after: 0..1,
                change_type: ChangeType::Added
            },
            ChangeGroup {
                before: 6..7,
                after: 7..7,
                change_type: ChangeType::RemovedBelow
            },
            ChangeGroup {
                before: 10..12,
                after: 10..12,
                change_type: ChangeType::Modified
            },
            ChangeGroup {
                before: 13..13,
                after: 13..14,
                change_type: ChangeType::Added
            },
            ChangeGroup {
                before: 17..17,
                after: 19..23,
                change_type: ChangeType::Added
            }
        ]
    );
}
