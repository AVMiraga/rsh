#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
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
            _ => println!("{}: command not found", &command.trim()),
        }
    }
}
