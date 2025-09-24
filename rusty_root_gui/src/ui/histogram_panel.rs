//! Histogram information and analysis panel

use eframe::egui;
use root_io::Histogram;

pub struct HistogramPanel {
    // Panel state
}

impl HistogramPanel {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, selected_histogram: &Option<Histogram>) {
        ui.heading("Histogram Analysis");
        
        ui.separator();
        
        if let Some(ref histogram) = selected_histogram {
            // Histogram information
            ui.group(|ui| {
                ui.label(format!("Name: {}", histogram.name));
                ui.label(format!("Title: {}", histogram.title));
                ui.label(format!("Entries: {}", histogram.entries));
                ui.label(format!("Bins: {}", histogram.bins.len()));
                ui.label(format!("Mean: {:.3}", histogram.mean));
                ui.label(format!("Std Dev: {:.3}", histogram.std_dev));
            });
            
            ui.separator();
            
            // Statistics
            ui.heading("Statistics");
            
            let total_weight: f64 = histogram.bins.iter().sum();
            let max_bin = histogram.bins.iter().fold(0.0f64, |a, &b| a.max(b));
            let min_bin = histogram.bins.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            
            ui.group(|ui| {
                ui.label(format!("Total Weight: {:.3}", total_weight));
                ui.label(format!("Max Bin: {:.3}", max_bin));
                ui.label(format!("Min Bin: {:.3}", min_bin));
                
                if !histogram.bin_edges.is_empty() {
                    let x_min = histogram.bin_edges[0];
                    let x_max = *histogram.bin_edges.last().unwrap();
                    ui.label(format!("X Range: [{:.3}, {:.3}]", x_min, x_max));
                }
            });
            
            ui.separator();
            
            // Bin information (first few bins)
            ui.heading("Bin Details (First 10)");
            
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for (i, &bin_content) in histogram.bins.iter().enumerate().take(10) {
                        let bin_center = if i + 1 < histogram.bin_edges.len() {
                            (histogram.bin_edges[i] + histogram.bin_edges[i + 1]) / 2.0
                        } else {
                            0.0
                        };
                        ui.label(format!("Bin {}: center={:.3}, content={:.3}", i, bin_center, bin_content));
                    }
                    
                    if histogram.bins.len() > 10 {
                        ui.label(format!("... and {} more bins", histogram.bins.len() - 10));
                    }
                });
            
        } else {
            ui.label("No histogram selected");
            ui.label("Select a TH1 from the file browser");
        }
    }
}