use std::collections::BTreeMap;
use std::io::{stdin, stdout, Write};
use crate::modules::settings::Settings;

mod modules;


fn main() {
    println!("This is the companion installer for RedAlt-Steam-Update-Creator.\n\
    Enter \"help\" to get a list of commands. Enter \"update\" to start the update process.");
    let mut settings = Settings::default();
    println!("\n{}", settings);

    loop {
        let input = get_input(">>");

        match input.as_str().split(' ').next().unwrap() {
            "changes" => settings.show_changes(),
            "exit" => break,
            "help" => get_help(input),
            "set" => settings.modify_fields(input),
            "settings" => println!("{}", settings),
            "update" => settings.update_game(),
            _ => println!("Command not recognised. Type \"help\" for a list of commands."),
        }
    }
}

pub fn get_input(prompt: &str) -> String {
    let mut line = String::new();
    print!("{} ", prompt);
    stdout().flush().expect("Error: Could not flush stdout");
    stdin().read_line(&mut line).expect("Error: Could not read a line");

    return line.trim().to_string()
}

fn get_help(input: String) {
    /*let input = input.split(' ').collect::<Vec<&str>>();
    match input.get(1) {
        None => {}
        Some(_) => {}
    };*/
    let mut help = BTreeMap::new();
    help.insert("changes", "Show the changelog.");
    help.insert("exit", "Exit the program.");
    help.insert("help", "Show help for the given command.");
    help.insert("set <field> <value>", "Set the given field to the given value.\
    To see available fields, type \"settings\".");
    help.insert("settings", "Get the current settings.");
    help.insert("update", "Update the game files.");

    for (key, value) in help {
        println!("{}: {}", key, value);
    }
}