use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "similarity-checker")]
#[command(about = "A CLI tool that groups files based on name similarity")]
#[command(version = "0.1.0")]
pub struct Args {
    /// File names to analyze (can be used multiple times)
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,

    /// Similarity threshold percentage (0-100)
    #[arg(short, long, default_value = "70")]
    pub threshold: u8,

    /// Similarity algorithm to use
    #[arg(short, long, default_value = "auto")]
    pub algorithm: Algorithm,

    /// Output format
    #[arg(short, long, default_value = "text")]
    pub format: OutputFormat,

    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Read file names from file
    #[arg(short, long)]
    pub input_file: Option<PathBuf>,

    /// Discover files in directory
    #[arg(short, long)]
    pub discover: Option<PathBuf>,

    /// Minimum files per group
    #[arg(long, default_value = "2")]
    pub min_group_size: usize,

    /// Enable case-sensitive matching
    #[arg(long)]
    pub case_sensitive: bool,
}

#[derive(Clone, ValueEnum)]
pub enum Algorithm {
    Levenshtein,
    Jaro,
    Token,
    Substring,
    Auto,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}