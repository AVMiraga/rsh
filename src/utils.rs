/// Find the longest common prefix among a list of strings
pub fn lcp(strings: Vec<String>) -> String {
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
