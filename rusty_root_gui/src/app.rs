//! Main application structure and state management

use eframe::egui;
use root_io::{RootReader, Tree, Histogram};
use std::path::PathBuf;

use crate::ui::{FilePanel, TreePanel, HistogramPanel, PlotPanel};

/// Main application state
pub struct RustyRootApp {
    // File management
    current_file: Option<PathBuf>,
    root_reader: Option<RootReader>,
    file_objects: Vec<String>,
    
    // UI panels
    file_panel: FilePanel,
    tree_panel: TreePanel,
    histogram_panel: HistogramPanel,
    plot_panel: PlotPanel,
    
    // Current selections
    selected_tree: Option<Tree>,
    selected_histogram: Option<Histogram>,
    
    // UI state
    show_file_dialog: bool,
    error_message: Option<String>,
}

impl RustyRootApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_file: None,
            root_reader: None,
            file_objects: Vec::new(),
            file_panel: FilePanel::new(),
            tree_panel: TreePanel::new(),
            histogram_panel: HistogramPanel::new(),
            plot_panel: PlotPanel::new(),
            selected_tree: None,
            selected_histogram: None,
            show_file_dialog: false,
            error_message: None,
        }
    }
    
    fn open_file(&mut self, path: PathBuf) {
        match RootReader::open(&path) {
            Ok(reader) => {
                let objects = reader.list_objects();
                self.current_file = Some(path);
                self.file_objects = objects;
                self.root_reader = Some(reader);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open file: {}", e));
            }
        }
    }
    
    fn load_tree(&mut self, name: &str) {
        if let Some(ref reader) = self.root_reader {
            match reader.read_tree(name) {
                Ok(tree) => {
                    self.selected_tree = Some(tree);
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load tree: {}", e));
                }
            }
        }
    }
    
    fn load_histogram(&mut self, name: &str) {
        if let Some(ref reader) = self.root_reader {
            match reader.read_histogram(name) {
                Ok(histogram) => {
                    self.selected_histogram = Some(histogram);
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load histogram: {}", e));
                }
            }
        }
    }
}

impl eframe::App for RustyRootApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main menu bar
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROOT File...").clicked() {
                        self.show_file_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Close File").clicked() {
                        self.current_file = None;
                        self.root_reader = None;
                        self.file_objects.clear();
                        self.selected_tree = None;
                        self.selected_histogram = None;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        // Reset any layout-specific state here
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        // Show about dialog
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Handle file dialog
        if self.show_file_dialog {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("ROOT Files", &["root"])
                .add_filter("All Files", &["*"])
                .pick_file()
            {
                self.open_file(path);
            }
            self.show_file_dialog = false;
        }
        
        // Error display
        if let Some(error) = self.error_message.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, error);
                    if ui.button("OK").clicked() {
                        self.error_message = None;
                    }
                });
        }
        
        // Left panel - File browser and object list
        let mut selected_object: Option<String> = None;
        egui::SidePanel::left("file_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                self.file_panel.show(
                    ui,
                    &self.current_file,
                    &self.file_objects,
                    |name| {
                        selected_object = Some(name.to_string());
                    }
                );
            });
        
        // Handle object selection outside the closure
        if let Some(name) = selected_object {
            if name.contains("TTree") {
                self.load_tree(&name.split_whitespace().next().unwrap_or(&name));
            } else if name.contains("TH1") {
                self.load_histogram(&name.split_whitespace().next().unwrap_or(&name));
            }
        }
        
        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Tree info panel
                ui.group(|ui| {
                    ui.set_width(400.0);
                    self.tree_panel.show(ui, &self.selected_tree);
                });
                
                ui.separator();
                
                // Histogram info panel
                ui.group(|ui| {
                    ui.set_width(400.0);
                    self.histogram_panel.show(ui, &self.selected_histogram);
                });
            });
            
            ui.separator();
            
            // Plot area
            self.plot_panel.show(ui, &self.selected_tree, &self.selected_histogram);
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status = if let Some(ref path) = self.current_file {
                    format!("File: {}", path.display())
                } else {
                    "No file loaded".to_string()
                };
                ui.label(status);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Objects: {}", self.file_objects.len()));
                });
            });
        });
    }
}