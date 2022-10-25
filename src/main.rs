#![allow(dead_code, unused_variables, unused_imports)]
extern crate indicatif;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
use indicatif::ProgressBar;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::{thread, time};

/* Genertions */
pub struct Generation {
    pokemon_start: i32,
    pokemon_end: i32,
}
static GEN1: Generation = Generation {
    pokemon_start: 1,
    pokemon_end: 151,
};
static GEN2: Generation = Generation {
    pokemon_start: 152,
    pokemon_end: 251,
};
static GEN3: Generation = Generation {
    pokemon_start: 252,
    pokemon_end: 386,
};
static GEN4: Generation = Generation {
    pokemon_start: 387,
    pokemon_end: 493,
};
static GEN5: Generation = Generation {
    pokemon_start: 494,
    pokemon_end: 649,
};
static GEN6: Generation = Generation {
    pokemon_start: 650,
    pokemon_end: 721,
};
static GEN7: Generation = Generation {
    pokemon_start: 722,
    pokemon_end: 809,
};
static GEN8: Generation = Generation {
    pokemon_start: 810,
    pokemon_end: 898,
};

/*
Pokémon Struct :
From the Pokedex folder (sorted by index)
 */
#[derive(Deserialize)]
pub struct Pokemon {
    species: PokeAPISpecies,
    id: i32,
    types: Vec<PokeAPITypes>,
    height: f32,
    weight: f32,
    abilities: Vec<PokeAPIAbilities>,
    stats: Vec<PokeAPIStats>,
    sprites: PokeAPISprites,
}
#[derive(Deserialize)]
struct PokeAPISpecies {
    name: String,
}
/* On utilise r#type car le mot type n'est pas utilisable comme nom de variable */
#[derive(Deserialize)]
pub struct PokeAPITypes {
    r#type: PokeAPIType,
}
/* Implémentation du trait Display pour PokeAPIType
pour éviter d'utiliser #[derive(Deserialize, Debug)]
qui, quand on le print avec {::?} donne :
Type : name {
    le type
}
 */
impl fmt::Display for PokeAPIType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
#[derive(Deserialize)]
pub struct PokeAPIType {
    name: String,
}
#[derive(Deserialize)]
struct PokeAPIAbilities {
    ability: PokeAPIAbility,
    is_hidden: bool,
}
#[derive(Deserialize)]
struct PokeAPIAbility {
    name: String,
}
#[derive(Deserialize)]
struct PokeAPIStats {
    stat: PokeAPIStat,
    base_stat: i32,
    effort: i32,
}
#[derive(Deserialize)]
struct PokeAPIStat {
    name: String,
}
#[derive(Deserialize)]
pub struct PokeAPISprites {
    front_default: String,
}

/* SimplePok Struct:
Used to sort pokemons by stat
 */
pub struct SimplePok {
    name: String,
    index: i32,
    stat_name: String,
    stat: i32,
}
impl SimplePok {
    pub fn new(name: String, index: i32, stat_name: String, stat: i32) -> Self {
        SimplePok {
            name,
            index,
            stat_name,
            stat,
        }
    }
}

/*
PokemonEvoInfo Struct :
Used to store the links of the evolution chains.
Json stored into the Evolutions folder (sorted by index).
*/
#[derive(Deserialize)]
pub struct PokemonEvoInfo {
    evolution_chain: PokeAPIChain,
}
#[derive(Deserialize)]
pub struct PokeAPIChain {
    url: String,
}

/* EvolutionChain Struct:
Used to store the informations about the evolution chains
Json stored in the EvolutionsChains folder (sorted by index).
*/
#[derive(Deserialize)]
pub struct EvolutionChain {
    id: i32,
    chain: PokeAPIChainLink,
}
#[derive(Deserialize)]
pub struct PokeAPIChainLink {
    species: NamedAPIResource,
    evolution_details: Vec<PokeAPIEvolutionDetail>,
    evolves_to: Vec<PokeAPIChainLink>,
}
/*
Toutes les variables sont en type Option<>
car elle peuvent être présente ou non dans le Json
*/
#[derive(Deserialize)]
pub struct PokeAPIEvolutionDetail {
    item: Option<NamedAPIResource>,
    trigger: Option<NamedAPIResource>,
    gender: Option<i32>,
    held_item: Option<NamedAPIResource>,
    known_move: Option<NamedAPIResource>,
    known_move_type: Option<NamedAPIResource>,
    location: Option<NamedAPIResource>,
    min_level: Option<i32>,
    min_happiness: Option<i32>,
    min_beauty: Option<i32>,
    min_affection: Option<i32>,
    needs_overworld_rain: Option<bool>,
    party_species: Option<NamedAPIResource>,
    party_type: Option<NamedAPIResource>,
    relative_physical_stats: Option<i32>,
    time_of_day: Option<String>,
    trade_species: Option<NamedAPIResource>,
    turn_upside_down: Option<bool>,
}
/*
Type frequently used in the Json's API.
Storing a name and a link to the API informations
 */
#[derive(Deserialize)]
pub struct NamedAPIResource {
    name: String,
    url: String,
}
impl fmt::Display for NamedAPIResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/*
Function get_pokemon_evochain :
Taking an index as argument
Returning a PokemonEvoInfo (used to store the link to the EvoChain)
*/
pub fn get_pokemon_evochain(index: i32) -> PokemonEvoInfo {
    let path = "Evolutions/";
    let json = ".json";
    let file_path = format!("{}{}{}", path, index, json);
    let mut file = match File::open(&file_path){
        Ok(t) => t,
        Err(e) => panic!("Problem opening the file: {:?}", e),
    }/* .unwrap() */;
    let mut buff = String::new();
    match file.read_to_string(&mut buff){
        Ok(t) => t,
        Err(e) => panic!("Problem reading the file: {:?}", e),
    }/* .unwrap() */;
    let pokemon_evo_info: PokemonEvoInfo = match serde_json::from_str(&buff) {
        Ok(t) => t,
        Err(e) => panic!("Problem parsing the data into the structure: {:?}", e),
    };
    pokemon_evo_info
}
/*
Function get_pokemon_evo_index :
Taking an index as argument
Returning a EvolutionChain (evolution details)
*/
pub fn get_pokemon_evo_index(str: String) -> Result<EvolutionChain, String> {
    if is_string_numeric(&str) {
        let index = string_to_i32(&str);
        let path = "EvolutionsChains/";
        let json = ".json";
        let file_path = format!("{}{}{}", path, index, json);
        let mut file = match File::open(&file_path) {
            Ok(t) => t,
            Err(e) => panic!("Problem opening the file: {:?}", e),
        };
        let mut buff = String::new();
        match file.read_to_string(&mut buff){
        Ok(t) => t,
        Err(e) => panic!("Problem reading the file: {:?}", e),
    }/* .unwrap() */;
        let pokemon_evo: EvolutionChain = match serde_json::from_str(&buff) {
            Ok(t) => t,
            Err(e) => panic!("Problem parsing the data into the structure: {:?}", e),
        };
        return Ok(pokemon_evo);
    } else if is_string_alphabetic(&str) {
        let mut x: i32 = 0;
        let mut pok;
        while x <= 898 {
            x += 1;
            pok = get_pokemon_data_i(x).expect("This pokemon doesn't exist");
            if str == pok.species.name {
                return Ok(get_pokemon_evo_index(x.to_string()).expect("Error"));
            }
        }
    }
    Err("This pokemon's evolution chain can't exist".to_string())
}

/*
Function print_evo_details :
Taking an Vec<PokeAPIEvolutionDetail> as argument
Printing the details of an evolution (can be multiple details)

Ex:
    eevee
    Evolves into : leafeon
    level-up
    Only in eterna-forest location

    level-up
    Only in pinwheel-forest location

    level-up
    Only in kalos-route-20 location

    use-item
    With the use of leaf-stone item
*/
pub fn print_evo_details(vec: &[PokeAPIEvolutionDetail]) {
    let rien = "".to_string(); //utilisé car "" ne peut être une valeur dans match (.to_string)
    for a in vec {
        match &a.trigger {
            Some(t) => println!("{}", t.name),
            None => print!(""),
        }
        match &a.item {
            Some(t) => println!("With the use of {} item", t.name),
            None => print!(""),
        }
        match &a.gender {
            Some(t) => println!("Needs to be {} (male/female)", t),
            None => print!(""),
        }
        match &a.held_item {
            Some(t) => println!("Needs to hold {} item ", t.name),
            None => print!(""),
        }
        match &a.known_move {
            Some(t) => println!("Needs to know {} move", t.name),
            None => print!(""),
        }
        match &a.known_move_type {
            Some(t) => println!("Needs to know {} move type", t.name),
            None => print!(""),
        }
        match &a.location {
            Some(t) => println!("Only in {} location", t.name),
            None => print!(""),
        }
        match &a.min_level {
            Some(t) => println!("Leveling up at the {} lvl", t),
            None => print!(""),
        }
        match &a.min_happiness {
            Some(t) => println!("Leveling up at the {} lvl of happiness", t),
            None => print!(""),
        }
        match &a.min_beauty {
            Some(t) => println!("Leveling up at the {} lvl of beauty", t),
            None => print!(""),
        }
        match &a.min_affection {
            Some(t) => println!("Leveling up at the {} lvl of affection", t),
            None => print!(""),
        }
        match &a.needs_overworld_rain {
            Some(t) => match t {
                true => println!("Needs overwold rain"),
                false => print!(""),
            },
            None => print!(""),
        }
        match &a.party_species {
            Some(t) => println!(
                "Pokémon species that must be in the players party : {}",
                t.name
            ),
            None => print!(""),
        }
        match &a.party_type {
            Some(t) => println!("Must have {} type in your party !", t.name),
            None => print!(""),
        }
        match &a.relative_physical_stats {
            Some(t) => match t {
                1 => println!("Needs to have a Att > Def ratio"),
                0 => println!("Needs to have a Att = Def ratio"),
                -1 => println!("Needs to have a Att < Def ratio"),
                _ => println!("Wrong ratio must be either -1, 0 or 1"),
            },
            None => print!(""),
        }
        match &a.time_of_day {
            Some(t) => match t {
                rien => print!(""), //je suis incapable de trouver un fix
                _ => println!("Required time of day : {}", t),
            },
            None => print!(""),
        }
        match &a.trade_species {
            Some(t) => println!(
                "Pokémon species for which this one must be traded : {}",
                t.name
            ),
            None => print!(""),
        }
        match &a.turn_upside_down {
            Some(t) => match t {
                true => println!("3DS needs to be turned upside-down"),
                false => print!(""),
            },
            None => print!(""),
        }
        println!();
    }
}

/*
Function print_evo :
Taking a PokeAPIChainLink as argument
Calling print_evo_details() multiple times,
to print the entire evolution chain and details.
Calls find_pokemon() to search and print the evolution pokemon details

Ex:
    eevee
    Evolves into : sylveon
    level-up
    Needs to know fairy move type
    Leveling up at the 2 lvl of affection

    Name: sylveon
    Index : 700
    Type : fairy,
    Height: 1 m
    Weight: 23.5 kg
    Abilities:
    cute-charm;    Hidden: false
    pixilate;      Hidden: true
    Stats:
    hp : 95;       Effort: 0
    attack : 65;   Effort: 0
    defense : 65;  Effort: 0
    special-attack : 110;  Effort: 0
    special-defense : 130; Effort: 2
    speed : 60;    Effort: 0
*/
pub fn print_evo(evo: &PokeAPIChainLink) {
    for x in &evo.evolves_to {
        println!("{}", evo.species.name);
        println!("Evolves into : {}", x.species);
        print_evo_details(&x.evolution_details);
        /* find_pokemon(&(*x.species.name).to_string()); */
        println!();
        print_evo(&evo.evolves_to[0]);
    }
}

/*
Function path_exists :
Taking a String as argument.
Returning 'true' if the input is a valid path
and 'false' if it isn't.
*/
pub fn path_exists(str: &str) -> bool {
    fs::metadata(str).is_ok()
}
/*
Function fill_pokedex :
Used to download or update the Pokedex folder.
iterating through all the pokemon (898).
If the file already exist delete it.
Copying the API json from a request.
Then create the file and copy the Json inside it.

This function is using a progressbar to see how the Dowload is doing.
*/
fn fill_pokedex() {
    let mut x = 1;
    let bar = ProgressBar::new(898);
    while x <= 898 {
        let path = "Pokedex/";
        let json = ".json";
        let save_file_path = format!("{}{}{}", path, x, json).to_owned();
        let url = "https://pokeapi.co/api/v2/pokemon/";
        if path_exists(&save_file_path) {
            fs::remove_file(&save_file_path).expect("File do not exist");
        }
        let download_url = format!("{}{}", url, x).to_owned();
        let resp = reqwest::blocking::get(download_url).expect("Request failed");
        let body = resp.bytes().expect("Body invalid");
        let mut out = File::create(save_file_path).expect("Failed to create file");
        let body_bytes = body.to_vec();
        io::copy(&mut &body_bytes[..], &mut out).expect("Failed to copy content");

        bar.inc(1);
        x += 1;
    }
    bar.finish();
    println!("\nDownload completed !\n");
}

/*
Function fill_evolution :
Used to download or update the Evolutions folder.
iterating through all the pokemon (898).
If the file already exist delete it.
Copying the API json from a request (url created by formating the known url and the pokemon index).
Then create the file and copy the Json inside it.

This function is using a progressbar to see how the Dowload is doing.
*/
fn fill_evolution() {
    let mut x = 1;
    let bar = ProgressBar::new(898);
    while x <= 898 {
        let path = "Evolutions/";
        let json = ".json";
        let save_file_path = format!("{}{}{}", path, x, json).to_owned();
        let url = "https://pokeapi.co/api/v2/pokemon-species/";
        if path_exists(&save_file_path) {
            fs::remove_file(&save_file_path).expect("File do not exist");
        }
        let download_url = format!("{}{}{}", url, x, "/").to_owned();
        let resp = reqwest::blocking::get(download_url).expect("Request failed");
        let body = resp.bytes().expect("Body invalid");
        let mut out = File::create(save_file_path).expect("Failed to create file");
        let body_bytes = body.to_vec();
        io::copy(&mut &body_bytes[..], &mut out).expect("Failed to copy content");

        bar.inc(1);
        x += 1;
    }
    bar.finish();
    println!("\nDownload completed !\n");
}

/*
Function fill_evolution_chains :
Taking a PokemonEvoInfo as argument.
Used to download or update the Evolutions folder.
Iterating through all the pokemon (898).
If the file already exist delete it.
Copying the API json from a request (url stored in the PokemonEvoInfo).
Then create the file and copy the Json inside it.

This function is using a progressbar to see how the Dowload is doing.
*/
fn fill_evolution_chains() {
    println!("\nDo not cancel the download, doing so might cause some troubles !");
    let mut x = 1;
    let bar = ProgressBar::new(898);
    let mut info;
    while x <= 898 {
        info = get_pokemon_evochain(x);
        let path = "EvolutionsChains/";
        let json = ".json";
        let save_file_path = format!("{}{}{}", path, x, json);
        if path_exists(&save_file_path) {
            fs::remove_file(&save_file_path).expect("File do not exist");
        }
        let download_url = &info.evolution_chain.url;
        let resp = reqwest::blocking::get(download_url).expect("Request failed");
        let body = resp.bytes().expect("Body invalid");
        let mut out = File::create(save_file_path).expect("Failed to create file");
        let body_bytes = body.to_vec();
        io::copy(&mut &body_bytes[..], &mut out).expect("Failed to copy content");

        x += 1;
        bar.inc(1);
    }
    bar.finish();
    println!("\nDownload completed !\n");

    /* let path = "EvolutionsChains/";
    let json = ".json";
    let save_file_path = format!("{}{}{}", path, index, json);
    if path_exists(&save_file_path) {
        fs::remove_file(&save_file_path).expect("File do not exist");
    }
    let download_url = &info.evolution_chain.url;
    let resp = reqwest::blocking::get(download_url).expect("Request failed");
    let body = resp.bytes().expect("Body invalid");
    let mut out = File::create(save_file_path).expect("Failed to create file");
    let body_bytes = body.to_vec();
    io::copy(&mut &body_bytes[..], &mut out).expect("Failed to copy content"); */
}

fn fill_sprites() {
    let mut x = 1;
    let bar = ProgressBar::new(898);
    while x <= 898 {
        let path = "Sprites/";
        let json = ".png";
        let save_file_path = format!(
            "{}{}{}",
            path,
            &get_pokemon_data_i(x).expect("cannot find this pokemon").id,
            json
        );
        if path_exists(&save_file_path) {
            fs::remove_file(&save_file_path).expect("file do not exist");
        }
        let download_url = &get_pokemon_data_i(x)
            .expect("cannot find this pokemon")
            .sprites
            .front_default;
        let resp = reqwest::blocking::get(download_url).expect("request failed");
        let body = resp.bytes().expect("body invalid");
        let mut out = File::create(save_file_path).expect("failed to create file");
        let body_bytes = body.to_vec();
        io::copy(&mut &body_bytes[..], &mut out).expect("failed to copy content");

        bar.inc(1);
        x += 1;
    }
    bar.finish();
    println!("\nDownload completed !\n");
}

/*
Function get_input :
Returning a String.
Used to read an input in the terminal.
The returned String is trimmed to delete the \n at the end
*/
fn get_input() -> String {
    let reader = io::stdin();
    let mut buffer: String = String::new();
    match reader.read_line(&mut buffer) {
        /* On utilise trim_end pour enlever le \n a la fin de chaque input */
        Ok(_) => buffer.trim_end().to_string(),
        Err(e) => "err".to_string(),
    }
}

/*
Function generation :
Taking a String as argument.
Returning a Generation reference (static).
Used to return a selected generation's pokemon start & end
The input is parsed in u32 (Generation from 1 to 8)
*/
pub fn generation(str: String) -> &'static Generation {
    let rst = match str.parse::<u32>(){
        Ok(t) => t,
        Err(e) => panic!("Problem parsing the data: {:?}", e),
    }/* .unwrap() */;
    let selected_gen: Generation;
    match rst {
        1 => &GEN1,
        2 => &GEN2,
        3 => &GEN3,
        4 => &GEN4,
        5 => &GEN5,
        6 => &GEN6,
        7 => &GEN7,
        8 => &GEN8,
        _ => &GEN1, /* err */
    }
}

/*
Function get_pokemon_data_i :
Taking an index as argument.
Returning a Result<Pokemon, String>.
Used to fill a Pokemon struct with the Json (Pokedex folder)
and returning it.
*/
pub fn get_pokemon_data_i(index: i32) -> Result<Pokemon, String> {
    match index {
        1..=898 => {
            let path = "Pokedex/";
            let json = ".json";
            let file_path = format!("{}{}{}", path, index, json);
            let mut file = match File::open(&file_path){
                Ok(t) => t,
                Err(e) => panic!("Problem opening the file: {:?}", e),
            }/* .unwrap() */;
            let mut buff = String::new();
            match file.read_to_string(&mut buff){
                Ok(t) => t,
                Err(e) => panic!("Problem reading the file: {:?}", e),
            }/* .unwrap() */;
            let pokemon: Pokemon = match serde_json::from_str(&buff) {
                Ok(t) => t,
                Err(e) => panic!("Problem parsing the data into the structure: {:?}", e),
            };
            Ok(pokemon)
        }
        _ => Err("ID does not exist".to_string()),
    }
}

/*
Function print_pokemon :
Taking a Pokemon as argument.
Printing all the fields in the Pokemon struct
*/
pub fn print_pokemon(pokemon: &Pokemon) {
    println!("Name: {}", pokemon.species.name);
    println!("Index : {}", pokemon.id);
    print!("Type :");
    for a in &pokemon.types {
        print!(" {}, ", a.r#type);
    }
    println!();

    println!("Height: {} m", pokemon.height / 10.0);
    println!("Weight: {} kg", pokemon.weight / 10.0);
    println!("Abilities:");
    for a in &pokemon.abilities {
        println!(" {};\tHidden: {}", a.ability.name, a.is_hidden);
    }

    println!("Stats:");
    for a in &pokemon.stats {
        println!(" {} : {};\tEffort: {}", a.stat.name, a.base_stat, a.effort);
    }

    println!("\n");
    println!("\nEvolution Chain :\n");
    print_evo(&get_pokemon_evo_index(pokemon.id.to_string()).expect("Error ").chain);
}

/*
Function is_string_numeric :
Taking a String as argument.
Checks char by char if the input is numeric
*/
fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    true
}

/*
Function is_string_alphabetic :
Taking a String as argument.
Checks char by char if the input is alphabetic
*/
fn is_string_alphabetic(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_alphabetic() {
            return false;
        }
    }
    true
}

/*
Function string_to_i32 :
Taking a String as argument.
Converts a String into an i32 and returns it
*/
pub fn string_to_i32(str: &str) -> i32 {
    if is_string_numeric(str) {
        let rst: i32 = match str.parse(){
            Ok(t) => t,
            Err(e) => panic!("Problem parsing the data: {:?}", e),
        }/* .unwrap() */;
        return rst;
    }
    println!("failed to parse input was not a i32");
    0
}

/*
Function find_pokemon :
Taking a String as argument.
Search a Pokemon by either index or name

index : checks if the index is between 1 & 898
then gets the pokemon data and prints it.

name : iterate through all pokemon to find matching name
then gets the pokemon data and prints it.
*/
fn find_pokemon(str: &str) {
    if is_string_numeric(str) {
        let index: i32 = match str.parse(){
            Ok(t) => t,
            Err(e) => panic!("Problem parsing the data: {:?}", e),
        }/* .unwrap() */;
        if index > 0 && index < 899 {
            print_pokemon(&get_pokemon_data_i(index).expect("This pokemon doesn't exist"));
        } else {
            println!(
                "This Pokemon index doesn't exist.\nRemember Index can only go from 1 to 898 \n"
            );
        }
    } else if is_string_alphabetic(str) {
        let mut x: i32 = 0;
        let mut pok;
        while x < 898 {
            x += 1;
            pok = get_pokemon_data_i(x).expect("This pokemon doesn't exist");
            if str == pok.species.name {
                print_pokemon(&get_pokemon_data_i(x).expect("This pokemon doesn't exist"));
                break;
            }
        }
    }
}

/*
Function get_type_pokemon :
Taking a String and a Generation as argument.
Iterate in the genreation interval to find matching type
then prints it.
*/
pub fn get_type_pokemon(str: String, gen: &Generation) -> Vec<i32> {
    if is_string_alphabetic(&str) {
        let mut x = gen.pokemon_start;
        let mut vec: Vec<i32> = Vec::new();
        while x <= gen.pokemon_end {
            let pokemon = get_pokemon_data_i(x).expect("This pokemon doesn't exist");
            for a in &pokemon.types {
                if str == a.r#type.name {
                    vec.push(pokemon.id);
                }
            }
            x += 1;
        }
        return vec;
    }
    panic!("Error incorrect input !");
}

/*
Functions sort_by_stat :
Taking a String as argument.
Iterate through all pokemons to find a matching stat.
Then takes this stat (name and value) and push it with the pokemon name & index in a vec (of SimplePok)
Sorts this vec by stat.
then prints it by getting the pokemon data with the index stored in the vec.
*/
pub fn sort_by_stat_high(str: String) -> Vec<SimplePok> {
    let mut x = 1;
    let mut vec: Vec<SimplePok> = Vec::new();
    while x <= 898 {
        let pok = &get_pokemon_data_i(x).expect("err");
        for a in &pok.stats {
            if a.stat.name == str {
                vec.push(SimplePok::new(
                    pok.species.name.clone(),
                    pok.id,
                    a.stat.name.clone(),
                    a.base_stat,
                ));
            }
        }
        x += 1;
    }
    vec.sort_by(|a, b| b.stat.cmp(&a.stat));
    vec
}
/* Sort in the other way */
pub fn sort_by_stat_low(str: String) -> Vec<SimplePok> {
    let mut x = 1;
    let mut vec: Vec<SimplePok> = Vec::new();
    while x <= 898 {
        let pok = &get_pokemon_data_i(x).expect("err");
        for a in &pok.stats {
            if a.stat.name == str {
                vec.push(SimplePok::new(
                    pok.species.name.clone(),
                    pok.id,
                    a.stat.name.clone(),
                    a.base_stat,
                ));
            }
        }
        x += 1;
    }
    vec.sort_by(|b, a| b.stat.cmp(&a.stat));
    vec
}




/****************** UI ******************/



pub fn start_menu() {
    std::process::Command::new("clear").status().expect("Error");
    println!("Welcome to the Pokedex\n");
    println!("Start: ENTER\n");
    let input = get_input();
    if input.is_empty() {
        main_menu()
    } else {
        println!("Incorrect Input\n");
        println!("Restarting ...");
        thread::sleep(time::Duration::from_millis(3000));
        start_menu();
    }
}

pub fn main_menu() {
    std::process::Command::new("clear").status().expect("Error");
    println!("\t\t\tMAIN MENU\n");
    println!("All Pokemons: 1          \t\tFind a Pokemon: 2\n");
    println!("Sort Pokemons by stats: 3\t\tSort pokemons by type: 4\n");
    println!("Update the database: 5   \t\tQuit: 6\n");
    let input = get_input();
    let rst = string_to_i32(&input);
    match rst {
        1 => all_pokemons(1),
        2 => find_a_pokemon(),
        3 => sort_pokemons_st(),
        4 => sort_pokemons_t(),
        5 => update_data(),
        6 => println!("End of program !"),
        _ => {
            println!("Incorrect Input\n");
            thread::sleep(time::Duration::from_millis(2000));
            main_menu();
        }
    };
}

pub fn all_pokemons(index: i32) {
    std::process::Command::new("clear").status().expect("Error");
    println!("\t\t\tPOKEMON PRINTER\n");
    print_pokemon(&get_pokemon_data_i(index).expect("Error"));
   /*  println!("\nEvolution Chain :\n");
    print_evo(&get_pokemon_evo_index(index.to_string()).expect("Error ").chain); */
    let evo = get_pokemon_evo_index(index.to_string()).expect("Error ").chain;
    println!("\nPrevious Pokemon: 1\tNext Pokemon: 2\t\tMain Menu: 3\n");
    let input = get_input();
    let rst = string_to_i32(&input);
    match rst {
        1 => {
            if index == 1 {
                println!("Can't go to the previous Pokemon you're already at the first one !\n");
                thread::sleep(time::Duration::from_millis(3000));
                all_pokemons(index);
            } else {
                all_pokemons(index - 1);
            }
        }
        2 => {
            if index == 898 {
                println!("Can't go to the next Pokemon you're already at the last one !\n");
                thread::sleep(time::Duration::from_millis(3000));
                all_pokemons(index);
            } else {
                all_pokemons(index + 1);
            }
        }
        3 => main_menu(),
        _ => {
            println!("Trying Konami code or what ? U stupid ?\n");
            thread::sleep(time::Duration::from_millis(3000));
            all_pokemons(index);
        }
    };
}

pub fn find_a_pokemon() {
    std::process::Command::new("clear").status().expect("Error");
    println!("\t\t\tPOKEMON FINDER\n");
    println!("Enter the name or the index of the pokemon you would like to find\n");
    println!("Main Menu: m\n");
    let input = get_input();
    let input_slice: &str = &input[..];
    match input_slice {
        "m" => main_menu(),
        _ => {
            find_pokemon(&input);
            println!("Another one ?: 1\t Main Menu: 2\n");
            let input = get_input();
            let rst = string_to_i32(&input);
            match rst {
                1 => find_a_pokemon(),
                2 => main_menu(),
                _ => {
                    println!("Incorrect Input\n");
                    thread::sleep(time::Duration::from_millis(2000));
                    find_a_pokemon();
                }
            };
        }
    }
}

pub fn sort_pokemons_st() {
    std::process::Command::new("clear").status().expect("Error");
    println!("\t\t\tPOKEMON SORTER\n");
    println!("Enter the stat you want pokemons to be sort by (hp, attack, defense, special-attack, special-defense, speed)\n");
    println!("Main Menu: m\n");
    let input = get_input();
    let input_slice: &str = &input[..];
    match input_slice {
        "m" => main_menu(),
        _ => {
            println!("Low to high: 1\tHigh to low: 2");
            let input2 = get_input();
            let rst = string_to_i32(&input2);
            match rst {
                1 => {
                    let vec = sort_by_stat_low(input);
                    if vec.is_empty() {
                        println!("This stat doesn't exist");
                        println!("Going back ...");
                        thread::sleep(time::Duration::from_millis(3000));
                        sort_pokemons_st();
                    }
                    let mut x = 0;
                    while x <= vec.len() {
                        std::process::Command::new("clear").status().expect("Error");
                        print_pokemon(&get_pokemon_data_i(vec[x].index).expect("Error"));
                        println!("\nPrevious Pokemon: 1\tNext Pokemon: 2\t\tMain Menu: 3\n");
                        let input3 = get_input();
                        let rst2 = string_to_i32(&input3);
                        match rst2 {
                            1 => {
                                if x == 0 {
                                    println!("Can't go to the previous Pokemon you're already at the first one !\n");
                                    thread::sleep(time::Duration::from_millis(3000));
                                } else {
                                    x -= 1;
                                }
                            }
                            2 => {
                                if x == vec.len() {
                                    println!("Can't go to the next Pokemon you're already at the last one !\n");
                                    thread::sleep(time::Duration::from_millis(3000));
                                } else {
                                    x += 1;
                                }
                            }
                            3 => {
                                main_menu();
                                break;
                            }
                            _ => {
                                println!("Incorrect Input\n");
                                thread::sleep(time::Duration::from_millis(3000));
                            }
                        }
                    }
                    println!("The End !\n");
                    println!("Returning to Main Menu ... \n");
                    thread::sleep(time::Duration::from_millis(3000));
                    main_menu();
                }
                2 => {
                    let vec = sort_by_stat_high(input);
                    let mut x = 0;
                    while x <= vec.len() {
                        print_pokemon(&get_pokemon_data_i(vec[x].index).expect("Error"));
                        println!("\nPrevious Pokemon: 1\tNext Pokemon: 2\t\tMain Menu: 3\n");
                        let input4 = get_input();
                        let rst3 = string_to_i32(&input4);
                        match rst3 {
                            1 => {
                                if x == 0 {
                                    println!("Can't go to the previous Pokemon you're already at the first one !\n");
                                    thread::sleep(time::Duration::from_millis(3000));
                                } else {
                                    x -= 1;
                                }
                            }
                            2 => {
                                if x == vec.len() {
                                    println!("Can't go to the next Pokemon you're already at the last one !\n");
                                    thread::sleep(time::Duration::from_millis(3000));
                                } else {
                                    x += 1;
                                }
                            }
                            3 => {
                                main_menu();
                                break;
                            }
                            _ => {
                                println!("Incorrect Input\n");
                                thread::sleep(time::Duration::from_millis(3000));
                            }
                        }
                    }
                    println!("Nothing !\n");
                    println!("Returning to Main Menu ... \n");
                    thread::sleep(time::Duration::from_millis(3000));
                    main_menu();
                }
                _ => {
                    println!("Incorrect Input\n");
                    thread::sleep(time::Duration::from_millis(2000));
                    sort_pokemons_st();
                }
            };
        }
    }
}

pub fn sort_pokemons_t() {
    std::process::Command::new("clear").status().expect("Error");
    println!("\t\t\tPOKEMON TYPE SORTER\n");
    println!("Enter the type you want pokemons to be sort by \n(Normal, Fighting, Flying, Poison, Ground, Rock, Bug, Ghost, Steel, Fire, Water, Grass, Electric, Psychic, Ice, Dragon, Dark, Fairy)\n");
    println!("Main Menu: m\n");
    let input = get_input();
    let input_slice: &str = &input[..];
    match input_slice {
        "m" => main_menu(),
        _ => {
            println!("Now enter the generation");
            let input2 = get_input();
            let rst = string_to_i32(&input2);
            match rst {
                1..=8 => {
                    let vec = get_type_pokemon(input, generation(input2));
                    let mut x = 0;
                    while x < vec.len() {
                        std::process::Command::new("clear").status().expect("Error");
                        print_pokemon(&get_pokemon_data_i(vec[x]).expect("Error"));
                        println!("\nPrevious Pokemon: 1\tNext Pokemon: 2\t\tMain Menu: 3\n");
                        let input3 = get_input();
                        let rst2 = string_to_i32(&input3);
                        match rst2 {
                            1 => {
                                if x == 0 {
                                    println!("Can't go to the previous Pokemon you're already at the first one !\n");
                                    thread::sleep(time::Duration::from_millis(3000));
                                } else {
                                    x -= 1;
                                }
                            }
                            2 => {
                                if x == vec.len() {
                                    println!("Can't go to the next Pokemon you're already at the last one !\n");
                                    thread::sleep(time::Duration::from_millis(3000));
                                } else {
                                    x += 1;
                                }
                            }
                            3 => {
                                main_menu();
                                break;
                            }
                            _ => {
                                println!("Incorrect Input\n");
                                thread::sleep(time::Duration::from_millis(3000));
                            }
                        }
                    }
                    println!("Nothing !\n");
                    println!("Returning to Main Menu ... \n");
                    thread::sleep(time::Duration::from_millis(3000));
                    main_menu();
                }
                _ => {
                    println!("This generation doesn't exist");
                    thread::sleep(time::Duration::from_millis(3000));
                    sort_pokemons_t();
                }
            }
        }
    };
}

pub fn update_data() {
    std::process::Command::new("clear").status().expect("Error");
    println!("\t\t\tPOKEMON DATA UPDATER\n");
    println!("Wich data base would you like to update ?\n");
    println!("Pokedex: 1         \t\tEvolutions: 2\n");
    println!("Evolution chains: 3\t\tSprites (unused atm): 4\n");
    println!("All: 5             \t\tMain Menu: 6\n");
    let input = get_input();
    let rst = string_to_i32(&input);
    match rst {
        1 => {
            println!("Updating the Pokedex\n");
            fill_pokedex()
        }
        2 => {
            println!("Updating the evolutions\n");
            fill_evolution()
        }
        3 => {
            println!("Updating the evoluton chains\n");
            fill_evolution_chains()
        }
        4 => {
            println!("Updating the sprites\n");
            fill_sprites()
        }
        5 => {
            println!("Updating the Pokedex\n");
            fill_pokedex();
            println!("Starting next download ...\n");
            thread::sleep(time::Duration::from_millis(2000));
            println!("Updating the evolutions\n");
            fill_evolution();
            println!("Starting next download ...\n");
            thread::sleep(time::Duration::from_millis(2000));
            println!("Updating the evoluton chains\n");
            fill_evolution_chains();
            println!("Starting next download ...\n");
            thread::sleep(time::Duration::from_millis(2000));
            println!("Updating the sprites\n");
            fill_sprites();
        }
        6 => main_menu(),
        _ => {
            println!("Incorrect Input\n");
            thread::sleep(time::Duration::from_millis(2000));
            update_data();
        }
    };
    println!("Nothing !\n");
    println!("Returning to Main Menu ... \n");
    thread::sleep(time::Duration::from_millis(3000));
    main_menu();
}

fn main() {
    start_menu();
}