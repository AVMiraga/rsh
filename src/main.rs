use std::collections::HashSet;
use std::env;
use std::os::unix::fs::MetadataExt;

use codecrafters_shell::builtins::VALID_COMMANDS_BUILTIN;
use codecrafters_shell::history::get_history;
use codecrafters_shell::input::input_loop;

fn main() -> std::io::Result<()> {
    let mut cmds = Vec::<String>::new();
    let mut local_history = Vec::<String>::new();

    // Load existing history
    let existing_history = get_history();
    if !existing_history.is_empty() {
        local_history.extend(existing_history);
    }

    // Build list of available commands from PATH
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

    // Add built-in commands to the list
    cmds.extend(VALID_COMMANDS_BUILTIN.iter().map(|s| s.to_string()));
    
    // Deduplicate commands
    let set_cmds = cmds.into_iter().collect::<HashSet<String>>();
    let cmds = set_cmds.into_iter().collect::<Vec<_>>();

    // Start the input loop
    input_loop(&cmds, &mut local_history)
}

#[cfg(test)]
mod tests {
    use codecrafters_shell::commands::pipeline_handler;

    #[test]
    fn testing() -> anyhow::Result<()> {
        let command = "history -r";
        let _pipelined = pipeline_handler(command);

        Ok(())
    }
}
