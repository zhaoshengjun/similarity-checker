#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Algorithm {
    Levenshtein,
    Jaro,
    Token,
    Substring,
    Auto,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}