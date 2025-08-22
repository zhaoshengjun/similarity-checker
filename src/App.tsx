import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import "./App.css";

interface FileInfo {
  name: string;
  size: number;
  file_type: string;
  last_modified: number;
  path: string;
  hash?: string;
}

interface SimilarGroup {
  id: string;
  files: FileInfo[];
  similarity_type: "identical" | "content" | "name";
  similarity_score: number;
}

interface SimilarityResult {
  groups: SimilarGroup[];
  ungrouped_files: FileInfo[];
}

function App() {
  const [folderPath, setFolderPath] = useState<string>("");
  const [result, setResult] = useState<SimilarityResult | null>(null);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<string>("");

  // Calculate statistics
  const totalFiles = result ? result.groups.reduce((sum, group) => sum + group.files.length, 0) : 0;
  const duplicateGroups = result ? result.groups.length : 0;
  const wastedSpace = result ? result.groups.reduce((sum, group) => {
    const maxSize = Math.max(...group.files.map(f => f.size));
    return sum + group.files.reduce((groupSum, file) => file.size === maxSize ? groupSum : groupSum + file.size, 0);
  }, 0) : 0;

  async function selectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        setFolderPath(selected);
        setResult(null);
        setSelectedFiles(new Set());
        setStatus("");
      }
    } catch (error) {
      setStatus(`Error selecting folder: ${error}`);
    }
  }

  async function analyzeFolder() {
    if (!folderPath) {
      setStatus("Please select a folder first");
      return;
    }

    setLoading(true);
    setStatus("Analyzing files...");

    try {
      const analysisResult: SimilarityResult = await invoke("analyze_folder", {
        folderPath,
      });
      setResult(analysisResult);
      setStatus(`Found ${analysisResult.groups.length} similar groups`);
    } catch (error) {
      setStatus(`Error analyzing folder: ${error}`);
    } finally {
      setLoading(false);
    }
  }

  function toggleFileSelection(filePath: string) {
    const newSelected = new Set(selectedFiles);
    if (newSelected.has(filePath)) {
      newSelected.delete(filePath);
    } else {
      newSelected.add(filePath);
    }
    setSelectedFiles(newSelected);
  }

  function selectAllInGroup(files: FileInfo[]) {
    const newSelected = new Set(selectedFiles);
    files.forEach((file) => newSelected.add(file.path));
    setSelectedFiles(newSelected);
  }

  function selectAllDuplicates() {
    if (!result) return;
    const newSelected = new Set<string>();
    result.groups.forEach(group => {
      // Select all but the largest file in each group
      const sortedFiles = [...group.files].sort((a, b) => b.size - a.size);
      sortedFiles.slice(1).forEach(file => newSelected.add(file.path));
    });
    setSelectedFiles(newSelected);
  }

  function clearSelection() {
    setSelectedFiles(new Set());
  }

  function smartSelectAll() {
    if (!result) return;
    const newSelected = new Set<string>();
    result.groups.forEach(group => {
      // Smart selection: keep the file with the most descriptive name or latest date
      const sortedFiles = [...group.files].sort((a, b) => {
        // Prefer files with longer names (more descriptive)
        if (a.name.length !== b.name.length) {
          return b.name.length - a.name.length;
        }
        // Then prefer newer files
        return b.last_modified - a.last_modified;
      });
      sortedFiles.slice(1).forEach(file => newSelected.add(file.path));
    });
    setSelectedFiles(newSelected);
  }

  async function deleteSelectedFiles() {
    if (selectedFiles.size === 0) {
      setStatus("No files selected for deletion");
      return;
    }

    if (
      !confirm(
        `Are you sure you want to delete ${selectedFiles.size} file(s) to trash?`
      )
    ) {
      return;
    }

    setLoading(true);
    setStatus("Deleting files...");

    try {
      const message: string = await invoke("delete_files", {
        filePaths: Array.from(selectedFiles),
      });
      setStatus(message);
      setSelectedFiles(new Set());
      // Re-analyze the folder to update the results
      await analyzeFolder();
    } catch (error) {
      setStatus(`Error deleting files: ${error}`);
    } finally {
      setLoading(false);
    }
  }

  function getFileName(filePath: string): string {
    return filePath.split("/").pop() || filePath;
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit"
    });
  }

  function getSimilarityTypeDisplay(type: string): string {
    switch (type) {
      case "identical": return "IDENTICAL";
      case "content": return "SIZE";
      case "name": return "NAME";
      default: return type.toUpperCase();
    }
  }

  function getRecommendationForGroup(group: SimilarGroup): FileInfo | null {
    if (group.files.length === 0) return null;

    // For identical files, recommend keeping the one with the most descriptive name
    if (group.similarity_type === "identical") {
      return group.files.reduce((best, current) =>
        current.name.length > best.name.length ? current : best
      );
    }

    // For other types, recommend keeping the largest file
    return group.files.reduce((largest, current) =>
      current.size > largest.size ? current : largest
    );
  }

  return (
    <main className="container">
      <h1>Similarity Checker</h1>

      <div className="controls">
        <button onClick={selectFolder}>Select Folder</button>
        {folderPath && (
          <div className="folder-info">
            <strong>Selected:</strong> {folderPath}
          </div>
        )}

        <button onClick={analyzeFolder} disabled={!folderPath || loading}>
          {loading ? "Analyzing..." : "Analyze Files"}
        </button>

        {selectedFiles.size > 0 && (
          <button
            onClick={deleteSelectedFiles}
            disabled={loading}
            className="delete-button"
          >
            Delete Selected ({selectedFiles.size}) to Trash
          </button>
        )}
      </div>

      {status && <div className="status">{status}</div>}

      {result && (
        <div className="results">

          {/* Duplicate Analysis Header */}
          <div className="analysis-header">
            <div className="analysis-title">
              <div className="title-icon">⚠️</div>
              <h2>Duplicate Analysis Results</h2>
            </div>
            <div className="analysis-subtitle">
              Found {duplicateGroups} group{duplicateGroups !== 1 ? 's' : ''} containing {totalFiles} duplicate file{totalFiles !== 1 ? 's' : ''}
            </div>

            {/* Statistics */}
            <div className="stats-row">
              <div className="stat-item">
                <div className="stat-number">{duplicateGroups}</div>
                <div className="stat-label">Duplicate Groups</div>
              </div>
              <div className="stat-item">
                <div className="stat-number">{totalFiles}</div>
                <div className="stat-label">Total Files</div>
              </div>
              <div className="stat-item">
                <div className="stat-number">{formatFileSize(wastedSpace)}</div>
                <div className="stat-label">Wasted Space</div>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="action-buttons">
              <button
                onClick={selectAllDuplicates}
                className="action-btn primary"
                disabled={duplicateGroups === 0}
              >
                ☑️ Select All Duplicates
              </button>
              <button
                onClick={clearSelection}
                className="action-btn secondary"
                disabled={selectedFiles.size === 0}
              >
                ◻️ Clear Selection
              </button>
              <button
                onClick={smartSelectAll}
                className="action-btn smart"
                disabled={duplicateGroups === 0}
              >
                🧠 Smart Select All
              </button>
            </div>
          </div>

          {/* File Groups */}
          {result.groups.length === 0 ? (
            <p>No similar file groups found.</p>
          ) : (
            result.groups.map((group, groupIndex) => {
              const recommendedFile = getRecommendationForGroup(group);
              const similarityDisplay = getSimilarityTypeDisplay(group.similarity_type);

              return (
                <div key={group.id} className="group">
                  <div className="group-header">
                    <div className="group-info">
                      <div className="similarity-badge">
                        <span className="similarity-type">{similarityDisplay}</span>
                        <span className="similarity-percentage">
                          {(group.similarity_score * 100).toFixed(0)}% Match
                        </span>
                      </div>
                      <div className="group-meta">
                        🧠 Smart
                        <span className="file-count">{group.files.length} files • {group.similarity_type === "identical" ? "Same file size" : "Size + name similarity"}</span>
                      </div>
                    </div>
                  </div>

                  {/* Recommendation */}
                  {recommendedFile && (
                    <div className="recommendation">
                      🔍 Recommendation: Keep largest file: {getFileName(recommendedFile.path)}
                    </div>
                  )}

                  <div className="file-list">
                    {group.files.map((file, fileIndex) => (
                      <div key={fileIndex} className="file-item">
                        <div className="file-checkbox">
                          <input
                            type="checkbox"
                            checked={selectedFiles.has(file.path)}
                            onChange={() => toggleFileSelection(file.path)}
                          />
                        </div>

                        <div className="file-icon">
                          📄
                        </div>

                        <div className="file-details">
                          <div className="file-name">{getFileName(file.path)}</div>
                          <div className="file-type">{file.file_type || 'Unknown'}</div>
                        </div>

                        <div className="file-metadata">
                          <div className="file-size">
                            📊 {formatFileSize(file.size)}
                          </div>
                          <div className="file-date">
                            📅 {formatDate(file.last_modified)}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}
    </main>
  );
}

export default App;
