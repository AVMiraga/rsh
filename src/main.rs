#[allow(unused_imports)]
use std::io::{self, Write};

const VALID_COMMANDS: &[&str] = &["exit"];

fn main() {
    loop {
        let mut command = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        if !VALID_COMMANDS.contains(&command.as_str().trim()) {
            println!("{}: command not found", &command.trim());
        }

        if command.trim() == "exit" {
            break;
        }
    }
}
