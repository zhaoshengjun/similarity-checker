use serde::{Deserialize, Serialize};
use anyhow::Result;

// Import CLI modules
mod cli;
mod similarity;
mod grouper;
mod input;
mod output;
mod file_info;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfoResult {
    pub groups: Vec<file_info::SimilarityGroup>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn analyze_files_advanced(file_paths: Vec<String>) -> Result<FileInfoResult, String> {
    use crate::file_info::{FileInfo, group_similar_files};
    use std::path::Path;
    
    // Convert file paths to FileInfo objects
    let mut files = Vec::new();
    for path_str in file_paths {
        let path = Path::new(&path_str);
        if path.exists() && path.is_file() {
            match FileInfo::from_path(path) {
                Ok(file_info) => files.push(file_info),
                Err(e) => eprintln!("Warning: Failed to process file {}: {}", path_str, e),
            }
        }
    }
    
    // Group similar files
    let groups = group_similar_files(files).await
        .map_err(|e| format!("Failed to group files: {}", e))?;
    
    Ok(FileInfoResult { groups })
}

#[tauri::command]
async fn analyze_folder(folder_path: String) -> Result<FileInfoResult, String> {
    use crate::input::FileDiscovery;
    use crate::file_info::{FileInfo, group_similar_files};
    use std::path::Path;

    // Use embedded CLI logic instead of external binary
    let folder_path_buf = std::path::Path::new(&folder_path);

    // Discover files
    let file_discovery = FileDiscovery::new();
    let file_paths = file_discovery.discover_files(folder_path_buf)
        .map_err(|e| format!("Failed to discover files: {}", e))?;

    // Convert file paths to FileInfo objects
    let mut files = Vec::new();
    for path_str in file_paths {
        let path = folder_path_buf.join(&path_str);
        if path.exists() && path.is_file() {
            match FileInfo::from_path(&path) {
                Ok(file_info) => files.push(file_info),
                Err(e) => eprintln!("Warning: Failed to process file {}: {}", path.display(), e),
            }
        }
    }

    // Group similar files
    let groups = group_similar_files(files).await
        .map_err(|e| format!("Failed to group files: {}", e))?;

    Ok(FileInfoResult { groups })
}

#[tauri::command]
async fn delete_files(file_paths: Vec<String>) -> Result<String, String> {
    let mut deleted_count = 0;
    let mut errors = Vec::new();

    for path in file_paths {
        match trash::delete(&path) {
            Ok(_) => deleted_count += 1,
            Err(e) => errors.push(format!("Failed to delete '{}': {}", path, e)),
        }
    }

    if !errors.is_empty() {
        Err(format!("Some files could not be deleted: {}", errors.join(", ")))
    } else {
        Ok(format!("Successfully deleted {} file(s) to trash", deleted_count))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, analyze_folder, analyze_files_advanced, delete_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
