use crate::cli::Algorithm;
use std::collections::HashSet;

pub fn calculate_similarity(s1: &str, s2: &str, algorithm: &Algorithm, case_sensitive: bool) -> f64 {
    let (s1, s2) = if case_sensitive {
        (s1.to_string(), s2.to_string())
    } else {
        (s1.to_lowercase(), s2.to_lowercase())
    };

    match algorithm {
        Algorithm::Levenshtein => levenshtein_similarity(&s1, &s2),
        Algorithm::Jaro => jaro_similarity(&s1, &s2),
        Algorithm::Token => token_similarity(&s1, &s2),
        Algorithm::Substring => substring_similarity(&s1, &s2),
        Algorithm::Auto => auto_similarity(&s1, &s2),
    }
}

fn levenshtein_similarity(s1: &str, s2: &str) -> f64 {
    let distance = strsim::levenshtein(s1, s2);
    let max_len = s1.len().max(s2.len());
    if max_len == 0 {
        1.0
    } else {
        1.0 - (distance as f64 / max_len as f64)
    }
}

fn jaro_similarity(s1: &str, s2: &str) -> f64 {
    strsim::jaro_winkler(s1, s2)
}

fn token_similarity(s1: &str, s2: &str) -> f64 {
    let tokens1 = tokenize(s1);
    let tokens2 = tokenize(s2);
    
    if tokens1.is_empty() && tokens2.is_empty() {
        return 1.0;
    }
    if tokens1.is_empty() || tokens2.is_empty() {
        return 0.0;
    }

    let set1: HashSet<_> = tokens1.iter().collect();
    let set2: HashSet<_> = tokens2.iter().collect();
    
    let intersection = set1.intersection(&set2).count();
    let union = set1.union(&set2).count();
    
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            current_token.push(ch);
        } else {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        }
    }
    
    if !current_token.is_empty() {
        tokens.push(current_token);
    }
    
    tokens
}

fn normalize_for_comparison(s: &str) -> String {
    // Remove file extension first
    let without_ext = if let Some(dot_pos) = s.rfind('.') {
        &s[..dot_pos]
    } else {
        s
    };
    
    // Then filter out non-alphanumeric characters and convert to lowercase
    without_ext.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn substring_similarity(s1: &str, s2: &str) -> f64 {
    let norm1 = normalize_for_comparison(s1);
    let norm2 = normalize_for_comparison(s2);
    
    if norm1.is_empty() && norm2.is_empty() {
        return 1.0;
    }
    if norm1.is_empty() || norm2.is_empty() {
        return 0.0;
    }
    
    // Check if one is a substring of the other
    let (shorter, longer) = if norm1.len() <= norm2.len() {
        (&norm1, &norm2)
    } else {
        (&norm2, &norm1)
    };
    
    if longer.contains(shorter) {
        // Return similarity based on length ratio
        shorter.len() as f64 / longer.len() as f64
    } else {
        0.0
    }
}

fn auto_similarity(s1: &str, s2: &str) -> f64 {
    // Use a combination of algorithms and take the maximum
    let levenshtein = levenshtein_similarity(s1, s2);
    let jaro = jaro_similarity(s1, s2);
    let token = token_similarity(s1, s2);
    
    // Weight the algorithms based on string characteristics
    let has_delimiters = s1.contains('_') || s1.contains('-') || s1.contains(' ') ||
                        s2.contains('_') || s2.contains('-') || s2.contains(' ');
    
    if has_delimiters {
        // Prefer token-based for structured names
        token * 0.6 + jaro * 0.3 + levenshtein * 0.1
    } else {
        // Prefer character-based for simple names
        jaro * 0.5 + levenshtein * 0.3 + token * 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_similarity() {
        assert!((levenshtein_similarity("hello", "hello") - 1.0).abs() < f64::EPSILON);
        assert!((levenshtein_similarity("hello", "hallo") - 0.8).abs() < 0.1);
        assert!((levenshtein_similarity("abc", "xyz") - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_jaro_similarity() {
        assert!(jaro_similarity("hello", "hello") > 0.9);
        assert!(jaro_similarity("hello", "hallo") > 0.8);
    }

    #[test]
    fn test_token_similarity() {
        assert!((token_similarity("report_v1.pdf", "report_v2.pdf") - 0.5).abs() < 0.1);
        assert!((token_similarity("file_name_test", "file_name_prod") - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("file_name.txt"), vec!["file", "name", "txt"]);
        assert_eq!(tokenize("report-v1"), vec!["report", "v1"]);
        assert_eq!(tokenize("simple"), vec!["simple"]);
    }

    #[test]
    fn test_normalize_for_comparison() {
        assert_eq!(normalize_for_comparison("AI_Usage.epub"), "aiusage");
        assert_eq!(normalize_for_comparison("AI usage.epub"), "aiusage");
        assert_eq!(normalize_for_comparison("AI_Usage_in_MSFT-Unknown.epub"), "aiusageinmsftunknown");
        assert_eq!(normalize_for_comparison("AI_Usage_in_MSFT.epub"), "aiusageinmsft");
        assert_eq!(normalize_for_comparison("report.pdf"), "report");
        assert_eq!(normalize_for_comparison("report_final.pdf"), "reportfinal");
    }

    #[test]
    fn test_substring_similarity() {
        // Perfect match
        assert!((substring_similarity("test.txt", "test.txt") - 1.0).abs() < f64::EPSILON);
        
        // Substring match with punctuation differences
        assert!((substring_similarity("AI_Usage.epub", "AI usage.epub") - 1.0).abs() < f64::EPSILON);
        
        // One is substring of another - your example case
        let similarity = substring_similarity("AI_Usage_in_MSFT.epub", "AI_Usage_in_MSFT-Unknown.epub");
        assert!(similarity > 0.6, "Expected similarity > 0.6, got {}", similarity);
        
        // Simple substring case
        let similarity = substring_similarity("report.pdf", "report_final.pdf");
        assert!(similarity > 0.5, "Expected similarity > 0.5, got {}", similarity);
        
        // No substring relationship
        assert!((substring_similarity("file1.txt", "document2.pdf") - 0.0).abs() < f64::EPSILON);
        
        // Empty strings
        assert!((substring_similarity("", "") - 1.0).abs() < f64::EPSILON);
        assert!((substring_similarity("test", "") - 0.0).abs() < f64::EPSILON);
    }
}