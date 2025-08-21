use crate::cli::OutputFormat;
use crate::grouper::GroupingResult;
use anyhow::Result;
use console::style;
use std::io::Write;

pub fn format_output<W: Write>(
    result: &GroupingResult,
    format: &OutputFormat,
    writer: &mut W,
) -> Result<()> {
    match format {
        OutputFormat::Text => format_text(result, writer),
        OutputFormat::Json => format_json(result, writer),
        OutputFormat::Csv => format_csv(result, writer),
    }
}

fn format_text<W: Write>(result: &GroupingResult, writer: &mut W) -> Result<()> {
    if result.groups.is_empty() {
        writeln!(writer, "{}", style("No similar file groups found.").yellow())?;
    } else {
        for group in &result.groups {
            writeln!(
                writer,
                "{}",
                style(format!(
                    "Group {} (similarity: {:.0}%):",
                    group.id,
                    group.similarity * 100.0
                ))
                .green()
                .bold()
            )?;
            
            for file in &group.files {
                writeln!(writer, "  - {}", file)?;
            }
            writeln!(writer)?;
        }
    }
    
    if !result.ungrouped.is_empty() {
        writeln!(writer, "{}", style("Ungrouped files:").cyan().bold())?;
        for file in &result.ungrouped {
            writeln!(writer, "  - {}", file)?;
        }
        writeln!(writer)?;
    }
    
    // Summary
    writeln!(writer, "{}", style("Summary:").blue().bold())?;
    writeln!(writer, "  Total files: {}", result.summary.total_files)?;
    writeln!(writer, "  Groups found: {}", result.summary.groups_found)?;
    writeln!(writer, "  Ungrouped files: {}", result.summary.ungrouped_files)?;
    writeln!(writer, "  Threshold used: {:.0}%", result.summary.threshold_used * 100.0)?;
    
    Ok(())
}

fn format_json<W: Write>(result: &GroupingResult, writer: &mut W) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

fn format_csv<W: Write>(result: &GroupingResult, writer: &mut W) -> Result<()> {
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    // Write header
    csv_writer.write_record(&["group_id", "file_name", "similarity", "status"])?;
    
    // Write grouped files
    for group in &result.groups {
        for file in &group.files {
            csv_writer.write_record(&[
                group.id.to_string(),
                file.clone(),
                format!("{:.2}", group.similarity),
                "grouped".to_string(),
            ])?;
        }
    }
    
    // Write ungrouped files
    for file in &result.ungrouped {
        csv_writer.write_record(&[
            "".to_string(),
            file.clone(),
            "".to_string(),
            "ungrouped".to_string(),
        ])?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grouper::{Group, Summary};

    fn create_test_result() -> GroupingResult {
        GroupingResult {
            groups: vec![
                Group {
                    id: 1,
                    files: vec!["file1.txt".to_string(), "file2.txt".to_string()],
                    similarity: 0.85,
                },
            ],
            ungrouped: vec!["different.doc".to_string()],
            summary: Summary {
                total_files: 3,
                groups_found: 1,
                ungrouped_files: 1,
                threshold_used: 0.7,
            },
        }
    }

    #[test]
    fn test_format_json() {
        let result = create_test_result();
        let mut output = Vec::new();
        format_json(&result, &mut output).unwrap();
        
        let json_str = String::from_utf8(output).unwrap();
        assert!(json_str.contains("\"id\": 1"));
        assert!(json_str.contains("\"file1.txt\""));
    }

    #[test]
    fn test_format_csv() {
        let result = create_test_result();
        let mut output = Vec::new();
        format_csv(&result, &mut output).unwrap();
        
        let csv_str = String::from_utf8(output).unwrap();
        assert!(csv_str.contains("group_id,file_name,similarity,status"));
        assert!(csv_str.contains("1,file1.txt,0.85,grouped"));
    }
}