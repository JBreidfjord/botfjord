mod eval;
mod genetic;
mod helpers;
mod mcts;

use crate::genetic::run_ga;

fn main() {
    run_ga(10, 0.4, 0.5, 100, 500);
}
