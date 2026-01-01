use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, ModifierKeyCode, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use pathsearch::find_executable_in_path;
use shlex::split;
use std::{
    env::{self, current_dir, set_current_dir},
    fs::OpenOptions,
    io::{Write, stderr, stdout},
    path::Path,
    process::Command,
};

const VALID_COMMANDS_BUILTIN: &[&str] = &["echo", "exit", "type", "pwd", "cd", ".", ".."];

#[test]
fn testing() {}

fn main() -> std::io::Result<()> {
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

        // stdin().read_line(&mut command).unwrap();
        // stdout().flush().unwrap();
        print!("\r$ ");
        stdout().flush()?;

        loop {
            enable_raw_mode()?;
            if let Event::Key(k) = read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }

                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code.is_char('c') {
                    return Ok(());
                }

                match k.code {
                    KeyCode::Tab => {
                        if command.is_empty() {
                            print!("\x07");
                            stdout().flush()?;
                            continue;
                        }
                        let possible_cmd: Vec<String> = VALID_COMMANDS_BUILTIN
                            .iter()
                            .filter(|x| x.starts_with(command.as_str()))
                            .map(|x| x.to_string())
                            .collect();
                        if possible_cmd.is_empty() {
                            print!("\x07");
                            stdout().flush()?;
                            continue;
                        }
                        command = possible_cmd[0].to_string() + " ";
                        print!("\r\x1b[2K$ {}", command);

                        stdout().flush()?;
                    }
                    KeyCode::Char(c) => {
                        command.push(c);
                        print!("{c}");
                        stdout().flush()?;
                    }
                    KeyCode::Enter => {
                        disable_raw_mode()?;
                        if command.len() == 0 {
                            println!();
                            print!("\r$ ");
                            stdout().flush()?;
                            continue;
                        }
                        println!();
                        stdout().flush()?;
                        let whole_command = split(command.trim()).unwrap_or([].to_vec());
                        command.clear();
                        let command = whole_command.first().unwrap();
                        let arguments = whole_command[1..].to_vec();

                        let mut redir_kind = &RedirectionKind::Stdout;
                        let mut to_file = "";
                        let mut from_content: Vec<String> = vec![];

                        for (ops, kind) in &redirections {
                            let mut argument_iter = arguments.iter();
                            if let Some(redirect_pos) =
                                argument_iter.position(|s| ops.contains(&s.as_str()))
                            {
                                if let Some(file) = argument_iter.next() {
                                    redir_kind = kind;
                                    to_file = file;
                                    from_content = arguments[..redirect_pos].to_vec();
                                }
                            }
                        }
                        match command.trim() {
                            "exit" => break,
                            "echo" => {
                                if to_file.is_empty() {
                                    println!("{}", arguments.join(" ").trim());
                                } else {
                                    let is_append = matches!(
                                        redir_kind,
                                        RedirectionKind::AppendStdout
                                            | RedirectionKind::AppendStderr
                                    );
                                    let mut file = OpenOptions::new()
                                        .create(true)
                                        .append(is_append)
                                        .truncate(!is_append)
                                        .write(true)
                                        .open(&to_file)?;

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
                                } else if let Some(path) =
                                    find_executable_in_path(&arguments.join(" ").trim())
                                {
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
                            "." => {
                                set_current_dir(current_dir()?)?;
                            }
                            ".." => {
                                let new_dir = current_dir()?.pop().to_string();
                                set_current_dir(new_dir)?;
                            }
                            "cd" => {
                                let new_arg = &arguments[0]
                                    .replace("~", env::home_dir().unwrap().to_str().unwrap());
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
                                        if out.status.success() && out.stdout.last() != Some(&b'\n')
                                        {
                                            println!();
                                        }
                                        stderr().write_all(&out.stderr)?;
                                    } else {
                                        let is_append = matches!(
                                            redir_kind,
                                            RedirectionKind::AppendStdout
                                                | RedirectionKind::AppendStderr
                                        );
                                        let mut file = OpenOptions::new()
                                            .create(true)
                                            .append(is_append)
                                            .truncate(!is_append)
                                            .write(true)
                                            .open(&to_file)?;

                                        match redir_kind {
                                            RedirectionKind::Stdout
                                            | RedirectionKind::AppendStdout => {
                                                file.write_all(&out.stdout)?;
                                                stderr().write_all(&out.stderr)?;
                                            }
                                            RedirectionKind::Stderr
                                            | RedirectionKind::AppendStderr => {
                                                file.write_all(&out.stderr)?;
                                                stdout().write(&out.stdout)?;
                                            }
                                        }
                                    }
                                }
                                _ => println!("{}: command not found", &command.trim()),
                            },
                        }
                        print!("\r$ ");
                        stdout().flush()?;
                    }
                    KeyCode::Backspace => {
                        if command.len() > 0 {
                            command.pop();
                            print!("\x08 \x08");
                            stdout().flush()?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
