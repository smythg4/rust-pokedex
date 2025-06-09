use std::io;
use std::io::prelude::*;
use rust_pokedex::config::{Config, Mode};
use crossterm::{cursor, execute, event::{self, Event, KeyCode}, terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType}};

fn clean_input(text: &mut str) -> Vec<String> {
    text.split_whitespace().map(|v| v.to_lowercase()).collect()
}

fn enter_catch_mode(config: &mut Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.get_mode() == Some(Mode::PokemonListing) {

        enable_raw_mode()?;
        let mut selected = 0;

        loop {
            execute!(io::stdout(), cursor::MoveTo(0, 10), terminal::Clear(terminal::ClearType::All))?;

            println!("--- CATCH MODE --- \r\r");
            println!("Use ↑↓ arrows to navigate, Enter to select, Esc to exit\r");
            for (i, pokemon) in config.get_current_pokemon().iter().enumerate() {
                if i == selected {
                    println!("> {}\r", pokemon);
                } 
                else {
                    println!("  {}\r", pokemon);
                }
            }

            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Up => {
                        if selected > 0 { selected -= 1; }
                    },
                    KeyCode::Down => {
                        if selected < config.get_current_pokemon().len() - 1 { selected += 1; }
                    },
                    KeyCode::Enter => {
                        let pok_name = config.get_current_pokemon().get(selected).unwrap();
                        println!("Selected: {}\r", pok_name);
                        let command = config.get_commands().get("catch").unwrap();
                        (command.callback)(config, &vec![pok_name.to_string()])?;
                        break;
                    },
                    KeyCode::Esc => break,
                    _ => {},
                }
            }
        }
        disable_raw_mode()?;
    }
    else {
        println!("You haven't explored a region to find Pokemon...")
    }
    Ok(())
}

fn enter_explore_mode(config: &mut Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.get_mode() == Some(Mode::LocationListing) {
        println!("--- EXPLORE MODE --- ");
        println!("Use ↑↓ arrows to navigate, Enter to select, Esc to exit\r");

        enable_raw_mode()?;
        let mut selected = 0;

        loop {
            execute!(io::stdout(), cursor::MoveTo(0, 10), terminal::Clear(terminal::ClearType::All))?;

            println!(" --- EXPLORE MODE --- \r\r");
            println!("Use ↑↓ arrows to navigate, Enter to select, Esc to exit\r");
            for (i, location) in config.get_current_locations().iter().enumerate() {
                if i == selected {
                    println!("> {}\r", location.get_name());
                } 
                else {
                    println!("  {}\r", location.get_name());
                }
            }

            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Up => {
                        if selected > 0 { selected -= 1; }
                    },
                    KeyCode::Down => {
                        if selected < config.get_current_locations().len() - 1 { selected += 1; }
                    },
                    KeyCode::Enter => {
                        let loc_name = config.get_current_locations()[selected].get_name();
                        println!("Selected: {}\r", loc_name);
                        let command = config.get_commands().get("explore").unwrap();
                        (command.callback)(config, &vec![loc_name.to_string()])?;
                        break;
                    },
                    KeyCode::Esc => break,
                    _ => {},
                }
            }
        }
        disable_raw_mode()?;
    }
    else {
        println!("You haven't mapped regions to explore...")
    }
    Ok(())
}

fn main() {
    let mut config = Config::new();
    loop {
        print!("Pokedex > ");
        if let Err(e) = io::stdout().flush(){// makes prompt appear immediately
            println!("Error flushing stdout: {}", e);
            continue;
        } 

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("Error reading line from stdin: {}", e);
            continue;
        }

        let args = clean_input(&mut input);
        if let Some(command_string) = args.first() {
            match command_string.as_str() {
                "e" => {
                    if let Err(e) = enter_explore_mode(&mut config) {
                        println!("error entering explore mode: {}", e);
                    }
                    continue;
                },
                "c" => {
                    if let Err(e) = enter_catch_mode(&mut config) {
                        println!("error entering catch mode: {}", e);
                    }
                    continue;
                },
                _ => {
                    // continue to normal mode
                }
            }
            let params = args[1..].to_vec();

            if let Some(command) = config.get_commands().get(command_string) {
                if let Err(e) = (command.callback)(&mut config, &params) {
                    println!("Error using callback function for {}: {}", command_string, e);
                    continue;
                }
            }
            else {
                println!("Command: {} not found", command_string);
            }
        }
        else {
            println!("No command entered, try again.");
        }

    }
}
