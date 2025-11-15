use rand::{rngs::StdRng, seq::SliceRandom, Rng};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct QueenRun {
    pub state: [u8; 8],
    pub steps: usize,
}

pub struct QueensConfig {
    pub max_steps: usize,
    pub start_temp: f64,
    pub cooling_rate: f64,
}

pub struct CollectionResult {
    pub runs: Vec<QueenRun>,
    pub restarts: usize,
    pub total_steps: usize,
}

pub fn collect_solutions(
    target: usize,
    max_restarts: usize,
    config: &QueensConfig,
    rng: &mut StdRng,
) -> CollectionResult {
    let mut unique = HashSet::new();
    let mut runs = Vec::new();
    let mut restarts = 0;
    let mut total_steps = 0;

    while unique.len() < target && restarts < max_restarts {
        restarts += 1;
        if let Some(run) = solve_single(config, rng) {
            total_steps += run.steps;
            if unique.insert(run.state) {
                runs.push(run);
            }
        }
    }

    CollectionResult {
        runs,
        restarts,
        total_steps,
    }
}

fn solve_single(config: &QueensConfig, rng: &mut StdRng) -> Option<QueenRun> {
    let mut state = random_queen_state(rng);
    let mut energy = queen_conflict_count(&state);
    let mut temperature = config.start_temp;

    for step in 0..config.max_steps {
        if energy == 0 {
            return Some(QueenRun { state, steps: step });
        }
        let row = rng.random_range(0..8);
        let current = state[row];
        let mut candidate = rng.random_range(0..8);
        while candidate == current {
            candidate = rng.random_range(0..8);
        }
        state[row] = candidate;
        let new_energy = queen_conflict_count(&state);
        let delta = new_energy as i64 - energy as i64;
        let accept = if delta <= 0 {
            true
        } else {
            let probability = (-(delta as f64) / temperature).exp().min(1.0);
            rng.random_bool(probability)
        };
        if accept {
            energy = new_energy;
        } else {
            state[row] = current;
        }
        temperature = (temperature * config.cooling_rate).max(0.25);
    }
    None
}

fn random_queen_state(rng: &mut StdRng) -> [u8; 8] {
    let mut columns: Vec<u8> = (0..8).map(|value| value as u8).collect();
    columns.shuffle(rng);
    let mut state = [0u8; 8];
    for (row, &value) in columns.iter().enumerate() {
        state[row] = value;
    }
    state
}

fn queen_conflict_count(state: &[u8; 8]) -> usize {
    let mut conflicts = 0;
    for i in 0..8 {
        for j in (i + 1)..8 {
            if state[i] == state[j]
                || (state[i] as i16 - state[j] as i16).abs() == (i as i16 - j as i16).abs()
            {
                conflicts += 1;
            }
        }
    }
    conflicts
}

pub fn conflict_mask(state: &[u8; 8]) -> [bool; 8] {
    let mut mask = [false; 8];
    for i in 0..8 {
        for j in (i + 1)..8 {
            if state[i] == state[j]
                || (state[i] as i16 - state[j] as i16).abs() == (i as i16 - j as i16).abs()
            {
                mask[i] = true;
                mask[j] = true;
            }
        }
    }
    mask
}
