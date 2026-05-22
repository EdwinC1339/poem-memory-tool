use std::fmt::Display;
use std::io::{self, Write};

use crate::poem::Poem;
use crate::comparison::Comparison;

use unicode_segmentation::UnicodeSegmentation;

pub enum GameMode {
    Blank,
    Practice,
}

pub struct GameReport {
    score: i32,
}

impl Display for GameReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Score: {}", self.score)
    }
}

/// Create a comparison
fn line_comparison<'a>(poem_line: &'a str, user_line: &'a str) -> Comparison<'a> {
    Comparison::build(poem_line, user_line)
}

/// Given a line from the poem and a line from the user, generate a line of annotations.
/// The annotation is a ^ under every error from the user, or a * if there's multiple errors in one space (eg if the user forgot multiple characters)
fn annotate(comparison: &Comparison) -> String {
    let mut annotation = String::new();

    for edit in comparison.edits() {
        let spaces_to_add = edit.position_comparison() as isize - annotation.graphemes(true).count() as isize;
        
        if spaces_to_add >= 0 {
            annotation.push_str(&" ".repeat(spaces_to_add as usize));
            annotation.push('^');
        } else {
            annotation.pop();
            annotation.push('*');
        }
    }

    annotation
}

// TODO: scoring
pub fn blank_game(poem: Poem) -> GameReport {
    let mut scores = Vec::new();
    for (stanza_n, stanza) in poem.stanzas().iter().enumerate() {
        println!("Begin stanza {}", stanza_n + 1);
        for (line_n, line) in stanza.verses().iter().enumerate() {
            let line = line.as_str();

            print!("Line {:>2}:\t", line_n);
            io::stdout().flush().expect("couldn't flush stdout");

            let mut user_input= String::new();
            io::stdin().read_line(&mut user_input).expect("couldn't read input");
            let user_input = user_input.trim_end();

            let comparison = line_comparison(line, user_input);
            let annotation = annotate(&comparison);
            println!("Line {:>2}:\t{}", line_n, annotation);
            println!("Line {:>2}:\t{}", line_n, line);
            
            scores.push(comparison.score());
        }
    }

    let score = scores.iter().sum();

    GameReport { score }
}

pub fn practice_game(poem: Poem) -> GameReport {
    let mut scores = Vec::new();
    
    for (stanza_n, stanza) in poem.stanzas().iter().enumerate() {
        println!("Begin stanza {}", stanza_n + 1);
        for (line_n, line) in stanza.verses().iter().enumerate() {
            let line = line.as_str();

            println!("Line {:>2}:\t{}", line_n, line);
            
            print!("Line {:>2}:\t", line_n);
            io::stdout().flush().expect("couldn't flush stdout");

            let mut user_input= String::new();
            io::stdin().read_line(&mut user_input).expect("couldn't read input");
            let user_input = user_input.trim_end();

            let comparison = line_comparison(line, user_input);
            
            let score = comparison.score();
            scores.push(score);
        }
    }

    let score = scores.iter().sum();

    GameReport { score }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn annotate1() {
        let poem_line = "once upon a midnight dreary";
        let user_line = "once uon a midnight dreary";
        let comparison = line_comparison(poem_line, user_line);
        let annotation = annotate(&comparison);
        assert_eq!(&annotation, "      ^")
    }

    #[test]
    fn annotate2() {
        let poem_line = "once upon a midnight dreary";
        let user_line = "once uon midnight dreary";
        let comparison = line_comparison(poem_line, user_line);
        let annotation = annotate(&comparison);
        assert_eq!(&annotation, "      ^  *")
    }

    #[test]
    fn annotate3() {
        let poem_line = "Panem nostrum quotidianum da nobis hodie";
        let user_line = "panem quotidianum da nobis hodie";
        let expected  = "^     *";
        let comparison = line_comparison(poem_line, user_line);
        let annotation = annotate(&comparison);
        assert_eq!(&annotation, expected)
    }
}