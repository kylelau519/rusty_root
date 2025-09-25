# Rusty ROOT

A GUI workstation for analyzing and reviewing CERN ROOT files, written in Rust.
The readme is a milestones, the functions are not implemented yet.

## Milestones

- **ROOT File I/O Library** (`rusty_root_io` crate):
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


### Running the GUI

```bash
cargo run --bin rusty_root
```

The `rusty_root_io` crate provides a pure Rust implementation for reading ROOT files:

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
