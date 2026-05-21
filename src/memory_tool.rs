use std::io::{self, Write};

use crate::poem::Poem;
use crate::comparison::Comparison;

use unicode_segmentation::UnicodeSegmentation;

pub enum GameMode {
    Blank,
    Practice,
}

/// Create a comparison
fn line_comparison<'a>(poem_line: &'a str, user_line: &'a str) -> Comparison<'a> {
    Comparison::build(poem_line, user_line)
}

/// Given a line from the poem and a line from the user, generate a line of annotations.
/// The annotation is a ^ under every error from the user, or a * if there's multiple errors in one space (eg if the user forgot multiple characters)
fn annotate(poem_line: &str, user_line: &str) -> String {
    let comparison = line_comparison(poem_line, user_line);
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

pub fn blank_game(poem: Poem) {
    for (stanza_n, stanza) in poem.stanzas().iter().enumerate() {
        println!("Begin stanza {}", stanza_n + 1);
        for (line_n, line) in stanza.verses().iter().enumerate() {
            let line = line.as_str();

            print!("Line {:>2}:\t", line_n);
            io::stdout().flush().expect("couldn't flush stdout");

            let mut user_input= String::new();
            io::stdin().read_line(&mut user_input).expect("couldn't read input");
            let user_input = user_input.trim_end();

            let annotation = annotate(line, user_input);
            println!("Line {:>2}:\t{}", line_n, annotation);
            println!("Line {:>2}:\t{}", line_n, line);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn annotate1() {
        let poem_line = "once upon a midnight dreary";
        let user_line = "once uon a midnight dreary";
        let annotation = annotate(poem_line, user_line);
        assert_eq!(&annotation, "      ^")
    }

    #[test]
    fn annotate2() {
        let poem_line = "once upon a midnight dreary";
        let user_line = "once uon midnight dreary";
        let annotation = annotate(poem_line, user_line);
        assert_eq!(&annotation, "      ^  *")
    }

    #[test]
    fn annotate3() {
        let poem_line = "Panem nostrum quotidianum da nobis hodie";
        let user_line = "panem quotidianum da nobis hodie";
        let expected  = "^     *";
        let annotation = annotate(poem_line, user_line);
        assert_eq!(&annotation, expected)
    }
}