use anyhow::{Context, Result};
use glob::glob;
use std::path::{Path, PathBuf};

pub struct FileDiscovery {
    // Empty for now, can add configuration later
}

impl FileDiscovery {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn discover_files(&self, dir: &Path) -> Result<Vec<String>> {
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
}

// Legacy functions kept for backwards compatibility
#[allow(dead_code)]
pub fn collect_files(
    cli_files: Vec<String>,
    _input_file: Option<PathBuf>,
    discover_dir: Option<PathBuf>,
) -> Result<Vec<String>> {
    let mut all_files = Vec::new();
    
    // Add files from command line arguments
    all_files.extend(cli_files);
    
    // Add files from directory discovery
    if let Some(discover_path) = discover_dir {
        let discovery = FileDiscovery::new();
        let discovered_files = discovery.discover_files(&discover_path)
            .with_context(|| format!("Failed to discover files in {}", discover_path.display()))?;
        all_files.extend(discovered_files);
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

#[allow(dead_code)]
pub fn validate_threshold(threshold: u8) -> Result<()> {
    if threshold > 100 {
        anyhow::bail!("Threshold must be between 0 and 100");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_min_group_size(size: usize) -> Result<()> {
    if size < 2 {
        anyhow::bail!("Minimum group size must be at least 2");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn read_files_from_file(file_path: &Path) -> Result<Vec<String>> {
    use std::fs;
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    
    let files: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();
    
    Ok(files)
}

#[allow(dead_code)]
pub fn discover_files(dir: &Path) -> Result<Vec<String>> {
    let discovery = FileDiscovery::new();
    discovery.discover_files(dir)
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