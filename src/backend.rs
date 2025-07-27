//! Backend module containing Sudoku game logic and state management

use std::collections::HashSet;

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
            Difficulty::VeryEasy => (25, 35), // Keep 46-56 numbers (very easy)
            Difficulty::Easy => (35, 45),     // Keep 36-46 numbers (easier)
            Difficulty::Medium => (45, 55),   // Keep 26-36 numbers (medium)
            Difficulty::Hard => (55, 65),     // Keep 16-26 numbers (harder)
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::VeryEasy => "Very Easy",
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SudokuGame {
    pub grid: [[Option<u8>; 9]; 9],
    pub initial_grid: [[Option<u8>; 9]; 9],
    pub selected_cell: Option<(usize, usize)>,
    // Optimization: track available numbers for each constraint
    row_available: [std::collections::HashSet<u8>; 9],
    col_available: [std::collections::HashSet<u8>; 9],
    box_available: [std::collections::HashSet<u8>; 9],
}

impl SudokuGame {
    pub fn new() -> Self {
        Self::generate_random_puzzle(Difficulty::Medium)
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
            row_available: Default::default(),
            col_available: Default::default(),
            box_available: Default::default(),
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
    }

    pub fn input_number(&mut self, num: u8) -> bool {
        if let Some((row, col)) = self.selected_cell {
            if !self.is_initial_cell(row, col) {
                if self.is_valid_move(row, col, num) {
                    // Remove old number from constraint sets if exists
                    if let Some(old_num) = self.grid[row][col] {
                        self.remove_number_from_constraints(row, col, old_num);
                    }

                    self.grid[row][col] = Some(num);
                    self.add_number_to_constraints(row, col, num);
                    return true;
                }
            }
        }
        false
    }

    pub fn clear_selected_cell(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            if !self.is_initial_cell(row, col) {
                if let Some(num) = self.grid[row][col] {
                    self.remove_number_from_constraints(row, col, num);
                }
                self.grid[row][col] = None;
            }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
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
                        self.grid[row][col] = Some(valid_numbers[0]);
                        self.add_number_to_constraints(row, col, valid_numbers[0]);
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
                        self.grid[row][col] = Some(solution_preserving_numbers[0]);
                        self.add_number_to_constraints(row, col, solution_preserving_numbers[0]);
                        return true;
                    }
                }
            }
        }

        // No helpful hint found - puzzle might be too complex or invalid
        false
    }

    fn has_valid_solution(grid: &[[Option<u8>; 9]; 9]) -> bool {
        // Simple check: ensure no conflicts exist
        for row in 0..9 {
            for col in 0..9 {
                if let Some(num) = grid[row][col] {
                    // Temporarily remove the number and check if it's valid
                    let mut temp_grid = *grid;
                    temp_grid[row][col] = None;
                    if !Self::is_valid_placement(&temp_grid, row, col, num) {
                        return false;
                    }
                }
            }
        }
        true
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
}
