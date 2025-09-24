//! File browser and object list panel

use eframe::egui;
use std::path::PathBuf;

pub struct FilePanel {
    // Panel state
}

impl FilePanel {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn show<F>(
        &mut self,
        ui: &mut egui::Ui,
        current_file: &Option<PathBuf>,
        file_objects: &[String],
        mut on_object_select: F,
    ) where
        F: FnMut(&str),
    {
        ui.heading("File Browser");
        
        ui.separator();
        
        // Current file info
        if let Some(ref path) = current_file {
            ui.group(|ui| {
                ui.label("Current File:");
                ui.label(path.display().to_string());
            });
        } else {
            ui.label("No file loaded");
            ui.label("Use File → Open to load a ROOT file");
        }
        
        ui.separator();
        
        // Object list
        ui.heading("ROOT Objects");
        
        if file_objects.is_empty() {
            ui.label("No objects available");
        } else {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for object in file_objects {
                        if ui.selectable_label(false, object).clicked() {
                            on_object_select(object);
                        }
                    }
                });
        }
    }
}