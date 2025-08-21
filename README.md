# Similarity Checker

A fast CLI tool that groups files based on name similarity.

## Features

- Multiple similarity algorithms (Levenshtein, Jaro-Winkler, token-based, auto)
- Configurable similarity thresholds
- Multiple output formats (text, JSON, CSV)
- File discovery from directories
- Input from files or stdin
- Progress bars for large datasets
- Colored output for better readability

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/similarity-checker`.

## Usage

### Basic Usage

```bash
# Analyze specific files
similarity-checker file1.txt file2.txt file3.txt

# From a file containing filenames
similarity-checker --input-file filelist.txt

# Discover files in a directory
similarity-checker --discover /path/to/directory

# With custom threshold and algorithm
similarity-checker --threshold 80 --algorithm jaro *.pdf

# Output to JSON
similarity-checker --format json --output results.json *.jpg
```

### Command Line Options

- `--threshold, -t`: Similarity threshold percentage (0-100, default: 70)
- `--algorithm, -a`: Algorithm (levenshtein, jaro, token, auto, default: auto)
- `--format, -f`: Output format (text, json, csv, default: text)
- `--output, -o`: Output file (default: stdout)
- `--input-file, -i`: Read file names from file
- `--discover, -d`: Discover files in directory
- `--min-group-size`: Minimum files per group (default: 2)
- `--case-sensitive`: Enable case-sensitive matching
- `--help, -h`: Show help
- `--version, -v`: Show version

### Examples

```bash
# Group similar PDF reports
similarity-checker --threshold 85 report_*.pdf

# Find similar images with token-based algorithm
similarity-checker --algorithm token --format json *.jpg *.png

# Analyze all files in a project directory
similarity-checker --discover ./src --min-group-size 3

# Read filenames from stdin
find . -name "*.rs" | similarity-checker --format csv
```

## Algorithms

### Levenshtein Distance
- Character-level edit distance
- Good for typos and minor variations
- Example: "file1.txt" vs "file2.txt"

### Jaro-Winkler
- Optimized for string similarity
- Gives more weight to common prefixes
- Good for names with common beginnings

### Token-Based
- Splits on delimiters (_, -, space, numbers)
- Compares token sets using Jaccard similarity
- Good for structured filenames
- Example: "report_2024_final.pdf" vs "report_2024_draft.pdf"

### Auto Mode (Recommended)
- Intelligently combines multiple algorithms
- Adapts based on filename patterns
- Best overall accuracy

## Output Formats

### Text (Default)
Human-readable grouped output with colors and summary statistics.

### JSON
Machine-readable format for programmatic use:
```json
{
  "groups": [
    {
      "id": 1,
      "similarity": 85,
      "files": ["file1.txt", "file2.txt"]
    }
  ],
  "ungrouped": ["different.doc"],
  "summary": {
    "total_files": 3,
    "groups_found": 1,
    "ungrouped_files": 1,
    "threshold_used": 0.7
  }
}
```

### CSV
Spreadsheet-friendly format with columns: group_id, file_name, similarity, status.

## Performance

- Handles up to 10,000 files efficiently
- Progress bars for large datasets
- Memory usage scales linearly with input size
- Optimized string algorithms

## Testing

```bash
cargo test
```

## License

MIT