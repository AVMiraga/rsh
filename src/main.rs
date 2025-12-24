#[allow(unused_imports)]
use std::io::{self, Write};

const VALID_COMMANDS_BUILTIN: &[&str] = &["echo", "exit", "type"];

fn main() {
    enum CommandTypes {
        Builtin,
    }

    loop {
        let mut command = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        let mut whole_command = command.split_whitespace();
        let command = whole_command.next().unwrap_or("");
        let arguments: String = whole_command.collect::<Vec<&str>>().join(" ");

        match command.trim() {
            "exit" => break,
            "echo" => {
                println!("{}", arguments);
            }
            "type" => {
                if VALID_COMMANDS_BUILTIN.contains(&arguments.trim()) {
                    println!("{} is a shell builtin", arguments.trim());
                } else {
                    println!("{}: not found", arguments.trim());
                }
            }
            _ => println!("{}: command not found", &command.trim()),
        }
    }
}
