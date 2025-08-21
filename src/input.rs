use anyhow::{Context, Result};
use glob::glob;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn collect_files(
    cli_files: Vec<String>,
    input_file: Option<PathBuf>,
    discover_dir: Option<PathBuf>,
) -> Result<Vec<String>> {
    let mut all_files = Vec::new();
    
    // Add files from command line arguments
    all_files.extend(cli_files);
    
    // Add files from input file
    if let Some(input_path) = input_file {
        let files_from_file = read_files_from_file(&input_path)
            .with_context(|| format!("Failed to read files from {}", input_path.display()))?;
        all_files.extend(files_from_file);
    }
    
    // Add files from directory discovery
    if let Some(discover_path) = discover_dir {
        let discovered_files = discover_files(&discover_path)
            .with_context(|| format!("Failed to discover files in {}", discover_path.display()))?;
        all_files.extend(discovered_files);
    }
    
    // Read from stdin if no other sources provided
    if all_files.is_empty() {
        let stdin_files = read_files_from_stdin()
            .context("Failed to read files from stdin")?;
        all_files.extend(stdin_files);
    }
    
    // Remove duplicates and filter out empty strings
    all_files.sort();
    all_files.dedup();
    all_files.retain(|f| !f.trim().is_empty());
    
    if all_files.is_empty() {
        anyhow::bail!("No files provided. Use --help for usage information.");
    }
    
    Ok(all_files)
}

fn read_files_from_file(path: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(path)
        .with_context(|| format!("Cannot open file: {}", path.display()))?;
    
    let reader = BufReader::new(file);
    let mut files = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line
            .with_context(|| format!("Error reading line {} from {}", line_num + 1, path.display()))?;
        
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            files.push(trimmed.to_string());
        }
    }
    
    Ok(files)
}

fn read_files_from_stdin() -> Result<Vec<String>> {
    let stdin = io::stdin();
    let mut files = Vec::new();
    
    for line in stdin.lock().lines() {
        let line = line.context("Error reading from stdin")?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            files.push(trimmed.to_string());
        }
    }
    
    Ok(files)
}

fn discover_files(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        anyhow::bail!("Directory does not exist: {}", dir.display());
    }
    
    if !dir.is_dir() {
        anyhow::bail!("Path is not a directory: {}", dir.display());
    }
    
    let pattern = dir.join("**").join("*");
    let pattern_str = pattern.to_string_lossy();
    
    let mut files = Vec::new();
    
    for entry in glob(&pattern_str)
        .with_context(|| format!("Failed to read glob pattern: {}", pattern_str))?
    {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name_str) = file_name.to_str() {
                            files.push(name_str.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Error processing path: {}", e);
            }
        }
    }
    
    Ok(files)
}

pub fn validate_threshold(threshold: u8) -> Result<()> {
    if threshold > 100 {
        anyhow::bail!("Threshold must be between 0 and 100, got: {}", threshold);
    }
    Ok(())
}

pub fn validate_min_group_size(min_size: usize) -> Result<()> {
    if min_size < 2 {
        anyhow::bail!("Minimum group size must be at least 2, got: {}", min_size);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_collect_files_from_cli() {
        let files = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        let result = collect_files(files, None, None).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"file1.txt".to_string()));
    }

    #[test]
    fn test_validate_threshold() {
        assert!(validate_threshold(50).is_ok());
        assert!(validate_threshold(0).is_ok());
        assert!(validate_threshold(100).is_ok());
        assert!(validate_threshold(101).is_err());
    }

    #[test]
    fn test_validate_min_group_size() {
        assert!(validate_min_group_size(2).is_ok());
        assert!(validate_min_group_size(10).is_ok());
        assert!(validate_min_group_size(1).is_err());
    }

    #[test]
    fn test_read_files_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("files.txt");
        
        fs::write(&file_path, "file1.txt\nfile2.txt\n# comment\n\nfile3.txt").unwrap();
        
        let files = read_files_from_file(&file_path).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));
        assert!(files.contains(&"file3.txt".to_string()));
    }

    #[test]
    fn test_discover_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test1.txt");
        let file2 = temp_dir.path().join("test2.txt");
        
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();
        
        let files = discover_files(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"test1.txt".to_string()));
        assert!(files.contains(&"test2.txt".to_string()));
    }
}