use crate::cli::Algorithm;
use crate::similarity::calculate_similarity;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use anyhow::Result;

pub struct FileGrouper {
    threshold: f64,
    algorithm: Algorithm,
    case_sensitive: bool,
    min_group_size: usize,
}

impl FileGrouper {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            algorithm: Algorithm::Auto,
            case_sensitive: false,
            min_group_size: 2,
        }
    }
    
    pub fn group_files(&mut self, files: Vec<String>) -> Result<GroupingResult> {
        let threshold_u8 = (self.threshold * 100.0) as u8;
        Ok(group_files(files, threshold_u8, &self.algorithm, self.case_sensitive, self.min_group_size))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: usize,
    pub files: Vec<String>,
    pub similarity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupingResult {
    pub groups: Vec<Group>,
    pub ungrouped: Vec<String>,
    pub summary: Summary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_files: usize,
    pub groups_found: usize,
    pub ungrouped_files: usize,
    pub threshold_used: f64,
}

pub fn group_files(
    files: Vec<String>,
    threshold: u8,
    algorithm: &Algorithm,
    case_sensitive: bool,
    min_group_size: usize,
) -> GroupingResult {
    let threshold_f64 = threshold as f64 / 100.0;
    let mut groups: Vec<Group> = Vec::new();
    let mut processed: HashSet<usize> = HashSet::new();
    
    for i in 0..files.len() {
        if processed.contains(&i) {
            continue;
        }
        
        let mut current_group = vec![i];
        let mut similarities = Vec::new();
        
        // Find all files similar to the current file
        for j in (i + 1)..files.len() {
            if processed.contains(&j) {
                continue;
            }
            
            let similarity = calculate_similarity(
                &files[i],
                &files[j],
                algorithm,
                case_sensitive,
            );
            
            if similarity >= threshold_f64 {
                current_group.push(j);
                similarities.push(similarity);
            }
        }
        
        // Check for transitive relationships within the group
        let mut expanded_group = current_group.clone();
        let mut added_any = true;
        
        while added_any {
            added_any = false;
            for &group_idx in current_group.iter() {
                for k in 0..files.len() {
                    if processed.contains(&k) || expanded_group.contains(&k) {
                        continue;
                    }
                    
                    let similarity = calculate_similarity(
                        &files[group_idx],
                        &files[k],
                        algorithm,
                        case_sensitive,
                    );
                    
                    if similarity >= threshold_f64 {
                        expanded_group.push(k);
                        similarities.push(similarity);
                        added_any = true;
                    }
                }
            }
            current_group = expanded_group.clone();
        }
        
        // Only create a group if it meets the minimum size requirement
        if expanded_group.len() >= min_group_size {
            let avg_similarity = if similarities.is_empty() {
                1.0
            } else {
                similarities.iter().sum::<f64>() / similarities.len() as f64
            };
            
            let group_files: Vec<String> = expanded_group
                .iter()
                .map(|&idx| files[idx].clone())
                .collect();
            
            groups.push(Group {
                id: groups.len() + 1,
                files: group_files,
                similarity: avg_similarity,
            });
            
            // Mark all files in this group as processed
            for &idx in &expanded_group {
                processed.insert(idx);
            }
        } else {
            // Don't mark single files as processed - they should be ungrouped
        }
    }
    
    // Collect ungrouped files
    let ungrouped: Vec<String> = files
        .iter()
        .enumerate()
        .filter_map(|(i, file)| {
            if !processed.contains(&i) {
                Some(file.clone())
            } else {
                None
            }
        })
        .collect();
    
    let summary = Summary {
        total_files: files.len(),
        groups_found: groups.len(),
        ungrouped_files: ungrouped.len(),
        threshold_used: threshold_f64,
    };
    
    // Sort groups by similarity score in descending order
    groups.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    
    GroupingResult {
        groups,
        ungrouped,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Algorithm;

    #[test]
    fn test_group_files_basic() {
        let files = vec![
            "report_v1.pdf".to_string(),
            "report_v2.pdf".to_string(),
            "image001.jpg".to_string(),
            "readme.txt".to_string(),
        ];
        
        let result = group_files(files, 50, &Algorithm::Token, false, 2);
        
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].files.len(), 2);
        assert!(result.groups[0].files.contains(&"report_v1.pdf".to_string()));
        assert!(result.groups[0].files.contains(&"report_v2.pdf".to_string()));
    }

    #[test]
    fn test_min_group_size() {
        let files = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "different.doc".to_string(),
        ];
        
        let result = group_files(files, 70, &Algorithm::Levenshtein, false, 3);
        assert_eq!(result.groups.len(), 0);
        assert_eq!(result.ungrouped.len(), 3);
    }
}