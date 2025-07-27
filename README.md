# PigSudoku - Dioxus Sudoku Game

A Sudoku game built with the Dioxus framework in Rust.

## Todo List

### Phase 1: Minimal App Setup
- [x] 1. Create a minimal app with Dioxus (Hello World)
  - Set up basic Dioxus project structure
  - Implement simple "Hello World" component
  - Configure Cargo.toml with Dioxus dependencies

### Phase 2: Sudoku Game Implementation
- [ ] 2. Transform it into a simple Sudoku game
  - Design Sudoku grid component (9x9)
  - Implement game logic (validation, solving)
  - Add user interaction (number input, cell selection)
  - Create game state management
  - Add basic styling and UI

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
│   ├── main.rs          # Entry point
│   ├── app.rs           # Main app component
│   └── components/      # Reusable components
├── assets/              # Static assets
├── Cargo.toml          # Dependencies
└── README.md           # This file
```

## Technologies

- **Dioxus**: Modern Rust GUI framework
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