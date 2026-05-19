use std::{cmp::{Ordering, max}, iter};

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison<'a> {
    errors: Vec<Edit<'a>>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edit<'a> {
    position: usize,
    do_what: EditDoWhat<'a>
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditDoWhat<'a> {
    Deletion,
    Insertion(&'a str),
    Substitution(&'a str),
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
        let mut edits = Vec::new();
    
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
            if pos1.0 != pos2.0 {
                let edit_do_what = match (pos1.1.cmp(&pos2.1), pos1.2.cmp(&pos2.2)) {
                    (Ordering::Greater, Ordering::Greater) => EditDoWhat::Substitution(a_graphemes[pos1.1 - 1]),
                    (Ordering::Greater, Ordering::Equal) => EditDoWhat::Insertion(a_graphemes[pos1.1 - 1]),
                    (Ordering::Equal, Ordering::Greater) => EditDoWhat::Deletion,
                    _ => panic!("There shouldn't be lesser comparisons, nor both equal!")
                };

                edits.push(Edit { position: max_length - i - 1, do_what: edit_do_what });
            }
        }

        edits.reverse();
        edits
    }
}

impl<'a> Edit<'a> {
    pub fn position(&self) -> usize {
        self.position
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
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
        assert_eq!(edit.do_what, EditDoWhat::Substitution(&"n"));
        assert_eq!(edit.position, 4);
    }

    #[test]
    fn substitution2() {
        let s1 = "levenshtein";
        let s2 = "leveNshtEin";

        let comparison = Comparison::build(s1, s2);
        let mut edits = comparison.edits();
        
        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Substitution(&"n"));
        assert_eq!(edit.position, 4);

        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Substitution(&"e"));
        assert_eq!(edit.position, 8);
    }

    #[test]
    fn insertion1() {
        let s1 = "levenshtein";
        let s2 = "leveshtein";

        let comparison = Comparison::build(s1, s2);
        let edit = comparison.edits().next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"n"));
        assert_eq!(edit.position, 4);
    }

    #[test]
    fn insertion2() {
        let s1 = "levenshtein";
        let s2 = "leveshtin";

        let comparison = Comparison::build(s1, s2);
        let mut edits = comparison.edits();
        
        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"n"));
        assert_eq!(edit.position, 4);

        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"e"));
        assert_eq!(edit.position, 8);
    }

    #[test]
    fn insertion3() {
        let s1 = "levenshtein";
        let s2 = "";

        let comparison = Comparison::build(s1, s2);
        let mut edits = comparison.edits();
        
        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"l"));
        assert_eq!(edit.position, 0);

        let edit = edits.next().unwrap();
        assert_eq!(edit.do_what, EditDoWhat::Insertion(&"e"));
        assert_eq!(edit.position, 1);
    }
}