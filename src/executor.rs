use pathsearch::find_executable_in_path;
use shlex::split;
use std::{
    env::{self, current_dir, set_current_dir},
    fs::OpenOptions,
    io::{stderr, stdout, Write},
    path::Path,
    process::Command,
};

use crate::builtins::VALID_COMMANDS_BUILTIN;
use crate::commands::pipeline_handler;
use crate::history::get_history;
use crate::redirection::RedirectionKind;

/// Redirections with their operators and kinds (runtime version)
const REDIRECTIONS: [(&[&str], RedirectionKind); 4] = [
    (&[">", "1>"], RedirectionKind::Stdout),
    (&["2>"], RedirectionKind::Stderr),
    (&[">>", "1>>"], RedirectionKind::AppendStdout),
    (&["2>>"], RedirectionKind::AppendStderr),
];

/// Execute a shell command
pub fn run_sh(command: &mut String, local_history: &mut Vec<String>) -> std::io::Result<()> {
    if command.is_empty() {
        println!();
        print!("\r$ ");
        stdout().flush()?;
    }
    println!();
    stdout().flush()?;

    if pipeline_handler(command)? {
        command.clear();
        return Ok(());
    }

    let whole_command = split(command.trim()).unwrap_or([].to_vec());
    command.clear();
    let command = whole_command.first().unwrap();
    let arguments = whole_command[1..].to_vec();

    let mut redir_kind = &RedirectionKind::Stdout;
    let mut to_file = "";
    let mut from_content: Vec<String> = vec![];

    for (ops, kind) in &REDIRECTIONS {
        let mut argument_iter = arguments.iter();
        if let Some(redirect_pos) = argument_iter.position(|s| ops.contains(&s.as_str())) {
            if let Some(file) = argument_iter.next() {
                redir_kind = kind;
                to_file = file;
                from_content = arguments[..redirect_pos].to_vec();
            }
        }
    }
    match command.trim() {
        "exit" => {
            let file_path = std::env::var_os("HISTFILE");
            let existing_history_len = get_history().len();

            if let Some(file_path) = file_path {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .write(true)
                    .open(file_path)?;

                file.write_all(local_history[existing_history_len..].join("\n").as_bytes())?;
                file.write_all("\n".as_bytes())?;
            }

            std::process::exit(0);
        }
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
                    .open(to_file)?;

                let from_content = from_content.join(" ");

                match redir_kind {
                    RedirectionKind::Stdout | RedirectionKind::AppendStdout => {
                        file.write_all(from_content.as_bytes())?;
                        file.write_all("\n".as_bytes())?;
                    }
                    RedirectionKind::Stderr | RedirectionKind::AppendStderr => {
                        file.write_all("".as_bytes())?;
                        println!("{}", from_content.trim());
                    }
                }
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
            println!("{}", current_dir()?.to_str().unwrap());
        }
        "history" => {
            let mut history_size: usize = local_history.len();
            let mut skip_print = false;

            if !arguments.is_empty() {
                let arg = &arguments[0];

                if arg == "-r" && arguments.len() > 1 {
                    let file = &arguments[1];
                    let file_content = std::fs::read_to_string(file)?;

                    let file_content = file_content.lines().collect::<Vec<&str>>();
                    local_history.extend(file_content.iter().map(ToString::to_string));

                    skip_print = true;
                } else if arg == "-w" && arguments.len() > 1 {
                    let file = &arguments[1];
                    let mut file = OpenOptions::new().create(true).write(true).open(file)?;

                    file.write_all(local_history.join("\n").as_bytes())?;
                    file.write_all("\n".as_bytes())?;

                    skip_print = true;
                } else if arg == "-a" && arguments.len() > 1 {
                    let file_name = &arguments[1];
                    let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(file_name)?;

                    let search_str = format!("history -a {}", file_name);

                    let history_slice = local_history
                        .iter()
                        .rev()
                        .enumerate()
                        .filter(|&(_, s)| *s == search_str)
                        .take(2)
                        .map(|(i, _)| local_history.len() - i)
                        .collect::<Vec<usize>>();

                    //as it is reversed, we need to slice reversed, if it contains one occurrence
                    //then it happened once

                    if history_slice.len() > 1 {
                        file.write_all(
                            local_history[history_slice[1]..history_slice[0]]
                                .join("\n")
                                .as_bytes(),
                        )?;
                        file.write_all("\n".as_bytes())?;
                    } else {
                        file.write_all(local_history[..].join("\n").as_bytes())?;
                        file.write_all("\n".as_bytes())?;
                    }
                    skip_print = true;
                }

                history_size = arg.parse::<usize>().unwrap_or(local_history.len());
            }

            let history_skip = if history_size > local_history.len() {
                0
            } else {
                local_history.len() - history_size
            };

            if !skip_print {
                for (i, cmd) in local_history.iter().enumerate().skip(history_skip) {
                    println!("    {} {}", i + 1, cmd);
                }
            }
        }
        "." => {
            set_current_dir(current_dir()?)?;
        }
        ".." => {
            let new_dir = current_dir()?.pop().to_string();
            set_current_dir(new_dir)?;
        }
        "cd" => {
            let new_arg = &arguments[0].replace("~", env::home_dir().unwrap().to_str().unwrap());
            let new_dir = Path::new(new_arg).to_path_buf();

            match set_current_dir(new_dir) {
                Ok(_) => {}
                Err(_) => {
                    println!("cd: {}: No such file or directory", new_arg)
                }
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
                    .output()?;

                if to_file.is_empty() {
                    stdout().write_all(&out.stdout)?;
                    if out.status.success() && out.stdout.last() != Some(&b'\n') {
                        println!();
                    }
                    stderr().write_all(&out.stderr)?;
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
                        .open(to_file)?;

                    match redir_kind {
                        RedirectionKind::Stdout | RedirectionKind::AppendStdout => {
                            file.write_all(&out.stdout)?;
                            stderr().write_all(&out.stderr)?;
                        }
                        RedirectionKind::Stderr | RedirectionKind::AppendStderr => {
                            file.write_all(&out.stderr)?;
                            stdout().write_all(&out.stdout)?;
                        }
                    }
                }
            }
            _ => println!("{}: command not found", &command.trim()),
        },
    }
    Ok(())
}
