//! Backend module containing Sudoku game logic and state management

#[derive(Clone, PartialEq)]
pub struct SudokuGame {
    pub grid: [[Option<u8>; 9]; 9],
    pub initial_grid: [[Option<u8>; 9]; 9],
    pub selected_cell: Option<(usize, usize)>,
}

impl SudokuGame {
    pub fn new() -> Self {
        Self::generate_random_puzzle()
    }
    
    fn generate_random_puzzle() -> Self {
        // Start with an empty grid
        let mut grid = [[None; 9]; 9];
        
        // Fill the grid with a valid complete solution
        Self::fill_grid(&mut grid);
        
        // Create the puzzle by removing numbers
        let initial_grid = Self::create_puzzle_from_solution(grid);
        
        Self {
            grid: initial_grid,
            initial_grid: initial_grid,
            selected_cell: None,
        }
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
    
    fn create_puzzle_from_solution(mut solution: [[Option<u8>; 9]; 9]) -> [[Option<u8>; 9]; 9] {
        // Remove about 40-50 numbers to create a puzzle
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
        
        // Remove numbers from random positions (keep about 30-35 numbers)
        let numbers_to_remove = 45 + (seed % 10); // Remove 45-54 numbers
        for i in 0..numbers_to_remove.min(positions.len()) {
            let (row, col) = positions[i];
            solution[row][col] = None;
        }
        
        solution
    }
    
    pub fn is_valid_move(&self, row: usize, col: usize, num: u8) -> bool {
        // Check row
        for c in 0..9 {
            if c != col && self.grid[row][c] == Some(num) {
                return false;
            }
        }
        
        // Check column
        for r in 0..9 {
            if r != row && self.grid[r][col] == Some(num) {
                return false;
            }
        }
        
        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                if (r != row || c != col) && self.grid[r][c] == Some(num) {
                    return false;
                }
            }
        }
        
        true
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
                    self.grid[row][col] = Some(num);
                    return true;
                }
            }
        }
        false
    }
    
    pub fn clear_selected_cell(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            if !self.is_initial_cell(row, col) {
                self.grid[row][col] = None;
            }
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn solve_one_cell(&mut self) -> bool {
        // Find the first empty cell that can be solved with only one valid number
        for row in 0..9 {
            for col in 0..9 {
                if self.grid[row][col].is_none() && !self.is_initial_cell(row, col) {
                    let mut valid_numbers = Vec::new();
                    
                    for num in 1..=9 {
                        if self.is_valid_move(row, col, num) {
                            valid_numbers.push(num);
                        }
                    }
                    
                    // If there's only one valid number, fill it
                    if valid_numbers.len() == 1 {
                        self.grid[row][col] = Some(valid_numbers[0]);
                        return true;
                    }
                }
            }
        }
        
        // If no cell with single solution found, try to find any solvable cell
        // using more advanced techniques or just fill the first empty cell with a valid number
        for row in 0..9 {
            for col in 0..9 {
                if self.grid[row][col].is_none() && !self.is_initial_cell(row, col) {
                    for num in 1..=9 {
                        if self.is_valid_move(row, col, num) {
                            // Check if this number leads to a valid solution
                            let mut temp_grid = self.grid;
                            temp_grid[row][col] = Some(num);
                            
                            if Self::has_valid_solution(&temp_grid) {
                                self.grid[row][col] = Some(num);
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
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
}