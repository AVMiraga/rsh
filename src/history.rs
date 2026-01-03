use std::env;

/// Get the command history from the HISTFILE environment variable
pub fn get_history() -> Vec<String> {
    let hist_file_path_env = env::var_os("HISTFILE");

    if let Some(hist_file_path) = hist_file_path_env {
        let file_path = hist_file_path.to_str().unwrap();

        std::fs::read_to_string(file_path)
            .unwrap()
            .lines()
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        Vec::<String>::new()
    }
}
