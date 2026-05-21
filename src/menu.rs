use std::io::{self, Write};
use std::fs::{self, DirEntry};
use std::path::Path;

use itertools::Itertools;

use crate::memory_tool::GameMode;
use crate::poem::{Poem};

fn scan_for_files(dir: &Path) -> io::Result<impl Iterator<Item=DirEntry>> {
    let entries = fs::read_dir(dir)?;
    Ok(entries.flat_map(|entry| {
        entry.ok().filter(|entry| entry.file_name().to_str().unwrap_or_default().ends_with(".txt"))
    }))
}

fn warn_user(max: usize) {
    println!("Please enter a valid number from 1 to {}.", max)
}

fn get_user_choice(max: usize) -> usize {
    let mut buffer = String::new();
    let mut done = false;
    while !done {
        print!("Enter your choice: ");
        
        io::stdout().flush().expect("couldn't flush stdout");
        buffer.clear();
        io::stdin().read_line(&mut buffer).expect("couldn't read stdin");
        
        buffer = buffer.trim().into();
        
        let parse = buffer.parse::<usize>();
        match parse {
            Ok(choice) if choice > max => warn_user(max),
            Err(_) => warn_user(max),
            Ok(_) => done = true,
        }
    }

    buffer.parse().expect("checked above")
}

pub struct StartMenuChoices {
    pub poem: Poem,
    pub mode: GameMode,
}

fn choose_poem() -> io::Result<Poem> {
    let dir = Path::new("./data/");
    let poem_files = scan_for_files(dir)?.collect_vec();
    println!("Please select a poem");
    for (i, entry) in poem_files.iter().enumerate() {
        let file_name = entry.file_name().into_string().unwrap_or("unrecognized file".into());
        println!("{} - {}", i + 1, file_name);
    }

    let choice = get_user_choice(poem_files.len());
    let file = &poem_files[choice - 1];
    let text = fs::read_to_string(file.path())?;
    let poem = Poem::from(text.as_str());

    Ok(poem)
}

fn choose_mode() -> io::Result<GameMode> {
    println!("Please select a game mode");
    println!("1 - Blank");
    println!("2 - Practice");
    let choice = get_user_choice(2);
    let mode = match choice {
        choice if choice == 1 => GameMode::Blank,
        choice if choice == 2 => GameMode::Practice,
        _ => panic!("Invalid choice match arm reached")
    };

    Ok(mode)
}

pub fn start_menu() -> io::Result<StartMenuChoices> {
    let poem = choose_poem()?;
    let mode = choose_mode()?;
    Ok(StartMenuChoices { poem, mode })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{collections::HashSet, ffi::OsString, hash::RandomState};

    #[test]
    fn scan_test() {
        let path = Path::new("./data/");
        let contents = scan_for_files(path).unwrap();
        let file_names: HashSet<OsString, RandomState> = HashSet::from_iter(contents.map(|file| file.file_name()));
        let target_name = OsString::from("sample_poem.txt");
        assert!(file_names.contains(&target_name));
    }
}