use std::fs;

use poem_memory_tool::memory_tool::blank_game;
use poem_memory_tool::poem::Poem;

fn main() {
    let text = fs::read_to_string("./data/sample_poem.txt").unwrap();
    let poem = Poem::from(text.as_str());
    blank_game(poem);
}
