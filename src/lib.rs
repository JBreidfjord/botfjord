use chess::{Board, ChessMove};
use pyo3::prelude::*;
use std::{str::FromStr, sync::Arc, time::Instant};

use crate::{eval::Evaluator, genetic::run_ga, mcts::start_search};

mod eval;
mod genetic;
mod mcts;

fn uci(action: &ChessMove) -> String {
    let squares = vec![
        "A1", "B1", "C1", "D1", "E1", "F1", "G1", "H1", "A2", "B2", "C2", "D2", "E2", "F2", "G2",
        "H2", "A3", "B3", "C3", "D3", "E3", "F3", "G3", "H3", "A4", "B4", "C4", "D4", "E4", "F4",
        "G4", "H4", "A5", "B5", "C5", "D5", "E5", "F5", "G5", "H5", "A6", "B6", "C6", "D6", "E6",
        "F6", "G6", "H6", "A7", "B7", "C7", "D7", "E7", "F7", "G7", "H7", "A8", "B8", "C8", "D8",
        "E8", "F8", "G8", "H8",
    ];
    let src = action.get_source().to_index();
    let dst = action.get_dest().to_index();
    format!(
        "{}{}",
        squares[src].to_lowercase(),
        squares[dst].to_lowercase()
    )
}

#[pyfunction]
fn search_tree(fen: String, time: f32, temperature: f32, processes: usize) -> String {
    let start = Instant::now();

    let board = Board::from_str(&fen).unwrap();
    let evaluator = Evaluator::new();

    let mut results = start_search(board, Arc::new(evaluator), time, temperature, processes);

    results.sort_by_key(|x| x.1);
    results.reverse();
    let mut fmt_results = vec![];
    let mut nodes = 0;
    for (i, (action, value)) in results.iter().enumerate() {
        nodes += value;
        if i < 5 {
            fmt_results.push(format!("{} {:.0}", uci(action), value));
        }
    }
    let run_time = start.elapsed().as_secs_f32();
    println!(
        "{} | {:.0} nodes/s ({:.2}s | {:.0} nodes)",
        fmt_results.join(" | "),
        nodes as f32 / run_time,
        run_time,
        nodes
    );

    uci(&results[0].0)
}

#[pyfunction]
fn get_genetic_evaluators(
    population_size: usize,
    survival_rate: f32,
    mutation_rate: f32,
    n_mutations: usize,
    n_generations: usize,
) {
    run_ga(
        population_size,
        survival_rate,
        mutation_rate,
        n_mutations,
        n_generations,
    );
}

#[pymodule]
#[allow(unused_variables)]
fn mcts_rust(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search_tree, m)?)?;
    m.add_function(wrap_pyfunction!(get_genetic_evaluators, m)?)?;
    Ok(())
}
