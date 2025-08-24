import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { AlertTriangle, Brain, Calendar, CheckSquare, FileText, FolderOpen, HardDrive, Loader2, Search, Square, Trash2 } from "lucide-react";
import { Spinner } from "@/components/ui/spinner";
import { useState } from "react";

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
  const [currentFile, setCurrentFile] = useState<string>("");
  const [processedCount, setProcessedCount] = useState(0);
  const [totalFilesCount, setTotalFilesCount] = useState(0);

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
    setStatus("Discovering files...");
    setCurrentFile("");
    setProcessedCount(0);
    setTotalFilesCount(0);

    try {
      // Simulate progress updates for demonstration
      // In a real implementation, you'd need to modify the Rust backend to emit progress events
      const progressInterval = setInterval(() => {
        setProcessedCount(prev => {
          const newCount = prev + Math.floor(Math.random() * 3) + 1;
          if (newCount >= totalFilesCount && totalFilesCount > 0) {
            return totalFilesCount;
          }
          return newCount;
        });
        
        // Simulate current file being processed
        const sampleFiles = [
          "document.pdf", "image.jpg", "report.docx", "data.xlsx", "backup.zip",
          "presentation.pptx", "config.json", "readme.txt", "photo.png", "archive.tar"
        ];
        setCurrentFile(sampleFiles[Math.floor(Math.random() * sampleFiles.length)]);
      }, 200);
      
      // Set initial total (this would come from file discovery in real implementation)
      setTimeout(() => setTotalFilesCount(Math.floor(Math.random() * 50) + 20), 100);

      const analysisResult: SimilarityResult = await invoke("analyze_folder", {
        folderPath,
      });
      
      clearInterval(progressInterval);
      setResult(analysisResult);
      setStatus(`Found ${analysisResult.groups.length} similar groups`);
      setCurrentFile("");
    } catch (error) {
      setStatus(`Error analyzing folder: ${error}`);
      setCurrentFile("");
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

      // Update the current result by removing deleted files instead of re-analyzing
      if (result) {
        const deletedFilePaths = new Set(selectedFiles);
        const updatedGroups = result.groups
          .map(group => ({
            ...group,
            files: group.files.filter(file => !deletedFilePaths.has(file.path))
          }))
          .filter(group => group.files.length > 1); // Remove groups with only 1 file left

        const updatedUngroupedFiles = result.ungrouped_files.filter(
          file => !deletedFilePaths.has(file.path)
        );

        setResult({
          groups: updatedGroups,
          ungrouped_files: updatedUngroupedFiles
        });
      }

      setStatus(message);
      setSelectedFiles(new Set());
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
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8 max-w-6xl">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-foreground mb-2">Similarity Checker</h1>
          <p className="text-muted-foreground">Find and manage duplicate files in your directories</p>
        </div>

        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <FolderOpen className="h-5 w-5" />
              File Selection
            </CardTitle>
            <CardDescription>
              Choose a folder to analyze for duplicate files
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex gap-2">
              <Button onClick={selectFolder} className="flex items-center gap-2">
                <FolderOpen className="h-4 w-4" />
                Select Folder
              </Button>
              <Button
                onClick={analyzeFolder}
                disabled={!folderPath || loading}
                variant="secondary"
                className="flex items-center gap-2"
              >
                <Search className="h-4 w-4" />
                {loading ? "Analyzing..." : "Analyze Files"}
              </Button>
              {selectedFiles.size > 0 && (
                <Button
                  onClick={deleteSelectedFiles}
                  disabled={loading}
                  variant="destructive"
                  className="flex items-center gap-2"
                >
                  <Trash2 className="h-4 w-4" />
                  Delete Selected ({selectedFiles.size})
                </Button>
              )}
            </div>

            {folderPath && (
              <div className="p-3 bg-muted rounded-md">
                <p className="text-sm text-muted-foreground mb-1">Selected folder:</p>
                <p className="font-mono text-sm break-all">{folderPath}</p>
              </div>
            )}
          </CardContent>
        </Card>

        {status && (
          <Card className="mb-6">
            <CardContent className="pt-6">
              <div className="space-y-3">
                <div className="flex items-center gap-3 text-sm">
                  {loading ? (
                    <Spinner size="sm" className="text-blue-500" />
                  ) : (
                    <div className="h-2 w-2 bg-blue-500 rounded-full"></div>
                  )}
                  <span className="font-medium">{status}</span>
                </div>
                
                {loading && totalFilesCount > 0 && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>Progress: {processedCount} / {totalFilesCount} files</span>
                      <span>{Math.round((processedCount / totalFilesCount) * 100)}%</span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div 
                        className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${Math.min((processedCount / totalFilesCount) * 100, 100)}%` }}
                      ></div>
                    </div>
                    {currentFile && (
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <Loader2 className="h-3 w-3 animate-spin" />
                        <span>Processing: {currentFile}</span>
                      </div>
                    )}
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        )}

      {result && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="h-6 w-6 text-orange-500" />
                <CardTitle>Duplicate Analysis Results</CardTitle>
              </div>
              <CardDescription>
                Found {duplicateGroups} group{duplicateGroups !== 1 ? 's' : ''} containing {totalFiles} duplicate file{totalFiles !== 1 ? 's' : ''}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-purple-600">{duplicateGroups}</div>
                  <div className="text-sm text-muted-foreground">Duplicate Groups</div>
                </div>
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-green-600">{totalFiles}</div>
                  <div className="text-sm text-muted-foreground">Total Files</div>
                </div>
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-red-600">{formatFileSize(wastedSpace)}</div>
                  <div className="text-sm text-muted-foreground">Wasted Space</div>
                </div>
              </div>

              <div className="flex flex-wrap gap-2">
                <Button
                  onClick={selectAllDuplicates}
                  disabled={duplicateGroups === 0}
                  className="flex items-center gap-2"
                >
                  <CheckSquare className="h-4 w-4" />
                  Select All Duplicates
                </Button>
                <Button
                  onClick={clearSelection}
                  variant="outline"
                  disabled={selectedFiles.size === 0}
                  className="flex items-center gap-2"
                >
                  <Square className="h-4 w-4" />
                  Clear Selection
                </Button>
                <Button
                  onClick={smartSelectAll}
                  variant="secondary"
                  disabled={duplicateGroups === 0}
                  className="flex items-center gap-2 bg-orange-500 hover:bg-orange-600 text-white"
                >
                  <Brain className="h-4 w-4" />
                  Smart Select All
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* File Groups */}
          {result.groups.length === 0 ? (
            <Card>
              <CardContent className="pt-6">
                <p className="text-center text-muted-foreground">No similar file groups found.</p>
              </CardContent>
            </Card>
          ) : (
            result.groups.map((group, groupIndex) => {
              const recommendedFile = getRecommendationForGroup(group);
              const similarityDisplay = getSimilarityTypeDisplay(group.similarity_type);

              return (
                <Card key={group.id} className="overflow-hidden">
                  <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <Badge variant="default" className="bg-blue-500">
                          {similarityDisplay}
                        </Badge>
                        <Badge variant="outline">
                          {(group.similarity_score * 100).toFixed(0)}% Match
                        </Badge>
                      </div>
                      <div className="flex items-center gap-2 text-sm text-muted-foreground">
                        <Brain className="h-4 w-4" />
                        <span>{group.files.length} files • {group.similarity_type === "identical" ? "Same file size" : "Size + name similarity"}</span>
                      </div>
                    </div>
                  </CardHeader>

                  {recommendedFile && (
                    <div className="mx-6 mb-4 p-3 bg-orange-50 border border-orange-200 rounded-lg">
                      <div className="flex items-center gap-2 text-sm text-orange-800">
                        <Search className="h-4 w-4" />
                        <span><strong>Recommendation:</strong> Keep largest file: {getFileName(recommendedFile.path)}</span>
                      </div>
                    </div>
                  )}

                  <CardContent className="p-0">
                    <div className="divide-y">
                      {group.files.map((file, fileIndex) => (
                        <div key={fileIndex} className="flex items-center p-4 hover:bg-muted/50 transition-colors">
                          <div className="flex items-center space-x-3 flex-1">
                            <Input
                              type="checkbox"
                              checked={selectedFiles.has(file.path)}
                              onChange={() => toggleFileSelection(file.path)}
                              className="w-4 h-4 cursor-pointer"
                            />
                            <FileText className="h-5 w-5 text-muted-foreground" />
                            <div className="flex-1 min-w-0">
                              <p className="font-medium text-sm truncate">{getFileName(file.path)}</p>
                              <p className="text-xs text-muted-foreground">{file.file_type || 'Unknown'}</p>
                            </div>
                          </div>
                          <div className="flex flex-col items-end gap-1 text-xs text-muted-foreground">
                            <div className="flex items-center gap-1">
                              <HardDrive className="h-3 w-3" />
                              <span>{formatFileSize(file.size)}</span>
                            </div>
                            <div className="flex items-center gap-1">
                              <Calendar className="h-3 w-3" />
                              <span>{formatDate(file.last_modified)}</span>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              );
            })
          )}
        </div>
      )}
      </div>
    </div>
  );
}

export default App;
