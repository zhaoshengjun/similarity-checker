# File Similarity Analysis

This document explains how the `groupSimilarFiles` function calculates similarity scores for files in the TypeScript implementation.

## Overview

The `groupSimilarFiles` function implements a **three-tier similarity detection system** that groups files based on different types of similarity, prioritizing more reliable indicators over less reliable ones.

## Function Signature

```typescript
export async function groupSimilarFiles(files: FileInfo[]): Promise<SimilarityGroup[]>
```

## Three-Tier Similarity Detection

### Tier 1: Identical Content Detection
**Priority**: Highest
**Method**: SHA-256 hash comparison
**Similarity Type**: `"identical"`
**Similarity Score**: `1.0`

```typescript
if (currentFile.hash === compareFile.hash) {
    similarFiles.push(compareFile);
    // Perfect match - files are identical
}
```

**Logic**: Files with identical hashes are considered the same file, regardless of name differences. This is the most reliable similarity indicator.

### Tier 2: Content Similarity (Size + Name)
**Priority**: Medium
**Method**: Size matching + name similarity
**Similarity Type**: `"content"`
**Threshold**: Name similarity > 0.8
**Similarity Score**: Actual name similarity score (0.8-1.0)

```typescript
if (currentFile.size === compareFile.size) {
    const nameSimilarity = calculateNameSimilarity(currentFile.name, compareFile.name);
    if (nameSimilarity > 0.8) {
        similarFiles.push(compareFile);
        similarityScore = Math.min(similarityScore, nameSimilarity);
    }
}
```

**Logic**: Files with the same size and similar names are likely the same content with minor name variations (e.g., different versions, renamed files).

### Tier 3: Name-Only Similarity
**Priority**: Lowest
**Method**: Pure name comparison
**Similarity Type**: `"name"`
**Threshold**: Name similarity > 0.9
**Similarity Score**: Actual name similarity score (0.9-1.0)

```typescript
const nameSimilarity = calculateNameSimilarity(currentFile.name, compareFile.name);
if (nameSimilarity > 0.9) {
    similarFiles.push(compareFile);
    similarityType = "name";
    similarityScore = Math.min(similarityScore, nameSimilarity);
}
```

**Logic**: Files with very similar names might be related (e.g., slight spelling differences, typos).

## Name Similarity Algorithm

The `calculateNameSimilarity` function uses a **normalized Levenshtein distance** approach:

### Step 1: Normalization
```typescript
const normalize = (str: string) => str.toLowerCase().replace(/[^a-z0-9]/g, "");
```
- Convert to lowercase
- Remove all non-alphanumeric characters
- Examples:
  - `"AI_Usage.epub"` → `"aiusage"`
  - `"Report-Final.pdf"` → `"reportfinal"`

### Step 2: Levenshtein Distance Calculation
```typescript
// Create matrix for dynamic programming
const matrix = Array(n2.length + 1).fill(null).map(() => Array(n1.length + 1).fill(null));

// Initialize base cases
for (let i = 0; i <= n1.length; i++) matrix[0][i] = i;
for (let j = 0; j <= n2.length; j++) matrix[j][0] = j;

// Fill matrix with minimum edit distances
for (let j = 1; j <= n2.length; j++) {
    for (let i = 1; i <= n1.length; i++) {
        const indicator = n1[i - 1] === n2[j - 1] ? 0 : 1;
        matrix[j][i] = Math.min(
            matrix[j][i - 1] + 1,     // Insertion
            matrix[j - 1][i] + 1,     // Deletion
            matrix[j - 1][i - 1] + indicator  // Substitution
        );
    }
}
```

### Step 3: Convert Distance to Similarity
```typescript
const distance = matrix[n2.length][n1.length];
const maxLength = Math.max(n1.length, n2.length);
return maxLength === 0 ? 1.0 : 1 - distance / maxLength;
```

**Formula**: `similarity = 1 - (edit_distance / max_length)`

## Group Scoring Methodology

For groups containing multiple files, the similarity score represents the **minimum similarity** among all file pairs:

```typescript
similarityScore = Math.min(similarityScore, nameSimilarity);
```

**Rationale**: The group score reflects the "weakest link" to ensure the entire group meets the similarity threshold.

## Algorithm Flow

```
1. Calculate SHA-256 hashes for all files
2. For each unprocessed file:
   a. Create new group with current file
   b. For each remaining unprocessed file:
      - Check hash match (Tier 1) → Add to group if match
      - Check size + name similarity (Tier 2) → Add if threshold met
      - Check name similarity only (Tier 3) → Add if threshold met
   c. If group has > 1 file, add to results
3. Sort groups by similarity score (highest first)
```

## Comparison with Rust Implementation

| Aspect | TypeScript (check.ts) | Rust (similarity.rs) |
|--------|----------------------|---------------------|
| **Algorithms** | Levenshtein only | 5 algorithms (Levenshtein, Jaro, Token, Substring, Auto) |
| **Multi-tier Detection** | Yes (hash → size+name → name) | No (single algorithm per call) |
| **Content Awareness** | Yes (SHA-256 hashing) | No (name-only) |
| **Thresholds** | Fixed (0.8, 0.9) | Configurable |
| **Normalization** | Simple (alphanumeric only) | Algorithm-specific |
| **Use Case** | File deduplication | General string similarity |

## Performance Characteristics

- **Time Complexity**: O(n² × m²) where n = number of files, m = average filename length
- **Space Complexity**: O(n × h) where h = hash size (256 bits for SHA-256)
- **Hash Calculation**: O(f) where f = average file size

## Example Usage

```typescript
const files: FileInfo[] = [
    { name: "report.pdf", size: 1024, /* ... */ },
    { name: "report_final.pdf", size: 1024, /* ... */ },
    { name: "document.txt", size: 512, /* ... */ }
];

const groups = await groupSimilarFiles(files);
// Returns groups with similarity types and scores
```

## Key Features

1. **Multi-modal Detection**: Combines content hashing with name similarity
2. **Hierarchical Approach**: More reliable methods take precedence
3. **Robust Normalization**: Handles common filename variations
4. **Conservative Scoring**: Uses minimum similarity for group integrity
5. **Content-Aware**: Detects identical files regardless of naming