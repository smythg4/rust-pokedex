use std::io;
use std::io::prelude::*;
use rust_pokedex::config::Config;

fn clean_input(text: &mut str) -> Vec<String> {
    text.split_whitespace().map(|v| v.to_lowercase()).collect()
}

fn main() {
    let mut config = Config::new();
    loop {
        print!("Pokedex > ");
        io::stdout().flush().unwrap(); // Important: makes prompt appear immediately

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let args = clean_input(&mut input);
        let command_string = args.first().take().unwrap();
        let params = args[1..].to_vec();

        if let Some(command) = config.get_commands().get(command_string) {
            (command.callback)(&mut config, &params).unwrap();
        }
        else {
            println!("Command: {} not found", command_string);
        }
    }
}
