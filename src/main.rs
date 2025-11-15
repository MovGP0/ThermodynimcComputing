mod queens;
mod sudoku;
mod ui;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use rand::{rngs::StdRng, SeedableRng};
use std::{error::Error, time::Instant};

#[derive(Parser)]
#[command(author, version, about = "Thermodynamic sampling emulation for Sudoku and 8-Queens")]
struct Cli {
    #[command(subcommand)]
    command: PuzzleCommand,
}

#[derive(Subcommand)]
enum PuzzleCommand {
    Sudoku(SudokuArgs),
    Queens(QueensArgs),
}

#[derive(Args, Debug)]
struct SudokuArgs {
    #[arg(long, default_value_t = 48, help = "Number of removed cells (holes)")]
    holes: usize,
    #[arg(long, default_value_t = 250_000, help = "Maximum annealing swaps")]
    max_steps: usize,
    #[arg(long, default_value_t = 2.4, help = "Starting temperature for the sampler")]
    start_temp: f64,
    #[arg(long, default_value_t = 0.9995, help = "Cooling multiplier per swap")]
    cooling_rate: f64,
    #[arg(long, help = "Optional RNG seed for deterministic runs")]
    seed: Option<u64>,
    #[arg(long, help = "Render the final board using ratatui (terminal required)")]
    tui: bool,
}

#[derive(Args, Debug)]
struct QueensArgs {
    #[arg(long, default_value_t = 92, help = "Unique 8-Queens solutions to collect (max 92)")]
    solutions: usize,
    #[arg(long, help = "Return every unique solution (up to 92)")]
    all_solutions: bool,
    #[arg(long, default_value_t = 100_000, help = "Max swaps per restart")]
    max_steps: usize,
    #[arg(long, default_value_t = 2.4, help = "Starting temperature for the sampler")]
    start_temp: f64,
    #[arg(long, default_value_t = 0.995, help = "Cooling multiplier per swap")]
    cooling_rate: f64,
    #[arg(long, help = "Optional RNG seed")]
    seed: Option<u64>,
    #[arg(long, help = "Render latest solution via ratatui")]
    tui: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        PuzzleCommand::Sudoku(args) => run_sudoku(args),
        PuzzleCommand::Queens(args) => run_queens(args),
    }
}

fn run_sudoku(args: SudokuArgs) -> Result<(), Box<dyn Error>> {
    let mut rng = make_rng(args.seed);
    let holes = args.holes.clamp(16, 64);
    let puzzle = sudoku::SudokuPuzzle::with_random_holes(holes, &mut rng);
    println!(
        "{} puzzle generated (holes={}, givens={}, seed={:?})",
        "Sudoku".bright_green().bold(),
        holes,
        sudoku::count_givens(&puzzle.givens),
        args.seed,
    );
    ui::print_given_grid(&puzzle.givens);

    let config = sudoku::SamplerConfig {
        max_steps: args.max_steps,
        start_temp: args.start_temp,
        cooling_rate: args.cooling_rate,
    };

    let start = Instant::now();
    let (solution, stats) = sudoku::solve(&puzzle, &config, &mut rng);
    let duration = start.elapsed();
    let solved = stats.best_energy == 0;

    println!(
        "{} {} after {} swaps ({:.2?})",
        "Result:".bold(),
        if solved {
            "solved".bright_green()
        } else {
            "best effort".yellow()
        },
        stats.steps,
        duration,
    );
    println!(
        "Best energy={} temperature={:.3}",
        stats.best_energy,
        stats.temperature
    );

    let mask = sudoku::conflict_mask(&solution.board);
    ui::print_sudoku_ascii(&solution.board, &puzzle.givens, &mask);

    if args.tui {
        if let Err(err) = ui::render_sudoku_tui(&solution.board, &puzzle.givens, &mask) {
            eprintln!("TUI render failed: {err}");
        }
    }

    Ok(())
}

fn run_queens(args: QueensArgs) -> Result<(), Box<dyn Error>> {
    let mut rng = make_rng(args.seed);
    let target = if args.all_solutions {
        92
    } else {
        args.solutions.clamp(1, 92)
    };
    let config = queens::QueensConfig {
        max_steps: args.max_steps,
        start_temp: args.start_temp,
        cooling_rate: args.cooling_rate,
    };
    let max_restarts = target * 12 + 5;

    let start = Instant::now();
    let result = queens::collect_solutions(target, max_restarts, &config, &mut rng);
    let duration = start.elapsed();

    if result.runs.is_empty() {
        println!("{} no valid placement found", "8-Queens".bright_red().bold());
        return Ok(());
    }

    println!(
        "{} collected {} unique solutions ({} restarts, {} swaps) in {:.2?}",
        "8-Queens".bright_green().bold(),
        result.runs.len(),
        result.restarts,
        result.total_steps,
        duration,
    );

    for (index, solution) in result.runs.iter().enumerate() {
        println!(
            "{} solution #{} after {} swaps",
            "Sampled".bright_blue(),
            index + 1,
            solution.steps,
        );
        let mask = queens::conflict_mask(&solution.state);
        ui::print_queens_ascii(&solution.state, mask);
    }

    if args.tui {
        if let Some(latest) = result.runs.last() {
            let mask = queens::conflict_mask(&latest.state);
            if let Err(err) = ui::render_queens_tui(&latest.state, mask) {
                eprintln!("TUI render failed: {err}");
            }
        }
    }

    Ok(())
}

fn make_rng(seed: Option<u64>) -> StdRng {
    seed.map_or_else(StdRng::from_os_rng, StdRng::seed_from_u64)
}
