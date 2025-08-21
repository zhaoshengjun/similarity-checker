mod cli;
mod similarity;
mod grouper;
mod output;
mod input;

use clap::Parser;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{self, BufWriter};

use cli::Args;
use input::{collect_files, validate_threshold, validate_min_group_size};
use grouper::group_files;
use output::format_output;

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate arguments
    validate_threshold(args.threshold)?;
    validate_min_group_size(args.min_group_size)?;
    
    // Collect all input files
    let files = collect_files(args.files, args.input_file, args.discover)?;
    
    if files.len() < args.min_group_size {
        eprintln!("Warning: Only {} files provided, but minimum group size is {}. No groups will be formed.", 
                 files.len(), args.min_group_size);
    }
    
    // Show progress bar for large datasets
    let progress = if files.len() > 100 {
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} files ({eta})")?
                .progress_chars("#>-"),
        );
        pb.set_message("Analyzing file similarities");
        Some(pb)
    } else {
        None
    };
    
    // Perform grouping
    let result = group_files(
        files,
        args.threshold,
        &args.algorithm,
        args.case_sensitive,
        args.min_group_size,
    );
    
    if let Some(pb) = progress {
        pb.finish_with_message("Analysis complete");
    }
    
    // Output results
    match args.output {
        Some(output_path) => {
            let file = File::create(&output_path)?;
            let mut writer = BufWriter::new(file);
            format_output(&result, &args.format, &mut writer)?;
            eprintln!("Results written to: {}", output_path.display());
        }
        None => {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout.lock());
            format_output(&result, &args.format, &mut writer)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_with_basic_files() {
        // This test would require more setup to test the full main function
        // For now, we'll test individual components
        assert!(true);
    }
}