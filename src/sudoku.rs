use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SudokuState {
    pub board: [[u8; 9]; 9],
}

pub struct SudokuPuzzle {
    pub givens: [[Option<u8>; 9]; 9],
}

impl SudokuPuzzle {
    pub fn with_random_holes(holes: usize, rng: &mut StdRng) -> Self {
        let solution = generate_full_solution(rng);
        let mut givens = [[None; 9]; 9];
        for row in 0..9 {
            for col in 0..9 {
                givens[row][col] = Some(solution[row][col]);
            }
        }

        let mut coords: Vec<(usize, usize)> = (0..9)
            .flat_map(|row| (0..9).map(move |col| (row, col)))
            .collect();
        coords.shuffle(rng);
        let removed = holes.min(81);
        for &(row, col) in coords.iter().take(removed) {
            givens[row][col] = None;
        }

        SudokuPuzzle { givens }
    }

    pub fn random_initial_state(&self, rng: &mut StdRng) -> SudokuState {
        let mut board = [[0u8; 9]; 9];
        for row in 0..9 {
            let mut digits: Vec<u8> = (1..=9).collect();
            for col in 0..9 {
                if let Some(value) = self.givens[row][col] {
                    board[row][col] = value;
                    if let Some(pos) = digits.iter().position(|&digit| digit == value) {
                        digits.remove(pos);
                    }
                }
            }
            digits.shuffle(rng);
            let mut filler = digits.into_iter();
            for col in 0..9 {
                if self.givens[row][col].is_none() {
                    board[row][col] = filler.next().unwrap();
                }
            }
        }
        SudokuState { board }
    }

    fn row_free_positions(&self) -> Vec<Vec<usize>> {
        (0..9)
            .map(|row| {
                self.givens[row]
                    .iter()
                    .enumerate()
                    .filter_map(|(col, value)| if value.is_none() { Some(col) } else { None })
                    .collect()
            })
            .collect()
    }
}

pub struct SamplerConfig {
    pub max_steps: usize,
    pub start_temp: f64,
    pub cooling_rate: f64,
}

pub struct SolveStats {
    pub steps: usize,
    pub best_energy: usize,
    pub temperature: f64,
}

pub fn solve(
    puzzle: &SudokuPuzzle,
    config: &SamplerConfig,
    rng: &mut StdRng,
) -> (SudokuState, SolveStats) {
    let mut state = puzzle.random_initial_state(rng);
    let mut energy = state.energy();
    let mut best_state = state.clone();
    let mut best_energy = energy;
    let mut temperature = config.start_temp;
    let cooling = config.cooling_rate.clamp(0.8, 0.9999);
    let row_free = puzzle.row_free_positions();
    let mut steps = 0;

    for _ in 0..config.max_steps {
        if energy == 0 {
            break;
        }
        steps += 1;
        let row = rng.random_range(0..9);
        if let Some(positions) = row_free.get(row) {
            if positions.len() < 2 {
                continue;
            }
            let idx_a = rng.random_range(0..positions.len());
            let mut idx_b = rng.random_range(0..positions.len());
            while idx_b == idx_a {
                idx_b = rng.random_range(0..positions.len());
            }
            let col_a = positions[idx_a];
            let col_b = positions[idx_b];
            state.board[row].swap(col_a, col_b);
            let new_energy = state.energy();
            let delta = new_energy as i64 - energy as i64;
            let accept = if delta <= 0 {
                true
            } else {
                let probability = (-(delta as f64) / temperature).exp().min(1.0);
                rng.random_bool(probability)
            };
            if accept {
                energy = new_energy;
                if energy < best_energy {
                    best_energy = energy;
                    best_state = state.clone();
                }
            } else {
                state.board[row].swap(col_a, col_b);
            }
            temperature = (temperature * cooling).max(0.25);
        }
    }

    (
        best_state,
        SolveStats {
            steps,
            best_energy,
            temperature,
        },
    )
}

impl SudokuState {
    fn energy(&self) -> usize {
        column_conflicts(&self.board) + box_conflicts(&self.board)
    }
}

pub fn conflict_mask(board: &[[u8; 9]; 9]) -> [[bool; 9]; 9] {
    let mut mask = [[false; 9]; 9];
    for col in 0..9 {
        let mut seen: HashMap<u8, Vec<usize>> = HashMap::new();
        for row in 0..9 {
            seen.entry(board[row][col]).or_default().push(row);
        }
        for rows in seen.values() {
            if rows.len() > 1 {
                for &row in rows {
                    mask[row][col] = true;
                }
            }
        }
    }

    for block_row in 0..3 {
        for block_col in 0..3 {
            let mut seen: HashMap<u8, Vec<(usize, usize)>> = HashMap::new();
            for row in (block_row * 3)..(block_row * 3 + 3) {
                for col in (block_col * 3)..(block_col * 3 + 3) {
                    seen.entry(board[row][col])
                        .or_default()
                        .push((row, col));
                }
            }
            for cells in seen.values() {
                if cells.len() > 1 {
                    for &(row, col) in cells {
                        mask[row][col] = true;
                    }
                }
            }
        }
    }
    mask
}

pub fn count_givens(givens: &[[Option<u8>; 9]; 9]) -> usize {
    givens.iter().flatten().filter(|value| value.is_some()).count()
}

fn column_conflicts(board: &[[u8; 9]; 9]) -> usize {
    let mut conflicts = 0;
    for col in 0..9 {
        let mut counts = [0u8; 10];
        for row in 0..9 {
            let value = board[row][col] as usize;
            counts[value] += 1;
        }
        for &count in counts.iter().skip(1) {
            if count > 1 {
                conflicts += (count - 1) as usize;
            }
        }
    }
    conflicts
}

fn box_conflicts(board: &[[u8; 9]; 9]) -> usize {
    let mut conflicts = 0;
    for block_row in 0..3 {
        for block_col in 0..3 {
            let mut counts = [0u8; 10];
            for row in (block_row * 3)..(block_row * 3 + 3) {
                for col in (block_col * 3)..(block_col * 3 + 3) {
                    let value = board[row][col] as usize;
                    counts[value] += 1;
                }
            }
            for &count in counts.iter().skip(1) {
                if count > 1 {
                    conflicts += (count - 1) as usize;
                }
            }
        }
    }
    conflicts
}

fn generate_full_solution(rng: &mut StdRng) -> [[u8; 9]; 9] {
    let mut row_bands: Vec<usize> = (0..3).collect();
    row_bands.shuffle(rng);
    let mut rows = Vec::with_capacity(9);
    for &band in &row_bands {
        let mut offsets = vec![0, 1, 2];
        offsets.shuffle(rng);
        for offset in offsets {
            rows.push(band * 3 + offset);
        }
    }

    let mut col_bands: Vec<usize> = (0..3).collect();
    col_bands.shuffle(rng);
    let mut cols = Vec::with_capacity(9);
    for &band in &col_bands {
        let mut offsets = vec![0, 1, 2];
        offsets.shuffle(rng);
        for offset in offsets {
            cols.push(band * 3 + offset);
        }
    }

    let mut nums: Vec<u8> = (1..=9).collect();
    nums.shuffle(rng);

    let mut board = [[0u8; 9]; 9];
    for (i, &row) in rows.iter().enumerate() {
        for (j, &col) in cols.iter().enumerate() {
            board[i][j] = nums[pattern(row, col)];
        }
    }
    board
}

fn pattern(row: usize, col: usize) -> usize {
    (3 * (row % 3) + row / 3 + col) % 9
}
