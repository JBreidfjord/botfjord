#![allow(unused_imports)]
use crate::{
    eval::Evaluator,
    mcts::{Limit, Tree},
};
use chess::{ChessMove, Game};
use ordered_float::OrderedFloat;
use pyo3::prelude::*;
use std::str::FromStr;

mod eval;
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
fn search_tree(fen: String) -> String {
    let game = Game::from_str(&fen).unwrap();

    let evaluator = Evaluator::new();
    let mut tree = Tree::new(evaluator, 1.41, 0.3);
    let limit = Limit::new(Some(30.0), Some(250_000.0));

    let mut results = tree.search(game.current_position(), Some(limit));
    results.sort_by_key(|x| OrderedFloat(x.1));
    let result = results.last().unwrap();
    uci(&result.0)
}

#[pymodule]
#[allow(unused_variables)]
fn mcts_rust(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search_tree, m)?)?;
    Ok(())
}
