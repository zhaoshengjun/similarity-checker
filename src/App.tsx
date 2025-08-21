import {invoke} from "@tauri-apps/api/core";
import {open} from "@tauri-apps/plugin-dialog";
import {useState} from "react";
import "./App.css";

interface SimilarGroup {
  files: string[];
  similarity_score: number;
}

interface SimilarityResult {
  groups: SimilarGroup[];
  ungrouped_files: string[];
}

function App() {
  const [folderPath, setFolderPath] = useState<string>("");
  const [result, setResult] = useState<SimilarityResult | null>(null);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<string>("");

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

  function selectAllInGroup(files: string[]) {
    const newSelected = new Set(selectedFiles);
    files.forEach((file) => newSelected.add(file));
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
          <h2>Similar File Groups</h2>

          {result.groups.length === 0 ? (
            <p>No similar file groups found.</p>
          ) : (
            result.groups.map((group, groupIndex) => (
              <div key={groupIndex} className="group">
                <div className="group-header">
                  <h3>
                    Group {groupIndex + 1} (Similarity:{" "}
                    {(group.similarity_score * 100).toFixed(1)}%)
                  </h3>
                  <button
                    onClick={() => selectAllInGroup(group.files)}
                    className="select-all-button"
                  >
                    Select All in Group
                  </button>
                </div>
                <div className="file-list">
                  {group.files.map((file, fileIndex) => (
                    <div key={fileIndex} className="file-item">
                      <label>
                        <input
                          type="checkbox"
                          checked={selectedFiles.has(file)}
                          onChange={() => toggleFileSelection(file)}
                        />
                        <span className="file-name">{getFileName(file)}</span>
                        <span className="file-path">{file}</span>
                      </label>
                    </div>
                  ))}
                </div>
              </div>
            ))
          )}

          {result.ungrouped_files.length > 0 && (
            <div className="ungrouped">
              <h3>Ungrouped Files ({result.ungrouped_files.length})</h3>
              <details>
                <summary>Show ungrouped files</summary>
                <div className="file-list">
                  {result.ungrouped_files.map((file, index) => (
                    <div key={index} className="file-item ungrouped-file">
                      {getFileName(file)}
                    </div>
                  ))}
                </div>
              </details>
            </div>
          )}
        </div>
      )}
    </main>
  );
}

export default App;
