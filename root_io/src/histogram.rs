//! ROOT Histogram handling

use crate::error::{Result, RootError};
use serde::{Deserialize, Serialize};

/// Represents a 1D histogram (TH1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub name: String,
    pub title: String,
    pub bins: Vec<f64>,
    pub bin_edges: Vec<f64>,
    pub entries: u64,
    pub mean: f64,
    pub std_dev: f64,
}

impl Histogram {
    /// Create a new histogram
    pub fn new(name: String, title: String, n_bins: usize, x_min: f64, x_max: f64) -> Self {
        let bin_width = (x_max - x_min) / n_bins as f64;
        let bin_edges: Vec<f64> = (0..=n_bins)
            .map(|i| x_min + i as f64 * bin_width)
            .collect();
        let bins = vec![0.0; n_bins];
        
        Self {
            name,
            title,
            bins,
            bin_edges,
            entries: 0,
            mean: 0.0,
            std_dev: 0.0,
        }
    }
    
    /// Fill the histogram with a value
    pub fn fill(&mut self, value: f64, weight: f64) {
        if let Some(bin_index) = self.find_bin(value) {
            self.bins[bin_index] += weight;
            self.entries += 1;
            self.update_statistics();
        }
    }
    
    /// Find the bin index for a given value
    fn find_bin(&self, value: f64) -> Option<usize> {
        if value < self.bin_edges[0] || value >= *self.bin_edges.last().unwrap() {
            return None;
        }
        
        for i in 0..self.bins.len() {
            if value >= self.bin_edges[i] && value < self.bin_edges[i + 1] {
                return Some(i);
            }
        }
        None
    }
    
    /// Update mean and standard deviation
    fn update_statistics(&mut self) {
        let total_weight: f64 = self.bins.iter().sum();
        if total_weight == 0.0 {
            self.mean = 0.0;
            self.std_dev = 0.0;
            return;
        }
        
        // Calculate mean
        let mut weighted_sum = 0.0;
        for (i, &weight) in self.bins.iter().enumerate() {
            let bin_center = (self.bin_edges[i] + self.bin_edges[i + 1]) / 2.0;
            weighted_sum += bin_center * weight;
        }
        self.mean = weighted_sum / total_weight;
        
        // Calculate standard deviation
        let mut variance = 0.0;
        for (i, &weight) in self.bins.iter().enumerate() {
            let bin_center = (self.bin_edges[i] + self.bin_edges[i + 1]) / 2.0;
            variance += weight * (bin_center - self.mean).powi(2);
        }
        self.std_dev = (variance / total_weight).sqrt();
    }
    
    /// Get histogram summary
    pub fn summary(&self) -> String {
        format!(
            "Histogram: {} ({})\nEntries: {}\nMean: {:.3}\nStd Dev: {:.3}\nBins: {}",
            self.name,
            self.title,
            self.entries,
            self.mean,
            self.std_dev,
            self.bins.len()
        )
    }
    
    /// Create a sample histogram for demonstration
    pub fn create_sample() -> Self {
        let mut hist = Histogram::new(
            "sample_hist".to_string(),
            "Sample Histogram".to_string(),
            50,
            -5.0,
            5.0,
        );
        
        // Fill with normal distribution-like data
        for i in 0..1000 {
            let x = -5.0 + 10.0 * i as f64 / 1000.0;
            let weight = (-0.5 * x * x).exp(); // Gaussian-like
            hist.fill(x, weight);
        }
        
        hist
    }
    
    /// Get data points for plotting
    pub fn get_plot_data(&self) -> Vec<[f64; 2]> {
        self.bins.iter().enumerate().map(|(i, &y)| {
            let x = (self.bin_edges[i] + self.bin_edges[i + 1]) / 2.0;
            [x, y]
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_histogram() {
        let hist = Histogram::new("test".to_string(), "Test Hist".to_string(), 10, 0.0, 1.0);
        assert_eq!(hist.name, "test");
        assert_eq!(hist.bins.len(), 10);
        assert_eq!(hist.bin_edges.len(), 11);
        assert_eq!(hist.entries, 0);
    }

    #[test]
    fn test_fill_histogram() {
        let mut hist = Histogram::new("test".to_string(), "Test Hist".to_string(), 10, 0.0, 1.0);
        hist.fill(0.5, 1.0);
        assert_eq!(hist.entries, 1);
        assert!(hist.bins[5] > 0.0);
    }

    #[test]
    fn test_sample_histogram() {
        let hist = Histogram::create_sample();
        assert_eq!(hist.name, "sample_hist");
        assert!(hist.entries > 0);
        assert!(!hist.bins.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_plot_data() {
        let hist = Histogram::create_sample();
        let plot_data = hist.get_plot_data();
        assert_eq!(plot_data.len(), hist.bins.len());
        
        // Check that x values are ordered
        for i in 1..plot_data.len() {
            assert!(plot_data[i][0] > plot_data[i-1][0]);
        }
    }
}