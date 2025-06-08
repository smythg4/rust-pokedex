
use crate::cache::*;
use std::collections::HashMap;
use std::time::Duration;
use std::io::Read;
use serde::{Deserialize, Serialize};
use rand::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct LocationAreas {
    count: usize,
    next: String,
    previous: Option<String>,
    results: Vec<LocationArea>, 
}

#[derive(Serialize, Deserialize, Debug)]
struct LocationArea {
    name: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct LocationAreaDetail {
    pokemon_encounters: Vec<PokemonEncounter>,
}

#[derive(Deserialize, Debug)]
struct PokemonEncounter {
    pokemon: PokemonReference,
}

#[derive(Deserialize, Debug)]
struct PokemonReference {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Stat {
    stat: StatName,
    base_stat: u32,
}

#[derive(Deserialize, Debug)]
struct StatName {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Type {
    #[serde(rename = "type")]
    type_info: TypeName,
}

#[derive(Deserialize, Debug)]
struct TypeName {
    name: String,
}

#[derive(Deserialize, Debug)]
struct PokemonDetails {
    name: String,
    base_experience: u32,
    height: u32,
    weight: u32,
    stats: Vec<Stat>,
    types: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct CliCommand {
    name: String,
    description: String,
    pub callback: fn(&mut Config, &Vec<String>) -> Result<(), Box<dyn std::error::Error>>,
}

pub struct Config {
    next: Option<String>,
    previous: Option<String>,
    current: Option<String>,
    command_registry: HashMap<String, CliCommand>,
    cache: Cache,
    pokedex: HashMap<String, PokemonDetails>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            next: None,
            previous: None,
            current: None,
            command_registry: Config::command_registry(),
            cache: Cache::new(Duration::new(30,0)),
            pokedex: HashMap::new(),
        }
    }

    pub fn get_commands(&self) -> &HashMap<String, CliCommand> {
        &self.command_registry
    }

    fn command_registry() -> HashMap<String, CliCommand> {
        let mut registry = HashMap::new();
        registry.insert("exit".to_string(),CliCommand{
            name: "exit".to_string(),
            description: "Exit the Pokedex".to_string(),
            callback: command_exit});
        registry.insert("help".to_string(), CliCommand {
            name: "help".to_string(),
            description: "Displays a help message".to_string(),
            callback: command_help});
        registry.insert("map".to_string(), CliCommand {
            name: "map".to_string(),
            description: "Display the next 20 locations in the Pokemon world".to_string(),
            callback: command_map });
        registry.insert("mapb".to_string(), CliCommand {
            name: "mapb".to_string(),
            description: "Display the previous 20 locations in the Pokemon world".to_string(),
            callback: command_mapb });
        registry.insert("explore".to_string(), CliCommand{
            name: "explore".to_string(),
            description: "List the Pokemon found in this area".to_string(),
            callback: command_explore });
        registry.insert("catch".to_string(), CliCommand {
            name: "catch".to_string(),
            description: "Attempt to catch a Pokemon".to_string(),
            callback: command_catch });
        registry.insert("pokedex".to_string(), CliCommand {
           name: "pokedex".to_string(),
           description: "List the Pokemon you've caught".to_string(),
           callback: command_pokedex });
        registry.insert("inspect".to_string(), CliCommand {
            name: "inspect".to_string(),
            description: "Get details of a Pokemon you've caught".to_string(),
            callback: command_inspect });

        registry
    }
}

fn command_exit(_config: &mut Config, _params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Closing the Pokedex... Goodbye!");
    std::process::exit(0);
}

fn command_help(config: &mut Config, _params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to the Pokedex!");
    println!("Usage: \n");
    for (_, value) in config.get_commands() {
        println!("{}: {}", value.name, value.description)
    }
    Ok(())
}

fn command_map(config: &mut Config, _params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if config.next.is_none() {
        config.next = Some("https://pokeapi.co/api/v2/location-area/".to_string());
    }
    let url = config.next.clone().unwrap();

    let mut body = String::new();
    if let Some(cached) = config.cache.get_cache(&url) { 
        println!("Using cached data...");
        body = cached.data;
    }
    else {
        let mut res = reqwest::blocking::get(url.clone())?;
        res.read_to_string(&mut body).unwrap(); 
        config.cache.add_cache(&url, &body);
    }

    let la: LocationAreas = serde_json::from_str(&body).unwrap();

    config.current = Some(url);
    config.next = Some(la.next);
    config.previous = la.previous.map(|url| url.clone());

    for location in la.results {
        println!(" {}", location.name);
    }
    Ok(())
}

fn command_mapb(config: &mut Config, _params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if config.previous.is_none() {
        println!("You're on the first page");
        return Ok(());
    }
    let url = config.previous.clone().unwrap();    

    let mut body = String::new();
    if let Some(cached) = config.cache.get_cache(&url) { 
        println!("Using cached data...");
        body = cached.data;
    }
    else {
        let mut res = reqwest::blocking::get(url.clone())?;
        res.read_to_string(&mut body).unwrap(); 
        config.cache.add_cache(&url, &body);
    }
    
    let la: LocationAreas = serde_json::from_str(&body).unwrap();

    config.current = Some(url);
    config.next = Some(la.next);
    config.previous = la.previous.map(|url| url.clone());

    for location in la.results {
        println!(" {}", location.name);
    }
    Ok(())
}

fn command_explore(config: &mut Config, params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(loc) = params.first() {
        println!("Searching for Pokemon in {}", loc);

        let url = format!("{}{}",config.current.clone().unwrap(), loc);
        let mut body = String::new();
        if let Some(cached) = config.cache.get_cache(&url) { 
            println!("Using cached data...");
            body = cached.data;
        }
        else {
            let mut res = reqwest::blocking::get(url.clone())?;
            res.read_to_string(&mut body).unwrap(); 
            config.cache.add_cache(&url, &body);
        }

        let lad: LocationAreaDetail = serde_json::from_str(&body).unwrap();
        let pokemon: Vec<_> = lad.pokemon_encounters.iter().map(|pe| &pe.pokemon.name).collect();
        println!("Found Pokemon:");
        for p in pokemon {
            println!(" - {}", p);
        }
    }
    else {
        println!("No location entered to explore...")
    }
    Ok(())
}

fn command_catch(config: &mut Config, params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(pname) = params.first() {
        println!("Throwing a Pokeball at {}", pname);
        let base_url = "https://pokeapi.co/api/v2/pokemon/";  // PokeAPI Pokemon endpoint
        let full_url = format!("{}{}", base_url, pname);

        let mut body = String::new();
        if let Some(cached) = config.cache.get_cache(&full_url) {
            println!("Using cached data...");
            body = cached.data;
        }
        else {
            let mut res = reqwest::blocking::get(full_url.clone())?;
            res.read_to_string(&mut body).unwrap();
            config.cache.add_cache(&full_url, &body);
        }

        let pdetails: PokemonDetails = serde_json::from_str(&body).unwrap();

        let mut rng = rand::rng();
        let r = (0..350).choose(&mut rng).unwrap();
        if r > pdetails.base_experience {
            println!("You caught {}!", pdetails.name);
            config.pokedex.entry(pdetails.name.clone()).or_insert(pdetails);
        }
        else {
            println!("You failed to catch {}... {}, {}", pdetails.name, r, pdetails.base_experience);
        }
    }
    else {
        println!("No Pokemon name entered to try and catch...")
    }
    Ok(())
}

fn command_pokedex(config: &mut Config, _params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Your Pokedex contains: ");
    for (_, value) in &config.pokedex {
        println!(" - {} ({})", value.name, value.base_experience)
    }
    Ok(())
}

fn command_inspect(config: &mut Config, params: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(pname) = params.first() {
        match config.pokedex.get(pname) {
            Some(pdeets) => {
                println!("Name: {}", pdeets.name);
                println!("Height: {}", pdeets.height);
                println!("Weight: {}", pdeets.weight);
                println!("Stats:");
                for stat in &pdeets.stats {
                    println!("   -{}: {}", stat.stat.name, stat.base_stat);
                }
                println!("Types:");
                for type_info in &pdeets.types {
                    println!("   - {}", type_info.type_info.name);
                }
            },
            None => println!("You haven't caught {} yet", pname)
        }
    }
    else {
        println!("No Pokemon name entered to inspect...")
    }
    Ok(())
}