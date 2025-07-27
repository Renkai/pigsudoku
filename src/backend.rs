//! Backend module containing Sudoku game logic and state management

#[derive(Clone, PartialEq)]
pub struct SudokuGame {
    pub grid: [[Option<u8>; 9]; 9],
    pub initial_grid: [[Option<u8>; 9]; 9],
    pub selected_cell: Option<(usize, usize)>,
}

impl SudokuGame {
    pub fn new() -> Self {
        // Create a simple puzzle with some pre-filled numbers
        let mut initial = [[None; 9]; 9];
        
        // Add some initial numbers to create a valid puzzle
        initial[0][0] = Some(5);
        initial[0][1] = Some(3);
        initial[0][4] = Some(7);
        initial[1][0] = Some(6);
        initial[1][3] = Some(1);
        initial[1][4] = Some(9);
        initial[1][5] = Some(5);
        initial[2][1] = Some(9);
        initial[2][2] = Some(8);
        initial[2][7] = Some(6);
        initial[3][0] = Some(8);
        initial[3][4] = Some(6);
        initial[3][8] = Some(3);
        initial[4][0] = Some(4);
        initial[4][3] = Some(8);
        initial[4][5] = Some(3);
        initial[4][8] = Some(1);
        initial[5][0] = Some(7);
        initial[5][4] = Some(2);
        initial[5][8] = Some(6);
        initial[6][1] = Some(6);
        initial[6][6] = Some(2);
        initial[6][7] = Some(8);
        initial[7][3] = Some(4);
        initial[7][4] = Some(1);
        initial[7][5] = Some(9);
        initial[7][8] = Some(5);
        initial[8][4] = Some(8);
        initial[8][7] = Some(7);
        initial[8][8] = Some(9);
        
        Self {
            grid: initial,
            initial_grid: initial,
            selected_cell: None,
        }
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
}