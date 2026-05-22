use std::{cmp::{Ordering, max}, iter};

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison<'a> {
    errors: Vec<Edit<'a>>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edit<'a> {
    position_ground_truth: usize,
    position_comparison: usize,
    do_what: EditDoWhat<'a>
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditDoWhat<'a> {
    Deletion(&'a str),
    Insertion(&'a str),
    Substitution{ground_truth: &'a str, comparison: &'a str},
}

impl<'a> Comparison<'a> {
    pub fn build(ground_truth: &'a str, to_compare: &'a str) -> Self {
        let errors = Self::lev(ground_truth, to_compare);
        
        Self {errors}
    }

    pub fn distance(&self) -> i32 {
        self.errors.len() as i32
    }

    pub fn edits(&self) -> impl Iterator<Item = &Edit<'_>> {
        self.errors.iter()
    }
    
    /// Edits are notated in the form "what must be done to b to make it like a"
    fn lev(a: &'a str, b: &'a str) -> Vec<Edit<'a>> {
        // algorithm from https://en.wikipedia.org/wiki/Levenshtein_distance#Iterative_with_full_matrix
        let a_graphemes = a.graphemes(true).collect_vec();
        let rows = a_graphemes.len() + 1;
        let b_graphemes = b.graphemes(true).collect_vec();
        let columns = b_graphemes.len() + 1;
        let mut distance_matrix = iter::repeat_n(
                iter::repeat_n(0, columns)
                .collect_vec(), rows
            ).collect_vec();
    
        for column_index in 1..columns {
            distance_matrix[0][column_index] = column_index;
        }
        
        for row_index in 1..rows {
            distance_matrix[row_index][0] = row_index;
        }
        
        for column_index in 1..columns {
            for row_index in 1..rows {
                let substitution_cost = match a_graphemes[row_index - 1] == b_graphemes[column_index - 1] {
                    true => 0,
                    false => 1,
                };

                distance_matrix[row_index][column_index] = *[
                    distance_matrix[row_index - 1][column_index] + 1,                       // deletion
                    distance_matrix[row_index][column_index - 1] + 1,                       // insertion
                    distance_matrix[row_index - 1][column_index-1] + substitution_cost,     // substitution/do nothing
                ].iter().min().expect("array has 3 elements")
            }
        }
        
        // Now it's necessary to backtrace how to get from b to a.
        // We will trace back each of the edits then add them to a stack.
        // We will traverse that stack again backwards to make the necessary adjustments for the positions of the edits in the comparison string.
        let mut edit_stack = Vec::new();
        let mut distance = distance_matrix[rows - 1][columns - 1];
        let mut row_index = rows - 1;
        let mut column_index = columns - 1;
        let mut backtrace = vec![(distance, row_index, column_index)];
        while distance != 0 {
            let mut options = Vec::new();
            
            if row_index > 0 && column_index > 0 {
                let substitution_distance = distance_matrix[row_index - 1][column_index - 1];
                options.push((substitution_distance, row_index - 1, column_index - 1));
            }

            if column_index > 0 {
                let deletion_distance = distance_matrix[row_index][column_index - 1];
                options.push((deletion_distance, row_index, column_index - 1));
            }

            if row_index > 0 {
                let insertion_distance = distance_matrix[row_index - 1][column_index];
                options.push((insertion_distance, row_index - 1, column_index));
            }

            (distance, row_index, column_index) = *options
                .iter()
                .min_by(|e1, e2| e1.0.cmp(&e2.0))
                .expect("options has at least 1 element");
            backtrace.push((distance, row_index, column_index))
        }

        let max_length = max(rows, columns) - 1;
        for (i, (pos1, pos2)) in backtrace.iter().zip(backtrace.iter().skip(1)).enumerate() {
            let a_grapheme = a_graphemes[pos1.1 - 1];
            if pos1.0 != pos2.0 {
                let edit_do_what = match (pos1.1.cmp(&pos2.1), pos1.2.cmp(&pos2.2)) {
                    (Ordering::Greater, Ordering::Greater) => {
                        let b_grapheme = b_graphemes[pos1.2 - 1];
                        EditDoWhat::Substitution { ground_truth: a_grapheme, comparison: b_grapheme }
                        
                    },
                    (Ordering::Greater, Ordering::Equal) => EditDoWhat::Insertion(a_grapheme),
                    (Ordering::Equal, Ordering::Greater) => EditDoWhat::Deletion(a_grapheme),
                    _ => panic!("There shouldn't be lesser comparisons, nor both equal!")
                };

                // To calculate the position of an edit in the ground truth, we can subtract the number of steps we're going down from the total length.
                // We adjust by -1 to account for the extra row we're adding in the matrix.
                let position_ground_truth = max_length.saturating_sub(i).saturating_sub(1);

                edit_stack.insert(0, (position_ground_truth, edit_do_what));
            }
        }

        let mut edits = Vec::new();
        let mut drift_adjustment: isize = 0;
        for (position_ground_truth, do_what) in edit_stack {
            // To calculate the position of the edit in the comparison string, we adjust the position in the ground truth by the drift adjustment.
            let position_comparison = (position_ground_truth as isize + drift_adjustment) as usize;

            // To calculate the drift adjustment, we increment it every step where there's a deletion, then decrement it with every insertion.
            match do_what {
                EditDoWhat::Deletion(_) => drift_adjustment += 1,
                EditDoWhat::Insertion(_) => drift_adjustment -= 1,
                EditDoWhat::Substitution{ground_truth: _, comparison: _} => (),
            }

            let edit = Edit {position_ground_truth, position_comparison, do_what};
            edits.push(edit);
        }

        edits
    }

    pub fn score(&self) -> i32 {
        self.errors.iter().map(|edit| edit.score()).sum()
    }
}

/// Try to strip away all the capitalization and diacritics from a grapheme to turn it into a 'neutral' char.
/// If the grapheme doesn't contain alphabetic chars, this returns None.
fn neutralize_grapheme(grapheme: &str) -> Option<char> {
    grapheme.nfd()
        .next()
        .map(|c|c.to_ascii_lowercase())
}

impl<'a> Edit<'a> {
    pub fn position_ground_truth(&self) -> usize {
        self.position_ground_truth
    }

    pub fn position_comparison(&self) -> usize {
        self.position_comparison
    }

    /// Gives the score for the edit.  
    /// Minor errors: 1 point
    /// - punctuation
    /// - whitespace
    /// - capitalization
    /// - diacritical marks
    /// Major errors: 5 points
    /// - Anything else
    pub fn score(&self) -> i32 {
        self.do_what.score()
    }
}

fn value(c: char) -> i32 {
    match c {
        c if c.is_whitespace() => 1,
        c if c.is_ascii_punctuation() => 1,
        _ => 5,
    }
}

impl<'a> EditDoWhat<'a> {
    pub fn score(&self) -> i32 {
        match self {
            EditDoWhat::Deletion(error) |
            EditDoWhat::Insertion(error) => {
                if let Some(c) = error.chars().next() {
                    value(c)
                } else {
                    eprintln!("ERROR: empty edit");
                    0
                }
            },
            EditDoWhat::Substitution { ground_truth, comparison } => {
                let ground_truth_neutral = neutralize_grapheme(ground_truth);
                let comparison_neutral = neutralize_grapheme(comparison);

                if ground_truth_neutral == comparison_neutral {
                    1
                } else if let Some(c) = ground_truth_neutral {
                    value(c)
                } else {
                    1
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn neutralize1() {
        assert_eq!(neutralize_grapheme("A"), neutralize_grapheme("a"));
    }

    #[test]
    fn neutralize2() {
        assert_eq!(neutralize_grapheme("é"), neutralize_grapheme("e"));
    }

    #[test]
    fn neutralize3() {
        assert_eq!(neutralize_grapheme("É"), neutralize_grapheme("e"));
    }
    
    #[test]
    fn distance1() {
        let s1 = "levenshtein";
        let s2 = "levenshtein";

        let comparison = Comparison::build(s1, s2);
        let distance = comparison.distance();
        assert_eq!(distance, 0);
    }

    #[test]
    fn distance2() {
        let s1 = "levenshtein";
        let s2 = "levenshtei";

        let comparison = Comparison::build(s1, s2);
        let distance = comparison.distance();
        assert_eq!(distance, 1);

    }

    #[test]
    fn substitution1() {
        let s1 = "levenshtein";
        let s2 = "leveNshtein";

        let comparison = Comparison::build(s1, s2);
        let edit = comparison.edits().next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Substitution { ground_truth: "n", comparison: "N" });
        assert_eq!(edit.position_ground_truth, 4);
    }

    #[test]
    fn substitution2() {
        let s1 = "levenshtein";
        let s2 = "leveNshtEin";

        let comparison = Comparison::build(s1, s2);
        let mut edits = comparison.edits();
        
        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Substitution { ground_truth: "n", comparison: "N" });
        assert_eq!(edit.position_ground_truth, 4);

        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Substitution { ground_truth: "e", comparison: "E" });
        assert_eq!(edit.position_ground_truth, 8);
    }

    #[test]
    fn insertion1() {
        let s1 = "levenshtein";
        let s2 = "leveshtein";

        let comparison = Comparison::build(s1, s2);
        let edit = comparison.edits().next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"n"));
        assert_eq!(edit.position_ground_truth, 4);
    }

    #[test]
    fn insertion2() {
        let s1 = "levenshtein";
        let s2 = "leveshtin";

        let comparison = Comparison::build(s1, s2);
        let mut edits = comparison.edits();
        
        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"n"));
        assert_eq!(edit.position_ground_truth, 4);

        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"e"));
        assert_eq!(edit.position_ground_truth, 8);
    }

    #[test]
    fn insertion3() {
        let s1 = "levenshtein";
        let s2 = "";

        let comparison = Comparison::build(s1, s2);
        let mut edits = comparison.edits();
        
        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"l"));
        assert_eq!(edit.position_ground_truth, 0);

        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"e"));
        assert_eq!(edit.position_ground_truth, 1);
    }

    #[test]
    fn insertion_score() {
        let s1 = "levenshtein";
        let s2 = "leenshtein";

        let comparison = Comparison::build(s1, s2);
        assert_eq!(comparison.score(), 5);
    }

    #[test]
    fn substitution_score() {
        let s1 = "levenshtein";
        let s2 = "le.enshtein";

        let comparison = Comparison::build(s1, s2);
        assert_eq!(comparison.score(), 5);
    }

    #[test]
    fn punctuation_score() {
        let s1 = "levenshtein.";
        let s2 = "levenshtein";

        let comparison = Comparison::build(s1, s2);
        assert_eq!(comparison.score(), 1);
    }

    #[test]
    fn score_capitalization() {
        let s1 = "Levenshtein";
        let s2 = "levenshtein";

        let comparison = Comparison::build(s1, s2);
        assert_eq!(comparison.score(), 1);
    }

    #[test]
    fn score_accent() {
        let s1 = "asención";
        let s2 = "asencion";

        let comparison = Comparison::build(s1, s2);
        assert_eq!(comparison.score(), 1);
    }
}