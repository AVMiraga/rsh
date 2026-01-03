/// Enum representing different types of output redirection
#[derive(Clone, Copy)]
pub enum RedirectionKind {
    Stdout,
    Stderr,
    AppendStdout,
    AppendStderr,
}

/// Standard redirections with their operators and kinds
pub const REDIRECTIONS: &[(&[&str], RedirectionKind)] = &[
    (&[">", "1>"], RedirectionKind::Stdout),
    (&["2>"], RedirectionKind::Stderr),
    (&[">>", "1>>"], RedirectionKind::AppendStdout),
    (&["2>>"], RedirectionKind::AppendStderr),
];
