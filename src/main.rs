#![feature(map_try_insert)]
use crate::{
    eval::Evaluator,
    mcts::{Limit, Tree},
};
use chess::{ChessMove, Game};
use ordered_float::OrderedFloat;
use std::{env, str::FromStr, time::Instant};

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

fn main() {
    let args: Vec<String> = env::args().collect();
    let fen = format!(
        "{} {} {} {} {} {}",
        &args[1], &args[2], &args[3], &args[4], &args[5], args[6]
    );

    let start = Instant::now();

    let game = Game::from_str(&fen).unwrap();
    println!("{}", game.current_position());

    let evaluator = Evaluator::new();
    let mut tree = Tree::new(evaluator, 1.41, 0.3);
    let limit = Limit::new(Some(30.0), Some(100_000.0));

    let mut results = tree.search(game.current_position(), Some(limit));
    results.sort_by_key(|x| OrderedFloat(x.1));
    results.reverse();
    let mut fmt_results = vec![];
    for (action, value) in results.iter().take(5) {
        fmt_results.push((uci(action), value));
    }
    println!("{:?}", fmt_results);
    println!("{}", start.elapsed().as_secs_f32());
}
