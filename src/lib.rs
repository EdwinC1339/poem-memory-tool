pub mod poem;
pub mod memory_tool;
pub mod comparison;
pub mod menu;

use anyhow::Result;

use crate::{memory_tool::blank_game, menu::start_menu};

pub fn run() -> Result<()> {
    let choices = start_menu()?;
    match choices.mode {
        memory_tool::GameMode::Blank => blank_game(choices.poem),
        memory_tool::GameMode::Practice => todo!(),
    }

    Ok(())
}