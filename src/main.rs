#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        let mut command = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        let mut whole_command = command.split_whitespace();
        let command = match whole_command.next() {
            Some(command) => command,
            _ => "",
        };
        let arguments: String = whole_command.collect();

        match command.trim() {
            "exit" => break,
            "echo" => {
                println!("{}", arguments);
            }
            _ => println!("{}: command not found", &command.trim()),
        }
    }
}
