use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use pathsearch::find_executable_in_path;
use shlex::split;
use std::cmp::max;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::process::Output;
use std::{
    env::{self, current_dir, set_current_dir},
    fs::OpenOptions,
    io::{Write, stderr, stdout},
    path::Path,
    process::{Command, Stdio},
};

const VALID_COMMANDS_BUILTIN: &[&str] =
    &["echo", "exit", "type", "pwd", "cd", "history", ".", ".."];

fn lcp(strings: Vec<String>) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let mut sorted_strings = strings.to_vec();
    sorted_strings.sort_unstable();

    let first = sorted_strings.first().unwrap();
    let last = sorted_strings.last().unwrap();

    let lcp_len = first
        .chars()
        .zip(last.chars())
        .take_while(|&(c1, c2)| c1 == c2)
        .count();

    first[..lcp_len].to_string()
}

// First command wont have stdin, only stdout with pipe
// middle command will have both stdin and stdout
// last command should stdout immediately

fn pipeline_handler(command: &str) -> std::io::Result<bool> {
    let cmds = command.split(" | ").collect::<Vec<&str>>();
    let mut last_output: Option<Stdio> = None;
    let mut children = Vec::new();

    if cmds.len() > 1 {
        for (i, cmd) in cmds.iter().enumerate() {
            let whole_command = split(cmd.trim()).unwrap_or([].to_vec());
            let command = whole_command.first().unwrap();
            let arguments = whole_command[1..].to_vec();

            match command.trim() {
                "exit" => std::process::exit(0),
                "echo" => {
                    let mut builtin_output = arguments.join(" ");
                    let builtin_output = format!("{}\n", builtin_output.trim());
                    let mut fake_process = Command::new("cat")
                        .stdin(Stdio::piped())
                        .stdout(if i == cmds.len() - 1 {
                            Stdio::inherit()
                        } else {
                            Stdio::piped()
                        })
                        .spawn()?;

                    if let Some(mut fake_stdin) = fake_process.stdin.take() {
                        fake_stdin.write_all(builtin_output.as_bytes())?;
                    }

                    last_output = fake_process.stdout.take().map(Stdio::from);
                    children.push(fake_process);
                }
                "type" => {
                    if VALID_COMMANDS_BUILTIN.contains(&arguments.join(" ").trim()) {
                        let mut builtin_output = arguments.join(" ");
                        let builtin_output =
                            format!("{} is a shell builtin\n", builtin_output.trim());

                        let mut fake_process = Command::new("cat")
                            .stdin(Stdio::piped())
                            .stdout(if i == cmds.len() - 1 {
                                Stdio::inherit()
                            } else {
                                Stdio::piped()
                            })
                            .spawn()?;

                        if let Some(mut fake_stdin) = fake_process.stdin.take() {
                            fake_stdin.write_all(builtin_output.as_bytes())?;
                        }

                        last_output = fake_process.stdout.take().map(Stdio::from);
                        children.push(fake_process);
                    } else if let Some(path) = find_executable_in_path(&arguments.join(" ").trim())
                    {
                        let mut builtin_output = arguments.join(" ");
                        let builtin_output =
                            format!("{} is {}\n", builtin_output.trim(), path.to_str().unwrap());

                        let mut fake_process = Command::new("cat")
                            .stdin(Stdio::piped())
                            .stdout(if i == cmds.len() - 1 {
                                Stdio::inherit()
                            } else {
                                Stdio::piped()
                            })
                            .spawn()?;

                        if let Some(mut fake_stdin) = fake_process.stdin.take() {
                            fake_stdin.write_all(builtin_output.as_bytes())?;
                        }

                        last_output = fake_process.stdout.take().map(Stdio::from);
                        children.push(fake_process);
                    } else {
                        let mut builtin_output = arguments.join(" ");
                        let builtin_output = format!("{}: not found\n", builtin_output.trim());

                        let mut fake_process = Command::new("cat")
                            .stdin(Stdio::piped())
                            .stdout(if i == cmds.len() - 1 {
                                Stdio::inherit()
                            } else {
                                Stdio::piped()
                            })
                            .spawn()?;

                        if let Some(mut fake_stdin) = fake_process.stdin.take() {
                            fake_stdin.write_all(builtin_output.as_bytes())?;
                        }

                        last_output = fake_process.stdout.take().map(Stdio::from);
                        children.push(fake_process);
                    }
                }
                "pwd" => {
                    let mut builtin_output = arguments.join(" ");
                    let builtin_output = format!("{}\n", current_dir()?.to_str().unwrap());

                    let mut fake_process = Command::new("cat")
                        .stdin(Stdio::piped())
                        .stdout(if i == cmds.len() - 1 {
                            Stdio::inherit()
                        } else {
                            Stdio::piped()
                        })
                        .spawn()?;

                    if let Some(mut fake_stdin) = fake_process.stdin.take() {
                        fake_stdin.write_all(builtin_output.as_bytes())?;
                    }

                    last_output = fake_process.stdout.take().map(Stdio::from);
                    children.push(fake_process);
                }
                _ => {
                    let mut child_process = Command::new(command)
                        .args(&arguments)
                        .stdin(last_output.unwrap_or(Stdio::inherit()))
                        .stdout(if i == cmds.len() - 1 {
                            Stdio::inherit()
                        } else {
                            Stdio::piped()
                        })
                        .spawn()?;

                    last_output = child_process.stdout.take().map(Stdio::from);

                    children.push(child_process);
                }
            }
        }
        stdout().flush()?;

        for mut child in children {
            let _ = child.wait();
        }

        return Ok(true);
    }

    Ok(false)
}

#[test]
fn testing() -> anyhow::Result<()> {
    let command = "cat test.txt | wc | wc";
    let pipelined = pipeline_handler(command);

    Ok(())

    // for cmd in pipelined {
    //     let whole_command = split(cmd.trim()).unwrap_or([].to_vec());
    //     let command = whole_command.first().unwrap();
    //     let arguments = whole_command[1..].to_vec();
    //
    //
    // }
}

enum RedirectionKind {
    Stdout,
    Stderr,
    AppendStdout,
    AppendStderr,
}

fn main() -> std::io::Result<()> {
    let mut cmds = Vec::<String>::new();
    let mut local_history = Vec::<String>::new();

    if let Some(path) = env::var_os("PATH") {
        for e in env::split_paths(&path) {
            if let Ok(p) = e.read_dir() {
                p.filter_map(Result::ok)
                    .filter_map(|ep| {
                        let meta = ep.metadata().ok()?;
                        if meta.mode() & 0o111 != 0 {
                            Some(ep)
                        } else {
                            None
                        }
                    })
                    .for_each(|path| {
                        cmds.push(
                            path.path()
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .into_owned(),
                        );
                    });
            }
        }
    }

    cmds.extend(VALID_COMMANDS_BUILTIN.iter().map(|s| s.to_string()));
    let set_cmds = cmds.into_iter().collect::<HashSet<String>>();
    let cmds = set_cmds.into_iter().collect::<Vec<_>>();

    let mut expect_completions = false;

    loop {
        let mut command = String::new();

        // stdin().read_line(&mut command).unwrap();
        // stdout().flush().unwrap();
        print!("\r$ ");
        stdout().flush()?;

        let mut idx = 0;

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

                        let mut possible_cmd: Vec<String> = cmds
                            .iter()
                            .filter(|x| x.starts_with(command.as_str()))
                            .map(|x| x.to_string())
                            .collect();

                        if possible_cmd.is_empty() {
                            print!("\x07");
                            stdout().flush()?;
                            continue;
                        }

                        possible_cmd.sort();

                        let lcp_possible_command = lcp(possible_cmd.clone());

                        if !lcp_possible_command.is_empty()
                            && possible_cmd.len() > 1
                            && !lcp_possible_command.eq_ignore_ascii_case(&command)
                        {
                            command = lcp_possible_command;
                            print!("\r\x1b[2K$ {}", command);
                            stdout().flush()?;
                        } else {
                            if expect_completions && possible_cmd.len() > 1 {
                                disable_raw_mode()?;
                                print!("\r\n");
                                print!("{}\n", possible_cmd.join("  "));
                                print!("$ {}", command);
                                stdout().flush()?;
                            } else {
                                if possible_cmd.len() > 1 && !expect_completions {
                                    expect_completions = true;
                                    print!("\x07");
                                    stdout().flush()?;
                                } else {
                                    command = possible_cmd[0].to_string() + " ";
                                    print!("\r\x1b[2K$ {}", command);

                                    stdout().flush()?;
                                }
                            }
                        }
                    }
                    KeyCode::Char('j') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        disable_raw_mode()?;
                        local_history.push(command.clone());
                        idx = 0;
                        run_sh(&mut command, &local_history)?;
                        print!("\r$ ");
                        stdout().flush()?;
                    }
                    KeyCode::Char(c) => {
                        command.push(c);
                        print!("{c}");
                        stdout().flush()?;
                    }
                    KeyCode::Enter => {
                        disable_raw_mode()?;
                        local_history.push(command.clone());
                        idx = 0;
                        run_sh(&mut command, &local_history)?;
                        print!("\r$ ");
                        stdout().flush()?;
                    }
                    KeyCode::Up => {
                        if local_history.len() > 0 && idx < local_history.len() {
                            idx += 1;
                            command = local_history[local_history.len() - idx].clone();
                            print!("\r\x1b[2K$ {}", command);
                            stdout().flush()?;
                        }
                    }
                    KeyCode::Down => {
                        if local_history.len() > 0 && idx > 1 {
                            idx -= 1;
                            command = local_history[local_history.len() - idx].clone();
                            print!("\r\x1b[2K$ {}", command);
                            stdout().flush()?;
                        }
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
}

fn run_sh(command: &mut String, local_history: &Vec<String>) -> std::io::Result<()> {
    let redirections = [
        (vec![">", "1>"], RedirectionKind::Stdout),
        (vec!["2>"], RedirectionKind::Stderr),
        (vec![">>", "1>>"], RedirectionKind::AppendStdout),
        (vec!["2>>"], RedirectionKind::AppendStderr),
    ];

    if command.len() == 0 {
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
    match command.trim() {
        "exit" => std::process::exit(0),
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
            if !arguments.is_empty() {
                history_size = arguments[0].parse::<usize>().unwrap_or(local_history.len());
            }
            let history_skip = if history_size > local_history.len() {
                0
            } else {
                local_history.len() - history_size
            };
            for (i, cmd) in local_history.iter().enumerate().skip(history_skip) {
                println!("    {} {}", i + 1, cmd);
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
                        .open(&to_file)?;

                    match redir_kind {
                        RedirectionKind::Stdout | RedirectionKind::AppendStdout => {
                            file.write_all(&out.stdout)?;
                            stderr().write_all(&out.stderr)?;
                        }
                        RedirectionKind::Stderr | RedirectionKind::AppendStderr => {
                            file.write_all(&out.stderr)?;
                            stdout().write(&out.stdout)?;
                        }
                    }
                }
            }
            _ => println!("{}: command not found", &command.trim()),
        },
    }
    Ok(())
}
