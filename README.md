# PigSudoku - Dioxus Sudoku Game

A Sudoku game built with the Dioxus framework in Rust.

## Todo List

### Phase 1: Minimal App Setup
- [x] 1. Create a minimal app with Dioxus (Hello World)
  - Set up basic Dioxus project structure
  - Implement simple "Hello World" component
  - Configure Cargo.toml with Dioxus dependencies

### Phase 2: Sudoku Game Implementation
- [x] 1. allow user to input via keyboard
- [x] 2. random generate a new game
- [x] 3. use different color for given number and number for user input
- [x] 4. add a solve helper button, per click, fill one empty cell with correct number
- [x] make different level of new game, easy, medium, hard, based on the number of given number, let the player chose when gen new game
- [x] ensure the puzzle has unique solution
- [x] make all the cell clickable, when one cell with number clicked, make all cells with the same number clicked
- [x] when the puzzle is solved, give more vivid congratulation message may be with animation
- [x] when an empty cell clicked, pop up a option cube with number 1-9, when a number clicked, fill the cell with the number, and close the cube. During the pop up of the cube, keyboard input shall be still acceptable.
- [x] It seems when the player type or chose the wrong anwser, the game board will block what the player typed. Don't block, let the player keep inputing until there are obvious errors, when the error happened give some alert mark
- In the right position of the puzzle field, make a log field, show the user's activities, like manually input or hint input of position and number. With the log, the player shall be able to revert or redo every step. We shall have buttons of revert and redo, also treat keyboard input left arrow and up arrow as revert, and right arrow and down arrow as redo.
- give user a progress bar, when it takes a long time to generate puzzle
- when give hint number, also give text format reason
  - read https://www.sudokudragon.com/sudokututorials.htm

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
- ✅ **Keyboard Input**: Use number keys (1-9) to fill cells, arrow keys to navigate
- ✅ **Random Puzzle Generation**: Each new game generates a unique, solvable puzzle
- ✅ **Visual Distinction**: Different colors for given numbers vs user input
- ✅ **Real-time Validation**: Prevents invalid moves according to Sudoku rules
- ✅ **Hint System**: Solve helper button fills one correct cell per click
- ✅ **Game Controls**: Clear cell, new game, and hint functionality
- ✅ **Win Detection**: Congratulations message when puzzle is solved
- ✅ **Responsive UI**: Clean, modern interface with proper styling
- ✅ **Keyboard Navigation**: Full keyboard support for seamless gameplay

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