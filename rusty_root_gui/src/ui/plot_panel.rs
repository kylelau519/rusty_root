//! Plotting and visualization panel

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Bar, BarChart};
use root_io::{Tree, Histogram};

pub struct PlotPanel {
    // Panel state
    plot_type: PlotType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlotType {
    Line,
    Scatter,
    Histogram,
}

impl PlotPanel {
    pub fn new() -> Self {
        Self {
            plot_type: PlotType::Line,
        }
    }
    
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        selected_tree: &Option<Tree>,
        selected_histogram: &Option<Histogram>,
    ) {
        ui.heading("Data Visualization");
        
        ui.separator();
        
        // Plot type selector
        ui.horizontal(|ui| {
            ui.label("Plot Type:");
            ui.selectable_value(&mut self.plot_type, PlotType::Line, "Line");
            ui.selectable_value(&mut self.plot_type, PlotType::Scatter, "Scatter");
            ui.selectable_value(&mut self.plot_type, PlotType::Histogram, "Histogram");
        });
        
        ui.separator();
        
        // Main plot area
        let plot_height = ui.available_height() - 50.0;
        
        Plot::new("main_plot")
            .view_aspect(2.0)
            .height(plot_height)
            .show(ui, |plot_ui| {
                // Plot histogram if available
                if let Some(ref histogram) = selected_histogram {
                    self.plot_histogram(plot_ui, histogram);
                }
                
                // Plot tree data if available
                if let Some(ref tree) = selected_tree {
                    self.plot_tree_data(plot_ui, tree);
                }
                
                // Show placeholder if no data
                if selected_histogram.is_none() && selected_tree.is_none() {
                    self.plot_placeholder(plot_ui);
                }
            });
    }
    
    fn plot_histogram(&self, plot_ui: &mut egui_plot::PlotUi, histogram: &Histogram) {
        let plot_data = histogram.get_plot_data();
        
        match self.plot_type {
            PlotType::Histogram => {
                // Create bar chart
                let bars: Vec<Bar> = plot_data
                    .iter()
                    .enumerate()
                    .map(|(i, [x, y])| Bar::new(*x, *y).width(0.8).name(format!("Bin {}", i)))
                    .collect();
                
                let chart = BarChart::new(bars)
                    .color(egui::Color32::BLUE)
                    .name(&histogram.name);
                plot_ui.bar_chart(chart);
            }
            PlotType::Line => {
                let points: PlotPoints = plot_data.into();
                let line = Line::new(points)
                    .color(egui::Color32::BLUE)
                    .name(&histogram.name);
                plot_ui.line(line);
            }
            PlotType::Scatter => {
                let points: PlotPoints = plot_data.into();
                let line = Line::new(points)
                    .color(egui::Color32::BLUE)
                    .style(egui_plot::LineStyle::Dotted { spacing: 10.0 })
                    .name(&histogram.name);
                plot_ui.line(line);
            }
        }
    }
    
    fn plot_tree_data(&self, plot_ui: &mut egui_plot::PlotUi, tree: &Tree) {
        // Plot the first two branches with numeric data
        let branch_names: Vec<_> = tree.branch_names();
        
        if branch_names.len() >= 2 {
            if let (Some(x_branch), Some(y_branch)) = (
                tree.get_branch(&branch_names[0]),
                tree.get_branch(&branch_names[1]),
            ) {
                let plot_data: Vec<[f64; 2]> = x_branch
                    .data
                    .iter()
                    .zip(y_branch.data.iter())
                    .map(|(&x, &y)| [x, y])
                    .collect();
                
                let points: PlotPoints = plot_data.into();
                
                match self.plot_type {
                    PlotType::Scatter => {
                        let line = Line::new(points)
                            .color(egui::Color32::RED)
                            .style(egui_plot::LineStyle::Dotted { spacing: 10.0 })
                            .name(&format!("{} vs {}", y_branch.name, x_branch.name));
                        plot_ui.line(line);
                    }
                    _ => {
                        let line = Line::new(points)
                            .color(egui::Color32::RED)
                            .name(&format!("{} vs {}", y_branch.name, x_branch.name));
                        plot_ui.line(line);
                    }
                }
            }
        } else if let Some(branch) = branch_names.first().and_then(|name| tree.get_branch(name)) {
            // Plot single branch vs index
            let plot_data: Vec<[f64; 2]> = branch
                .data
                .iter()
                .enumerate()
                .map(|(i, &y)| [i as f64, y])
                .collect();
            
            let points: PlotPoints = plot_data.into();
            let line = Line::new(points)
                .color(egui::Color32::GREEN)
                .name(&branch.name);
            plot_ui.line(line);
        }
    }
    
    fn plot_placeholder(&self, plot_ui: &mut egui_plot::PlotUi) {
        // Show a simple sine wave as placeholder
        let points: PlotPoints = (0..=100)
            .map(|i| {
                let x = i as f64 * 0.1;
                [x, x.sin()]
            })
            .collect();
        
        let line = Line::new(points)
            .color(egui::Color32::GRAY)
            .name("Demo: sin(x)");
        plot_ui.line(line);
    }
}