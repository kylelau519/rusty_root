# Rusty ROOT

A GUI workstation for analyzing and reviewing CERN ROOT files, written in Rust.

## Features

- **ROOT File I/O Library** (`root_io` crate):
  - Read ROOT file headers and validate format
  - Parse ROOT Trees (TTree) with branch data
  - Handle ROOT Histograms (TH1) with statistics
  - High-level reader interface for easy file access
  - Full error handling and type safety

- **GUI Workstation** (`rusty_root_gui`):
  - Cross-platform GUI built with egui
  - File browser for selecting and opening ROOT files
  - Object explorer showing Trees, Histograms, and other ROOT objects
  - Interactive data visualization with multiple plot types
  - Tree analysis with branch inspection and statistics
  - Histogram analysis with bin details and statistics
  - Real-time plotting with line plots, scatter plots, and bar charts

## Quick Start

### Prerequisites

- Rust toolchain (1.70 or later)
- Git

### Building

```bash
git clone <repository-url>
cd rusty_root
cargo build --release
```

### Running the GUI

```bash
cargo run --bin rusty_root_gui
```

### Using the Library

```rust
use root_io::RootReader;

// Open a ROOT file
let reader = RootReader::open("data.root")?;

// List objects in the file
let objects = reader.list_objects();
println!("Objects: {:?}", objects);

// Read a tree
let tree = reader.read_tree("my_tree")?;
println!("Tree has {} entries", tree.entries);

// Read a histogram
let hist = reader.read_histogram("my_histogram")?;
println!("Histogram: {}", hist.summary());
```

## Project Structure

```
rusty_root/
├── root_io/                 # Core ROOT file I/O library
│   ├── src/
│   │   ├── lib.rs          # Library entry point
│   │   ├── error.rs        # Error types
│   │   ├── file.rs         # ROOT file handling
│   │   ├── tree.rs         # TTree implementation
│   │   ├── histogram.rs    # TH1 histogram implementation
│   │   └── reader.rs       # High-level reader interface
│   └── Cargo.toml
├── rusty_root_gui/         # GUI application
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── app.rs          # Main application logic
│   │   └── ui/             # UI components
│   │       ├── mod.rs
│   │       ├── file_panel.rs    # File browser panel
│   │       ├── tree_panel.rs    # Tree analysis panel
│   │       ├── histogram_panel.rs # Histogram analysis panel
│   │       └── plot_panel.rs     # Plotting and visualization
│   └── Cargo.toml
└── Cargo.toml              # Workspace configuration
```

## Features in Detail

### ROOT File I/O Library

The `root_io` crate provides a pure Rust implementation for reading ROOT files:

- **File Format Support**: Reads ROOT file headers and validates magic signatures
- **Tree Support**: Parses TTree objects with branch data and statistics
- **Histogram Support**: Handles TH1 histograms with bin data and statistical analysis
- **Error Handling**: Comprehensive error types for all failure modes
- **Memory Safe**: No unsafe code, leveraging Rust's memory safety guarantees

### GUI Workstation

The GUI application provides a complete analysis environment:

- **File Management**: Open ROOT files through native file dialogs
- **Object Browser**: Navigate and explore ROOT file contents
- **Data Visualization**: Multiple plot types for different data representations
- **Interactive Analysis**: Click-to-select objects and branches
- **Real-time Updates**: Immediate visualization of selected data

## Testing

Run the test suite:

```bash
cargo test
```

The project includes comprehensive tests for:
- ROOT file parsing
- Tree and histogram data structures
- Reader interface functionality
- Mock file creation for testing

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under either the MIT License or the Apache License 2.0, at your option.