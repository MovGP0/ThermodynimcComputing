# Thermodynamic Computing Samples

This small CLI project explores thermodynamic sampling heuristics for two classic constraint problems:

1. **Sudoku** — randomly remove given cells (holes) from a solved grid, then anneal to fill them.
2. **8-Queens** — sample conflict-free placements by treating the board as a thermal system and allowing swaps.

Both commands emit diagnostic output to the terminal and optionally render their end states through a `ratatui` grid.

## Getting started
1. **Restore dependencies**: `cargo fetch` (Rust manages crates automatically).
2. **Build**: `cargo build` compiles the binary for later runs and keeps the build artifacts in `target/`.
3. **Execute**: `cargo run -- <mode> [options]` runs either the `sudoku` or `queens` subcommand.

## CLI usage

### Sudoku

```sh
cargo run -- sudoku \
  --holes 48 \
  --max-steps 250000 \
  --start-temp 2.4 \
  --cooling-rate 0.9995 \
  --seed 1234 \
  --tui
```

- `--holes` controls how many givens are removed (clamped between 16 and 64).
- `--max-steps`, `--start-temp`, and `--cooling-rate` tune the annealing sampler.
- `--seed` makes runs deterministic and `--tui` renders the final board via `ratatui`.

### 8-Queens

```sh
cargo run -- queens \
  --solutions 5 \
  --max-steps 100000 \
  --start-temp 2.4 \
  --cooling-rate 0.995 \
  --seed 42 \
  --tui
```

- `--solutions` requests up to 92 unique placements; use `--all-solutions` to collect every known solution.
- The annealing options behave the same as in the Sudoku command, and `--tui` draws the latest valid board.

## Notes
- Running either subcommand without `--tui` leaves output in plain text (givens, best energy, conflict masks).
- Passing `--all-solutions` forces the queens solver to stop only after gathering the full set of 92 valid placements.
