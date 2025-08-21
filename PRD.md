# Product Requirements Document: File Similarity Checker CLI

## Overview

A command line application that analyzes a list of file names and groups them based on similarity, helping users identify related, duplicate, or similarly named files.

## Problem Statement

Users often have directories with many files that may be related but have slightly different names (e.g., `report_v1.pdf`, `report_final.pdf`, `report_2024.pdf`). Currently, there's no easy way to automatically identify these groups of similar files without manual inspection.

## Goals

### Primary Goals

- Automatically group files based on name similarity
- Provide clear, readable output showing file groups
- Support common similarity detection algorithms
- Handle large lists of files efficiently

### Secondary Goals

- Support different similarity thresholds
- Provide multiple output formats
- Allow customizable similarity algorithms

## Target Users

- Developers managing code repositories with similar file names
- Data analysts working with datasets
- Content managers organizing media files
- Anyone needing to identify duplicate or similar files

## Requirements

### Functional Requirements

#### Core Features

1. **File Input**

   - Accept file names as command line arguments
   - Accept file names from stdin
   - Accept file names from a text file
   - Support glob patterns for file discovery

2. **Similarity Detection**

   - String similarity algorithms (Levenshtein distance, Jaro-Winkler, etc.)
   - Token-based similarity (common words/tokens)
   - Pattern-based similarity (prefixes, suffixes, numbering)
   - File extension awareness

3. **Grouping Logic**

   - Configurable similarity threshold (0-100%)
   - Handle transitive relationships (if A~B and B~C, then A~B~C)
   - Support minimum group size filtering

4. **Output Formats**
   - Human-readable grouped output (default)
   - JSON format for programmatic use
   - CSV format for spreadsheet analysis
   - Summary statistics

#### CLI Interface

```bash
# Basic usage
similarity-checker file1.txt file2.txt file3.txt

# From file list
similarity-checker --input-file files.txt

# With threshold
similarity-checker --threshold 80 *.pdf

# Different output format
similarity-checker --format json --output results.json *.jpg

# Discovery mode
similarity-checker --discover /path/to/directory
```

#### Command Line Options

- `--threshold, -t`: Similarity threshold percentage (default: 70)
- `--algorithm, -a`: Similarity algorithm (levenshtein, jaro, token, auto)
- `--format, -f`: Output format (text, json, csv)
- `--output, -o`: Output file (default: stdout)
- `--input-file, -i`: Read file names from file
- `--discover, -d`: Discover files in directory
- `--min-group-size`: Minimum files per group (default: 2)
- `--case-sensitive`: Enable case-sensitive matching
- `--help, -h`: Show help
- `--version, -v`: Show version

### Non-Functional Requirements

#### Performance

- Handle up to 10,000 files efficiently
- Complete analysis in under 10 seconds for 1,000 files
- Memory usage proportional to input size

#### Usability

- Clear error messages for invalid inputs
- Progress indication for large file sets
- Intuitive command line interface
- Comprehensive help documentation

#### Reliability

- Handle edge cases gracefully (empty inputs, duplicate names)
- Validate file existence when using discovery mode
- Robust error handling and recovery

## Technical Specifications

### Similarity Algorithms

1. **Levenshtein Distance**

   - Character-level edit distance
   - Good for typos and minor variations

2. **Jaro-Winkler**

   - Optimized for string similarity
   - Gives more weight to common prefixes

3. **Token-Based**

   - Split on delimiters (\_, -, space, numbers)
   - Compare token sets
   - Good for differently structured names

4. **Auto Mode**
   - Intelligently choose algorithm based on file patterns
   - Combine multiple algorithms for best results

### Output Examples

#### Text Format (Default)

```
Group 1 (similarity: 85%):
  - report_v1.pdf
  - report_v2.pdf
  - report_final.pdf

Group 2 (similarity: 92%):
  - image001.jpg
  - image002.jpg
  - image003.jpg

Ungrouped files:
  - readme.txt
  - config.json
```

#### JSON Format

```json
{
  "groups": [
    {
      "id": 1,
      "similarity": 85,
      "files": ["report_v1.pdf", "report_v2.pdf", "report_final.pdf"]
    }
  ],
  "ungrouped": ["readme.txt", "config.json"],
  "summary": {
    "total_files": 5,
    "groups_found": 2,
    "ungrouped_files": 2
  }
}
```

## Success Metrics

- **Accuracy**: 90%+ of obviously similar files grouped correctly
- **Performance**: Sub-second response for <100 files
- **Adoption**: Easy enough for non-technical users
- **Flexibility**: Configurable enough for power users

## Future Enhancements

### Phase 2 Features

- Content-based similarity (file hashes, metadata)
- Machine learning-based similarity
- Interactive mode for threshold tuning
- Integration with file managers
- Batch processing capabilities

### Phase 3 Features

- GUI interface
- Cloud storage integration
- Advanced pattern recognition
- Similarity visualization
- API for integration with other tools

## Dependencies

### Technology Stack

- **Language**: Python 3.8+ (for rapid development and rich ecosystem)
- **CLI Framework**: Click or argparse
- **Algorithms**: python-Levenshtein, jellyfish
- **Output**: Built-in json, csv modules

### External Dependencies

- String similarity libraries
- File system utilities
- Progress indicators (tqdm)

## Risks and Mitigation

### Technical Risks

- **Performance with large datasets**: Implement efficient algorithms, add progress indicators
- **Memory usage**: Stream processing, batch handling
- **Algorithm accuracy**: Provide multiple algorithms, allow tuning

### User Experience Risks

- **Complex configuration**: Provide sensible defaults, clear documentation
- **Unexpected groupings**: Show similarity scores, allow threshold adjustment

## Timeline

### MVP (4 weeks)

- Basic CLI with file input
- Levenshtein distance algorithm
- Text output format
- Core grouping logic

### V1.0 (8 weeks)

- Multiple similarity algorithms
- JSON/CSV output formats
- Discovery mode
- Comprehensive testing

### V1.1 (12 weeks)

- Performance optimizations
- Advanced filtering options
- Documentation and examples
