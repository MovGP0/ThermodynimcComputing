use colored::Colorize;
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use std::{error::Error, io::stdout};

pub fn print_given_grid(givens: &[[Option<u8>; 9]; 9]) {
    println!("{}", "Sudoku puzzle (givens in cyan)".bright_blue());
    for row in 0..9 {
        if row % 3 == 0 {
            println!("+-------+-------+-------+");
        }
        for col in 0..9 {
            if col % 3 == 0 {
                print!("| ");
            }
            match givens[row][col] {
                Some(value) => print!("{} ", format!("{value}").cyan()),
                None => print!(". "),
            }
        }
        println!("|");
    }
    println!("+-------+-------+-------+");
}

pub fn print_sudoku_ascii(
    board: &[[u8; 9]; 9],
    givens: &[[Option<u8>; 9]; 9],
    mask: &[[bool; 9]; 9],
) {
    println!("{}", "Final Sudoku state".bright_blue());
    for row in 0..9 {
        if row % 3 == 0 {
            println!("+-------+-------+-------+");
        }
        for col in 0..9 {
            if col % 3 == 0 {
                print!("| ");
            }
            let token = format!("{}", board[row][col]);
            let styled = if mask[row][col] {
                token.red().bold()
            } else if givens[row][col].is_some() {
                token.cyan()
            } else {
                token.yellow()
            };
            print!("{} ", styled);
        }
        println!("|");
    }
    println!("+-------+-------+-------+");
}

pub fn print_queens_ascii(state: &[u8; 8], mask: [bool; 8]) {
    for (row, &queen_col) in state.iter().enumerate() {
        for col in 0..8 {
            if col == queen_col as usize {
                let glyph = if mask[row] {
                    "Q".red().bold()
                } else {
                    "Q".green().bold()
                };
                print!("{} ", glyph);
            } else {
                print!(".");
                print!(" ");
            }
        }
        println!();
    }
    println!();
}

pub fn render_sudoku_tui(
    board: &[[u8; 9]; 9],
    givens: &[[Option<u8>; 9]; 9],
    mask: &[[bool; 9]; 9],
) -> Result<(), Box<dyn Error>> {
    let cells: Vec<Vec<Cell>> = board
        .iter()
        .enumerate()
        .map(|(row, line)| {
            line.iter()
                .enumerate()
                .map(|(col, &value)| {
                    let style = if mask[row][col] {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    } else if givens[row][col].is_some() {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    Cell::from(Span::styled(format!("{value}"), style))
                })
                .collect()
        })
        .collect();
    draw_cells_table(cells, "Sudoku thermodynamic grid", 9)
}

pub fn render_queens_tui(solution: &[u8; 8], mask: [bool; 8]) -> Result<(), Box<dyn Error>> {
    let cells: Vec<Vec<Cell>> = solution
        .iter()
        .enumerate()
        .map(|(row, &queen_col)| {
            (0..8)
                .map(|col| {
                    if col == queen_col as usize {
                        let style = if mask[row] {
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                        };
                        Cell::from(Span::styled(" Q ", style))
                    } else {
                        Cell::from(Span::styled(" . ", Style::default().fg(Color::DarkGray)))
                    }
                })
                .collect()
        })
        .collect();
    draw_cells_table(cells, "8-Queens placement", 8)
}

fn draw_cells_table(cells: Vec<Vec<Cell>>, title: &str, columns: usize) -> Result<(), Box<dyn Error>> {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let rows = cells.into_iter().map(Row::new).collect::<Vec<Row>>();
    let widths = vec![Constraint::Length(3); columns];
    let table = Table::new(rows, widths).block(Block::default().title(title).borders(Borders::ALL));
    terminal.draw(|frame| {
        frame.render_widget(table.clone(), frame.area());
    })?;
    terminal.show_cursor()?;
    Ok(())
}
