//! Example program to create sample ROOT-like data for demonstration
//! 
//! This creates mock ROOT files that can be opened by the GUI application.

use root_io::{Tree, Histogram};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating sample ROOT-like data files...");
    
    // Create a sample tree
    let tree = Tree::create_sample();
    println!("Created sample tree: {}", tree.summary());
    
    // Create a sample histogram
    let histogram = Histogram::create_sample();
    println!("Created sample histogram: {}", histogram.summary());
    
    // Save tree data as JSON for demonstration
    let tree_json = serde_json::to_string_pretty(&tree)?;
    let mut tree_file = File::create("sample_tree.json")?;
    tree_file.write_all(tree_json.as_bytes())?;
    println!("Saved tree data to sample_tree.json");
    
    // Save histogram data as JSON for demonstration
    let hist_json = serde_json::to_string_pretty(&histogram)?;
    let mut hist_file = File::create("sample_histogram.json")?;
    hist_file.write_all(hist_json.as_bytes())?;
    println!("Saved histogram data to sample_histogram.json");
    
    println!("Sample data created successfully!");
    println!("You can now run the GUI with: cargo run --bin rusty_root_gui");
    
    Ok(())
}