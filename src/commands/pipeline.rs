use crate::builtins::VALID_COMMANDS_BUILTIN;
use pathsearch::find_executable_in_path;
use shlex::split;
use std::{
    env::current_dir,
    io::{stdout, Write},
    process::{Command, Stdio},
};

/// Handle piped commands (e.g., "cmd1 | cmd2 | cmd3")
/// Returns Ok(true) if pipeline was handled, Ok(false) if not a pipeline
pub fn pipeline_handler(command: &str) -> std::io::Result<bool> {
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
                    let builtin_output = arguments.join(" ");
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
                        let builtin_output = arguments.join(" ");
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
                        let builtin_output = arguments.join(" ");
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
                        let builtin_output = arguments.join(" ");
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
