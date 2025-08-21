use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarGroup {
    pub files: Vec<String>,
    pub similarity_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub groups: Vec<SimilarGroup>,
    pub ungrouped_files: Vec<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn analyze_folder(folder_path: String) -> Result<SimilarityResult, String> {
    // Get the path to the similarity-checker executable
    let exe_path = get_similarity_checker_path()
        .map_err(|e| format!("Failed to find similarity-checker executable: {}", e))?;
    
    // Run the similarity-checker CLI
    let output = Command::new(exe_path)
        .arg("--discover")
        .arg(&folder_path)
        .arg("--format")
        .arg("json")
        .arg("--show-ungrouped")
        .output()
        .map_err(|e| format!("Failed to execute similarity-checker: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Similarity checker failed: {}", stderr));
    }
    
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 output: {}", e))?;
    
    // Parse the JSON output and convert relative paths to absolute ones
    let mut result = parse_similarity_output(&stdout)
        .map_err(|e| format!("Failed to parse similarity checker output: {}", e))?;
    
    // Convert relative paths to absolute paths
    let folder_path_buf = std::path::Path::new(&folder_path);
    
    for group in &mut result.groups {
        for file in &mut group.files {
            let absolute_path = folder_path_buf.join(&*file);
            *file = absolute_path.to_string_lossy().to_string();
        }
    }
    
    for file in &mut result.ungrouped_files {
        let absolute_path = folder_path_buf.join(&*file);
        *file = absolute_path.to_string_lossy().to_string();
    }
    
    Ok(result)
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

fn get_similarity_checker_path() -> Result<PathBuf> {
    // Try to find the executable in common locations
    let possible_paths = vec![
        // In the parent directory (development setup) - absolute path
        "/Users/samzhao/Documents/Code/Playground/similarity-checker/target/release/similarity-checker",
        "/Users/samzhao/Documents/Code/Playground/similarity-checker/target/debug/similarity-checker",
        // Relative paths from GUI directory
        "../../target/release/similarity-checker",
        "../../target/debug/similarity-checker",
        "../target/release/similarity-checker", 
        "../target/debug/similarity-checker",
        // System-wide installation
        "similarity-checker",
        // In current directory
        "./similarity-checker",
    ];
    
    for path in possible_paths {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Ok(path_buf);
        }
    }
    
    // Try using 'which' command as fallback
    match which::which("similarity-checker") {
        Ok(path) => Ok(path),
        Err(_) => Err(anyhow!("similarity-checker executable not found. Please ensure it's installed and in PATH.")),
    }
}

fn parse_similarity_output(json_str: &str) -> Result<SimilarityResult> {
    let parsed: serde_json::Value = serde_json::from_str(json_str)?;
    
    let mut groups = Vec::new();
    let mut ungrouped_files = Vec::new();
    
    if let Some(groups_array) = parsed.get("groups").and_then(|v| v.as_array()) {
        for group in groups_array {
            if let Some(files_array) = group.get("files").and_then(|v| v.as_array()) {
                let files: Vec<String> = files_array
                    .iter()
                    .filter_map(|f| f.as_str().map(|s| s.to_string()))
                    .collect();
                
                let similarity_score = group.get("similarity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                
                if !files.is_empty() {
                    groups.push(SimilarGroup {
                        files,
                        similarity_score,
                    });
                }
            }
        }
    }
    
    // Get actual ungrouped file names
    if let Some(ungrouped_array) = parsed.get("ungrouped").and_then(|v| v.as_array()) {
        ungrouped_files = ungrouped_array
            .iter()
            .filter_map(|f| f.as_str().map(|s| s.to_string()))
            .collect();
    }
    
    Ok(SimilarityResult {
        groups,
        ungrouped_files,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, analyze_folder, delete_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
