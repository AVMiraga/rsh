use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{stdout, Write};

use crate::executor::run_sh;
use crate::utils::lcp;

/// Handle keyboard input loop for the shell
#[allow(clippy::never_loop)]
pub fn input_loop(
    cmds: &[String],
    local_history: &mut Vec<String>,
) -> std::io::Result<()> {
    let mut expect_completions = false;

    loop {
        let mut command = String::new();

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
                        handle_tab_completion(
                            &mut command,
                            cmds,
                            &mut expect_completions,
                        )?;
                    }
                    KeyCode::Char('j') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        disable_raw_mode()?;
                        local_history.push(command.clone());
                        idx = 0;
                        run_sh(&mut command, local_history)?;
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
                        run_sh(&mut command, local_history)?;
                        print!("\r$ ");
                        stdout().flush()?;
                    }
                    KeyCode::Up => {
                        if !local_history.is_empty() && idx < local_history.len() {
                            idx += 1;
                            command = local_history[local_history.len() - idx].clone();
                            print!("\r\x1b[2K$ {}", command);
                            stdout().flush()?;
                        }
                    }
                    KeyCode::Down => {
                        if !local_history.is_empty() && idx > 1 {
                            idx -= 1;
                            command = local_history[local_history.len() - idx].clone();
                            print!("\r\x1b[2K$ {}", command);
                            stdout().flush()?;
                        }
                    }
                    KeyCode::Backspace => {
                        if !command.is_empty() {
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

/// Handle tab completion for commands
fn handle_tab_completion(
    command: &mut String,
    cmds: &[String],
    expect_completions: &mut bool,
) -> std::io::Result<()> {
    if command.is_empty() {
        print!("\x07");
        stdout().flush()?;
        return Ok(());
    }

    let mut possible_cmd: Vec<String> = cmds
        .iter()
        .filter(|x| x.starts_with(command.as_str()))
        .map(|x| x.to_string())
        .collect();

    if possible_cmd.is_empty() {
        print!("\x07");
        stdout().flush()?;
        return Ok(());
    }

    possible_cmd.sort();

    let lcp_possible_command = lcp(possible_cmd.clone());

    if !lcp_possible_command.is_empty()
        && possible_cmd.len() > 1
        && !lcp_possible_command.eq_ignore_ascii_case(command)
    {
        *command = lcp_possible_command;
        print!("\r\x1b[2K$ {}", command);
        stdout().flush()?;
    } else {
        if *expect_completions && possible_cmd.len() > 1 {
            disable_raw_mode()?;
            print!("\r\n");
            print!("{}\n", possible_cmd.join("  "));
            print!("$ {}", command);
            stdout().flush()?;
        } else {
            if possible_cmd.len() > 1 && !*expect_completions {
                *expect_completions = true;
                print!("\x07");
                stdout().flush()?;
            } else {
                *command = possible_cmd[0].to_string() + " ";
                print!("\r\x1b[2K$ {}", command);

                stdout().flush()?;
            }
        }
    }

    Ok(())
}
