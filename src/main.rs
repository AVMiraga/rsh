use pathsearch::find_executable_in_path;
use shlex::split;
use std::env;
use std::env::{current_dir, set_current_dir};
use std::fs::OpenOptions;
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
    enum RedirectionKind {
        Stdout,
        Stderr,
        AppendStdout,
        AppendStderr,
    }

    let redirections = [
        (vec![">", "1>"], RedirectionKind::Stdout),
        (vec!["2>"], RedirectionKind::Stderr),
        (vec![">>", "1>>"], RedirectionKind::AppendStdout),
        (vec!["2>>"], RedirectionKind::AppendStderr),
    ];

    loop {
        let mut command = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        let whole_command = split(command.trim()).unwrap_or([].to_vec());
        let command = whole_command.first().unwrap();
        let arguments = whole_command[1..].to_vec();

        let mut redir_kind = &RedirectionKind::Stdout;
        let mut to_file = "";
        let mut from_content: Vec<String> = vec![];

        for (ops, kind) in &redirections {
            let mut argument_iter = arguments.iter();
            if let Some(redirect_pos) = argument_iter.position(|s| ops.contains(&s.as_str())) {
                if let Some(file) = argument_iter.next() {
                    redir_kind = kind;
                    to_file = file;
                    from_content = arguments[..redirect_pos].to_vec();
                }
            }
        }

        // match argument_iter.position(|s| s == ">" || s == "1>") {
        //     Some(redirect_pos) => {
        //         is_redirection = true;
        //         if let Some(file) = argument_iter.next() {
        //             to_file = file;
        //         }
        //         from_content = arguments[..redirect_pos].to_vec();
        //     }
        //     None => {}
        // }

        // let mut file = OpenOptions::new().append().open(to_file).unwrap();

        // if is_redirection || is_err_redirection {}

        match command.trim() {
            "exit" => break,
            "echo" => {
                if to_file.is_empty() {
                    println!("{}", arguments.join(" ").trim());
                } else {
                    let is_append = matches!(
                        redir_kind,
                        RedirectionKind::AppendStdout | RedirectionKind::AppendStderr
                    );
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(is_append)
                        .truncate(!is_append)
                        .write(true)
                        .open(&to_file)
                        .unwrap();

                    let mut from_content = from_content.join(" ");
                    from_content.push_str("\n");

                    match redir_kind {
                        RedirectionKind::Stdout | RedirectionKind::AppendStdout => {
                            file.write_all(from_content.as_bytes()).unwrap();
                        }
                        RedirectionKind::Stderr | RedirectionKind::AppendStderr => {
                            file.write_all("".as_bytes()).unwrap();
                            println!("{}", from_content.trim());
                        }
                    }
                    // if is_redirection {
                    //     std::fs::write(to_file, from_content.join(" ")).unwrap();
                    // } else if is_err_redirection {
                    //     println!("{}", from_content.join(" ").trim());
                    //     std::fs::write(to_file, "").unwrap();
                    // }
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
                        .args(if !to_file.is_empty() {
                            &from_content
                        } else {
                            &arguments
                        })
                        .output()
                        .unwrap();

                    if to_file.is_empty() || !out.status.success() {
                        stdout().write_all(&out.stdout).unwrap();
                        if out.status.success() && out.stdout.last() != Some(&b'\n') {
                            println!();
                        }
                        stderr().write_all(&out.stderr).unwrap();
                    } else {
                        let is_append = matches!(
                            redir_kind,
                            RedirectionKind::AppendStdout | RedirectionKind::AppendStderr
                        );
                        let mut file = OpenOptions::new()
                            .create(true)
                            .append(is_append)
                            .truncate(!is_append)
                            .write(true)
                            .open(&to_file)
                            .unwrap();

                        match redir_kind {
                            RedirectionKind::Stdout | RedirectionKind::AppendStdout => {
                                file.write_all(&out.stdout).unwrap();
                                stderr().write_all(&out.stderr).unwrap();
                            }
                            RedirectionKind::Stderr | RedirectionKind::AppendStderr => {
                                file.write_all(&out.stderr).unwrap();
                                stdout().write(&out.stdout).unwrap();
                            }
                        }
                    }
                }
                _ => println!("{}: command not found", &command.trim()),
            },
        }
    }
}
