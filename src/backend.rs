//! Backend module containing Sudoku game logic and state management

#[derive(Clone, PartialEq, Debug)]
pub enum MoveType {
    Input,
    Clear,
    Hint,
}

#[derive(Clone, PartialEq, Debug)]
pub struct GameMove {
    pub row: usize,
    pub col: usize,
    pub old_value: Option<u8>,
    pub new_value: Option<u8>,
    pub move_type: MoveType,
    pub timestamp: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn numbers_to_remove(&self) -> (usize, usize) {
        match self {
            Difficulty::VeryEasy => (31, 36), // Keep 45-50 numbers (very easy)
            Difficulty::Easy => (36, 41),     // Keep 40-45 numbers (easy)
            Difficulty::Medium => (41, 46),   // Keep 35-40 numbers (medium)
            Difficulty::Hard => (46, 51),     // Keep 30-35 numbers (hard)
        }
    }


}

#[derive(Clone, PartialEq)]
pub struct SudokuGame {
    pub grid: [[Option<u8>; 9]; 9],
    pub initial_grid: [[Option<u8>; 9]; 9],
    pub selected_cell: Option<(usize, usize)>,
    pub highlighted_number: Option<u8>,
    // Notes: track which numbers are noted in each cell
    pub notes: [[std::collections::HashSet<u8>; 9]; 9],
    // Optimization: track available numbers for each constraint
    row_available: [std::collections::HashSet<u8>; 9],
    col_available: [std::collections::HashSet<u8>; 9],
    box_available: [std::collections::HashSet<u8>; 9],
    // Undo/Redo system
    pub move_history: Vec<GameMove>,
    pub current_move_index: Option<usize>,
}

impl SudokuGame {
    pub fn new() -> Self {
        Self::generate_random_puzzle(Difficulty::Easy)
    }

    pub fn new_with_difficulty(difficulty: Difficulty) -> Self {
        Self::generate_random_puzzle(difficulty)
    }

    fn generate_random_puzzle(difficulty: Difficulty) -> Self {
        // Start with an empty grid
        let mut grid = [[None; 9]; 9];

        // Fill the grid with a valid complete solution
        Self::fill_grid(&mut grid);

        // Create the puzzle by removing numbers based on difficulty
        let initial_grid = Self::create_puzzle_from_solution(grid, difficulty);

        let mut game = Self {
            grid: initial_grid,
            initial_grid: initial_grid,
            selected_cell: None,
            highlighted_number: None,
            notes: Default::default(),
            row_available: Default::default(),
            col_available: Default::default(),
            box_available: Default::default(),
            move_history: Vec::new(),
            current_move_index: None,
        };
        game.initialize_constraint_sets();
        game
    }

    fn fill_grid(grid: &mut [[Option<u8>; 9]; 9]) -> bool {
        // Simple backtracking algorithm to fill the grid
        for row in 0..9 {
            for col in 0..9 {
                if grid[row][col].is_none() {
                    // Try numbers 1-9 in random order
                    let mut numbers: Vec<u8> = (1..=9).collect();

                    // Simple shuffle using current time as seed
                    let seed = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as usize;

                    for i in 0..numbers.len() {
                        let j = (seed + i * 7) % numbers.len();
                        numbers.swap(i, j);
                    }

                    for &num in &numbers {
                        if Self::is_valid_placement(grid, row, col, num) {
                            grid[row][col] = Some(num);
                            if Self::fill_grid(grid) {
                                return true;
                            }
                            grid[row][col] = None;
                        }
                    }
                    return false;
                }
            }
        }
        true
    }

    fn is_valid_placement(grid: &[[Option<u8>; 9]; 9], row: usize, col: usize, num: u8) -> bool {
        // Check row
        for c in 0..9 {
            if grid[row][c] == Some(num) {
                return false;
            }
        }

        // Check column
        for r in 0..9 {
            if grid[r][col] == Some(num) {
                return false;
            }
        }

        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                if grid[r][c] == Some(num) {
                    return false;
                }
            }
        }

        true
    }

    // Optimized version using constraint sets
    fn is_valid_placement_fast(&self, row: usize, col: usize, num: u8) -> bool {
        let box_idx = (row / 3) * 3 + (col / 3);
        self.row_available[row].contains(&num)
            && self.col_available[col].contains(&num)
            && self.box_available[box_idx].contains(&num)
    }

    fn create_puzzle_from_solution(
        mut solution: [[Option<u8>; 9]; 9],
        difficulty: Difficulty,
    ) -> [[Option<u8>; 9]; 9] {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;

        let mut positions: Vec<(usize, usize)> = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                positions.push((row, col));
            }
        }

        // Simple shuffle
        for i in 0..positions.len() {
            let j = (seed + i * 13) % positions.len();
            positions.swap(i, j);
        }

        // Remove numbers one by one, ensuring unique solution
        let (min_remove, max_remove) = difficulty.numbers_to_remove();
        let target_remove = min_remove + (seed % (max_remove - min_remove + 1));

        let mut removed_count = 0;
        for &(row, col) in &positions {
            if removed_count >= target_remove {
                break;
            }

            // Try removing this cell
            let original_value = solution[row][col];
            solution[row][col] = None;

            // Check if the puzzle still has a unique solution
            if Self::has_unique_solution(&solution) {
                removed_count += 1;
            } else {
                // Restore the cell if removing it creates multiple solutions
                solution[row][col] = original_value;
            }
        }

        solution
    }

    pub fn is_valid_move(&self, row: usize, col: usize, num: u8) -> bool {
        // Use optimized constraint checking
        self.is_valid_placement_fast(row, col, num)
    }

    pub fn is_complete(&self) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                if self.grid[row][col].is_none() {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_initial_cell(&self, row: usize, col: usize) -> bool {
        self.initial_grid[row][col].is_some()
    }

    pub fn select_cell(&mut self, row: usize, col: usize) {
        if !self.is_initial_cell(row, col) {
            self.selected_cell = Some((row, col));
        }

        // Set highlighted number based on the clicked cell's value
        self.highlighted_number = self.grid[row][col];
    }


    pub fn is_cell_highlighted(&self, row: usize, col: usize) -> bool {
        if let Some(highlighted) = self.highlighted_number {
            self.grid[row][col] == Some(highlighted)
        } else {
            false
        }
    }

    pub fn input_number(&mut self, num: u8) -> bool {
        if let Some((row, col)) = self.selected_cell {
            if !self.is_initial_cell(row, col) {
                let old_value = self.grid[row][col];

                // Remove old number from constraint sets if exists
                if let Some(old_num) = old_value {
                    self.remove_number_from_constraints(row, col, old_num);
                }

                // Always allow the input, regardless of validity
                self.grid[row][col] = Some(num);
                self.add_number_to_constraints(row, col, num);

                // Clear notes when a number is filled
                self.clear_notes(row, col);

                // Record the move
                self.record_move(row, col, old_value, Some(num), MoveType::Input);

                return true;
            }
        }
        false
    }

    pub fn has_conflicts(&self, row: usize, col: usize) -> bool {
        if let Some(num) = self.grid[row][col] {
            // Check for conflicts in the same row
            for c in 0..9 {
                if c != col && self.grid[row][c] == Some(num) {
                    return true;
                }
            }

            // Check for conflicts in the same column
            for r in 0..9 {
                if r != row && self.grid[r][col] == Some(num) {
                    return true;
                }
            }

            // Check for conflicts in the same 3x3 box
            let box_row = (row / 3) * 3;
            let box_col = (col / 3) * 3;
            for r in box_row..box_row + 3 {
                for c in box_col..box_col + 3 {
                    if (r != row || c != col) && self.grid[r][c] == Some(num) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn clear_selected_cell(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            if !self.is_initial_cell(row, col) {
                let old_value = self.grid[row][col];
                if let Some(num) = old_value {
                    self.remove_number_from_constraints(row, col, num);
                }
                self.grid[row][col] = None;

                // Record the move
                self.record_move(row, col, old_value, None, MoveType::Clear);
            }
        }
    }


    pub fn reset_with_difficulty(&mut self, difficulty: Difficulty) {
        *self = Self::new_with_difficulty(difficulty);
    }

    pub fn solve_one_cell(&mut self) -> bool {
        // First, verify the current puzzle state has a unique solution
        if !Self::has_unique_solution(&self.grid) {
            // Puzzle is invalid - no unique solution exists
            return false;
        }

        // Find the first empty cell that can be solved with only one valid number
        for row in 0..9 {
            for col in 0..9 {
                if self.grid[row][col].is_none() && !self.is_initial_cell(row, col) {
                    let mut valid_numbers = Vec::new();

                    for num in 1..=9 {
                        if self.is_valid_move(row, col, num) {
                            // Test if placing this number still leads to a unique solution
                            let mut temp_grid = self.grid;
                            temp_grid[row][col] = Some(num);

                            if Self::has_unique_solution(&temp_grid) {
                                valid_numbers.push(num);
                            }
                        }
                    }

                    // If there's only one valid number that leads to unique solution, fill it
                    if valid_numbers.len() == 1 {
                        let old_value = self.grid[row][col];
                        self.grid[row][col] = Some(valid_numbers[0]);
                        self.add_number_to_constraints(row, col, valid_numbers[0]);

                        // Record the hint move
                        self.record_move(
                            row,
                            col,
                            old_value,
                            Some(valid_numbers[0]),
                            MoveType::Hint,
                        );

                        return true;
                    }
                }
            }
        }

        // If no cell with single solution found, try to find any cell where
        // only one number leads to the unique solution
        for row in 0..9 {
            for col in 0..9 {
                if self.grid[row][col].is_none() && !self.is_initial_cell(row, col) {
                    let mut solution_preserving_numbers = Vec::new();

                    for num in 1..=9 {
                        if self.is_valid_move(row, col, num) {
                            // Check if this number leads to the unique solution
                            let mut temp_grid = self.grid;
                            temp_grid[row][col] = Some(num);

                            if Self::has_unique_solution(&temp_grid) {
                                solution_preserving_numbers.push(num);
                            }
                        }
                    }

                    // If only one number preserves the unique solution, use it as hint
                    if solution_preserving_numbers.len() == 1 {
                        let old_value = self.grid[row][col];
                        self.grid[row][col] = Some(solution_preserving_numbers[0]);
                        self.add_number_to_constraints(row, col, solution_preserving_numbers[0]);

                        // Record the hint move
                        self.record_move(
                            row,
                            col,
                            old_value,
                            Some(solution_preserving_numbers[0]),
                            MoveType::Hint,
                        );

                        return true;
                    }
                }
            }
        }

        // No helpful hint found - puzzle might be too complex or invalid
        false
    }


    fn has_unique_solution(grid: &[[Option<u8>; 9]; 9]) -> bool {
        let mut solution_count = 0;
        let mut temp_grid = *grid;
        Self::count_solutions(&mut temp_grid, &mut solution_count);
        solution_count == 1
    }

    fn count_solutions(grid: &mut [[Option<u8>; 9]; 9], count: &mut usize) {
        if *count > 1 {
            return; // Early exit if we already found multiple solutions
        }

        // Find the first empty cell
        for row in 0..9 {
            for col in 0..9 {
                if grid[row][col].is_none() {
                    // Try each number 1-9
                    for num in 1..=9 {
                        if Self::is_valid_placement(grid, row, col, num) {
                            grid[row][col] = Some(num);
                            Self::count_solutions(grid, count);
                            grid[row][col] = None;

                            if *count > 1 {
                                return; // Early exit
                            }
                        }
                    }
                    return; // Backtrack
                }
            }
        }

        // If we reach here, we found a complete solution
        *count += 1;
    }

    fn initialize_constraint_sets(&mut self) {
        // Initialize all sets with numbers 1-9
        for i in 0..9 {
            self.row_available[i] = (1..=9).collect();
            self.col_available[i] = (1..=9).collect();
            self.box_available[i] = (1..=9).collect();
        }

        // Remove numbers that are already placed
        for row in 0..9 {
            for col in 0..9 {
                if let Some(num) = self.grid[row][col] {
                    self.add_number_to_constraints(row, col, num);
                }
            }
        }
    }

    fn add_number_to_constraints(&mut self, row: usize, col: usize, num: u8) {
        let box_idx = (row / 3) * 3 + (col / 3);
        self.row_available[row].remove(&num);
        self.col_available[col].remove(&num);
        self.box_available[box_idx].remove(&num);
    }

    fn remove_number_from_constraints(&mut self, row: usize, col: usize, num: u8) {
        let box_idx = (row / 3) * 3 + (col / 3);
        self.row_available[row].insert(num);
        self.col_available[col].insert(num);
        self.box_available[box_idx].insert(num);
    }

    // Undo/Redo functionality
    fn record_move(
        &mut self,
        row: usize,
        col: usize,
        old_value: Option<u8>,
        new_value: Option<u8>,
        move_type: MoveType,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let game_move = GameMove {
            row,
            col,
            old_value,
            new_value,
            move_type,
            timestamp,
        };

        // If we're not at the end of history, truncate future moves
        if let Some(current_index) = self.current_move_index {
            self.move_history.truncate(current_index + 1);
        } else {
            self.move_history.clear();
        }

        self.move_history.push(game_move);
        self.current_move_index = Some(self.move_history.len() - 1);
    }

    pub fn can_undo(&self) -> bool {
        self.current_move_index.is_some()
    }

    pub fn can_redo(&self) -> bool {
        if let Some(current_index) = self.current_move_index {
            current_index + 1 < self.move_history.len()
        } else {
            !self.move_history.is_empty()
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(current_index) = self.current_move_index {
            let game_move = &self.move_history[current_index].clone();

            // Revert the move
            if let Some(old_num) = self.grid[game_move.row][game_move.col] {
                self.remove_number_from_constraints(game_move.row, game_move.col, old_num);
            }

            self.grid[game_move.row][game_move.col] = game_move.old_value;

            if let Some(new_num) = game_move.old_value {
                self.add_number_to_constraints(game_move.row, game_move.col, new_num);
            }

            // Update move index
            if current_index == 0 {
                self.current_move_index = None;
            } else {
                self.current_move_index = Some(current_index - 1);
            }

            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        let next_index = if let Some(current_index) = self.current_move_index {
            current_index + 1
        } else {
            0
        };

        if next_index < self.move_history.len() {
            let game_move = &self.move_history[next_index].clone();

            // Apply the move
            if let Some(old_num) = self.grid[game_move.row][game_move.col] {
                self.remove_number_from_constraints(game_move.row, game_move.col, old_num);
            }

            self.grid[game_move.row][game_move.col] = game_move.new_value;

            if let Some(new_num) = game_move.new_value {
                self.add_number_to_constraints(game_move.row, game_move.col, new_num);
            }

            self.current_move_index = Some(next_index);
            true
        } else {
            false
        }
    }

    pub fn get_move_log(&self) -> Vec<String> {
        self.move_history
            .iter()
            .enumerate()
            .map(|(index, game_move)| {
                let action = match game_move.move_type {
                    MoveType::Input => "Manual input",
                    MoveType::Clear => "Clear cell",
                    MoveType::Hint => "Hint input",
                };

                let position = format!("R{}C{}", game_move.row + 1, game_move.col + 1);
                let value_change = match (game_move.old_value, game_move.new_value) {
                    (None, Some(new)) => format!("→ {}", new),
                    (Some(old), None) => format!("{} → ∅", old),
                    (Some(old), Some(new)) => format!("{} → {}", old, new),
                    (None, None) => "∅ → ∅".to_string(),
                };

                let marker = if Some(index) == self.current_move_index {
                    "► "
                } else if Some(index) < self.current_move_index {
                    "✓ "
                } else {
                    "○ "
                };

                format!("{}{}: {} {}", marker, action, position, value_change)
            })
            .collect()
    }

    // Note management methods
    pub fn toggle_note(&mut self, row: usize, col: usize, num: u8) {
        // Only allow notes in empty cells
        if self.grid[row][col].is_none() {
            if self.notes[row][col].contains(&num) {
                self.notes[row][col].remove(&num);
            } else {
                self.notes[row][col].insert(num);
            }
        }
    }

    pub fn get_notes(&self, row: usize, col: usize) -> &std::collections::HashSet<u8> {
        &self.notes[row][col]
    }

    pub fn clear_notes(&mut self, row: usize, col: usize) {
        self.notes[row][col].clear();
    }


}
