# PigSudoku - Dioxus Sudoku Game

A Sudoku game built with the Dioxus framework in Rust.

## Todo List

### Phase 1: Minimal App Setup
- [x] 1. Create a minimal app with Dioxus (Hello World)
  - Set up basic Dioxus project structure
  - Implement simple "Hello World" component
  - Configure Cargo.toml with Dioxus dependencies

### Phase 2: Sudoku Game Implementation
1. allow user to input via keyboard
2. random generate a new game
3. use different color for given number and number for user input
4. add a solve helper button, per click, fill one empty cell with correct number

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Cargo
- Web browser for testing

### Development

```bash
# Run the app in development mode
cargo run
```

## Project Structure

```
pigsudoku/
├── src/
│   └── main.rs          # Entry point and Sudoku game implementation
├── Cargo.toml          # Dependencies and project configuration
├── Cargo.lock          # Dependency lock file
└── README.md           # This file
```

## Features

- ✅ **Interactive 9x9 Sudoku Grid**: Click cells to select and input numbers
- ✅ **Real-time Validation**: Prevents invalid moves according to Sudoku rules
- ✅ **Visual Feedback**: Selected cells highlighted, initial numbers distinguished
- ✅ **Game Controls**: Clear cell, new game functionality
- ✅ **Win Detection**: Congratulations message when puzzle is solved
- ✅ **Responsive UI**: Clean, modern interface with proper styling

## Technologies

- **Dioxus 0.6**: Modern Rust GUI framework
- **Rust**: Systems programming language
- **Desktop**: Current target platform

## Roadmap

### Future Features
- **Web Compatibility**: Enable web variant of Dioxus
  - Test web compilation and rendering
  - Set up web-specific configurations
  - Verify cross-browser compatibility
  - WebAssembly optimization
- **Mobile Support**: Extend to mobile platforms
- **Advanced Game Features**:
  - Multiple difficulty levels
  - Hint system
  - Timer and scoring
  - Save/load game state
  - Dark/light theme toggle