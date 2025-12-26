use pathsearch::find_executable_in_path;
use std::io::{self, Write};
use std::process::Command;

// #[cfg(windows)]
// const PATH_SEP: char = ';';
// #[cfg(not(windows))]
// const PATH_SEP: char = ':';

const VALID_COMMANDS_BUILTIN: &[&str] = &["echo", "exit", "type"];

#[test]
fn testing() {
    dbg!(find_executable_in_path("lse"));
}

fn main() {
    loop {
        let mut command = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        let mut whole_command = command.split_whitespace();
        let command = whole_command.next().unwrap_or("");
        let arguments = whole_command.collect::<Vec<&str>>();

        match command.trim() {
            "exit" => break,
            "echo" => {
                print!("{}", arguments.join(" "));
            }
            "type" => {
                if VALID_COMMANDS_BUILTIN.contains(&arguments.join(" ").trim()) {
                    println!("{} is a shell builtin", arguments.join(" ").trim());
                } else if let Some(path) = find_executable_in_path(&arguments.join(" ").trim()) {
                    println!(
                        "{} is {}",
                        &arguments.join(" ").trim(),
                        path.to_str().unwrap()
                    );
                } else {
                    println!("{}: not found", arguments.join(" ").trim());
                }
            }
            _ => match find_executable_in_path(command.trim()) {
                Some(_) => {
                    let out = Command::new(command)
                        .args(arguments)
                        .output()
                        .expect("failed to execute process");

                    io::stdout().write_all(&out.stdout).unwrap();
                }
                _ => println!("{}: command not found", &command.trim()),
            },
        }
    }
}
