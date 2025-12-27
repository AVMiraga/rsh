use pathsearch::find_executable_in_path;
use shlex::split;
use std::env;
use std::env::{current_dir, set_current_dir};
use std::io::{self, Write, stderr, stdout};
use std::path::Path;
use std::process::Command;

// #[cfg(windows)]
// const PATH_SEP: char = ';';
// #[cfg(not(windows))]
// const PATH_SEP: char = ':';

const VALID_COMMANDS_BUILTIN: &[&str] = &["echo", "exit", "type", "pwd", "cd", ".", ".."];

#[test]
fn testing() {
    let command = String::from("echo hello hello > output.txt");

    let whole_command = split(command.trim()).unwrap_or([].to_vec());
    let command = whole_command.first().unwrap();
    let arguments = whole_command[1..].to_vec();

    let mut argument_iter = arguments.iter();
    let mut is_redirection = false;
    let mut to_file = "";
    let mut from_content: Vec<String> = vec![];

    match argument_iter.position(|s| s == ">" || s == "1>") {
        Some(redirect_pos) => {
            is_redirection = true;
            if let Some(file) = argument_iter.next() {
                to_file = file;
            }
            from_content = arguments[..redirect_pos].to_vec();
        }
        None => {}
    }

    dbg!(&to_file);
    dbg!(&from_content);

    // if let Some(redirect_operator_pos) = argument_iter.position(|s| s == ">" || s == "1>") {
    //     if let Some(filename) = argument_iter.next() {
    //         dbg!(filename);
    //     }
    // }

    dbg!(&arguments);
    dbg!(&command);
    dbg!(whole_command);
}

fn main() {
    loop {
        let mut command = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        let whole_command = split(command.trim()).unwrap_or([].to_vec());
        let command = whole_command.first().unwrap();
        let arguments = whole_command[1..].to_vec();

        let mut argument_iter = arguments.iter();
        let mut is_redirection = false;
        let mut is_err_redirection = false;
        let mut to_file = "";
        let mut from_content: Vec<String> = vec![];

        match argument_iter.position(|s| s == ">" || s == "1>") {
            Some(redirect_pos) => {
                is_redirection = true;
                if let Some(file) = argument_iter.next() {
                    to_file = file;
                }
                from_content = arguments[..redirect_pos].to_vec();
            }
            None => {}
        }

        let mut argument_iter = arguments.iter();

        match argument_iter.position(|s| s == "2>") {
            Some(redirect_pos) => {
                is_err_redirection = true;
                is_redirection = false;
                if let Some(file) = argument_iter.next() {
                    to_file = file;
                }
                from_content = arguments[..redirect_pos].to_vec();
            }
            None => {}
        }

        match command.trim() {
            "exit" => break,
            "echo" => {
                if is_redirection {
                    std::fs::write(to_file, from_content.join(" ")).unwrap();
                } else {
                    println!("{}", arguments.join(" ").trim());
                }
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
            "pwd" => {
                println!("{}", current_dir().unwrap().to_str().unwrap());
            }
            "." => {
                set_current_dir(current_dir().unwrap()).unwrap();
            }
            ".." => {
                let new_dir = current_dir().unwrap().pop().to_string();
                set_current_dir(new_dir).unwrap();
            }
            "cd" => {
                let new_arg =
                    &arguments[0].replace("~", env::home_dir().unwrap().to_str().unwrap());
                let new_dir = Path::new(new_arg).to_path_buf();

                match set_current_dir(new_dir) {
                    Ok(_) => {}
                    Err(_) => println!("cd: {}: No such file or directory", new_arg),
                }
            }
            _ => match find_executable_in_path(command.trim()) {
                Some(_) => {
                    let out = Command::new(command)
                        .args(if is_redirection || is_err_redirection {
                            &from_content
                        } else {
                            &arguments
                        })
                        .output()
                        .unwrap();
                    if is_redirection {
                        std::fs::write(to_file, &out.stdout).unwrap();
                        stderr().write_all(&out.stderr).unwrap();
                    } else if is_err_redirection {
                        std::fs::write(to_file, &out.stderr).unwrap();
                        stdout().write_all(&out.stdout).unwrap();
                    } else {
                        stdout().write_all(&out.stdout).unwrap();
                        if out.stdout.last() != Some(&b'\n') {
                            println!();
                        }
                        stderr().write_all(&out.stderr).unwrap();
                    }
                }
                _ => println!("{}: command not found", &command.trim()),
            },
        }
    }
}
