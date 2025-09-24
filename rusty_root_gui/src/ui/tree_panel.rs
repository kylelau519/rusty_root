//! Tree information and analysis panel

use eframe::egui;
use root_io::Tree;

pub struct TreePanel {
    // Panel state
    selected_branch: Option<String>,
}

impl TreePanel {
    pub fn new() -> Self {
        Self {
            selected_branch: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, selected_tree: &Option<Tree>) {
        ui.heading("Tree Analysis");
        
        ui.separator();
        
        if let Some(ref tree) = selected_tree {
            // Tree information
            ui.group(|ui| {
                ui.label(format!("Name: {}", tree.name));
                ui.label(format!("Title: {}", tree.title));
                ui.label(format!("Entries: {}", tree.entries));
                ui.label(format!("Branches: {}", tree.branches.len()));
            });
            
            ui.separator();
            
            // Branch list
            ui.heading("Branches");
            
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for branch_name in tree.branch_names() {
                        let is_selected = self.selected_branch.as_ref() == Some(&branch_name);
                        if ui.selectable_label(is_selected, &branch_name).clicked() {
                            self.selected_branch = Some(branch_name.clone());
                        }
                    }
                });
            
            // Branch details
            if let Some(ref selected_name) = self.selected_branch {
                if let Some(branch) = tree.get_branch(selected_name) {
                    ui.separator();
                    ui.heading("Branch Details");
                    
                    ui.group(|ui| {
                        ui.label(format!("Name: {}", branch.name));
                        ui.label(format!("Type: {}", branch.data_type));
                        ui.label(format!("Entries: {}", branch.entries));
                        ui.label(format!("Data points: {}", branch.data.len()));
                        
                        if !branch.data.is_empty() {
                            let min = branch.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                            let max = branch.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                            let mean = branch.data.iter().sum::<f64>() / branch.data.len() as f64;
                            
                            ui.label(format!("Min: {:.3}", min));
                            ui.label(format!("Max: {:.3}", max));
                            ui.label(format!("Mean: {:.3}", mean));
                        }
                    });
                }
            }
            
        } else {
            ui.label("No tree selected");
            ui.label("Select a TTree from the file browser");
        }
    }
    
    pub fn selected_branch(&self) -> Option<&String> {
        self.selected_branch.as_ref()
    }
}