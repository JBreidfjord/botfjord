use chess::Board;
use pyo3::prelude::*;
use std::{str::FromStr, sync::Arc, time::Instant};

use crate::{eval::Evaluator, genetic::run_ga, helpers::uci, mcts::start_search};

mod eval;
mod genetic;
mod helpers;
mod mcts;

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
