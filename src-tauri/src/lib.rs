use serde::{Deserialize, Serialize};
use anyhow::Result;

// Import CLI modules
mod cli;
mod similarity;
mod grouper;
mod input;
mod output;

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
    use crate::input::FileDiscovery;
    use crate::grouper::FileGrouper;
    use crate::cli::OutputFormat;

    // Use embedded CLI logic instead of external binary
    let folder_path_buf = std::path::Path::new(&folder_path);

    // Discover files
    let file_discovery = FileDiscovery::new();
    let files = file_discovery.discover_files(folder_path_buf)
        .map_err(|e| format!("Failed to discover files: {}", e))?;

    // Group files by similarity
    let mut grouper = FileGrouper::new(0.7); // Use default threshold
    let grouped_files = grouper.group_files(files)
        .map_err(|e| format!("Failed to group files: {}", e))?;

    // Convert to output format
    let output_format = OutputFormat::Json;
    let output_str = output_format.format(&grouped_files, true) // show_ungrouped = true
        .map_err(|e| format!("Failed to format output: {}", e))?;

    // Parse the JSON output and convert relative paths to absolute ones
    let mut result = parse_similarity_output(&output_str)
        .map_err(|e| format!("Failed to parse similarity output: {}", e))?;

    // Convert relative paths to absolute paths
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

// No longer needed since we're using embedded CLI logic

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
