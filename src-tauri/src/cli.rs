#[derive(Clone, Debug)]
pub enum Algorithm {
    Levenshtein,
    Jaro,
    Token,
    Substring,
    Auto,
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}