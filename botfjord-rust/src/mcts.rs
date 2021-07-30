use chess::{Board, ChessMove, MoveGen};
use ordered_float::OrderedFloat;
use rand::{prelude::*, thread_rng};
use rand_distr::Dirichlet;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Formatter, Result},
    option::Option,
    rc::Rc,
    time::Instant,
};

use crate::eval::Evaluator;

struct Branch {
    prior: f32,
    visit_count: f32,
    total_value: f32,
}

pub struct Limit {
    time: f32,
    nodes: f32,
}

struct Node {
    state: Board,
    value: f32,
    priors: HashMap<ChessMove, f32>,
    parent: Option<Rc<RefCell<Node>>>,
    last_move: Option<Rc<ChessMove>>,
    total_visit_count: f32,
    branches: HashMap<ChessMove, Branch>,
    children: HashMap<Rc<ChessMove>, Rc<RefCell<Node>>>,
}

pub struct Tree {
    evaluator: Evaluator,
    c: f32,
    noise: f32,
    rng: ThreadRng,
}

impl Branch {
    fn new(prior: f32) -> Branch {
        Branch {
            prior,
            visit_count: 0.0,
            total_value: 0.0,
        }
    }
}

impl Limit {
    pub fn new(time: Option<f32>, nodes: Option<f32>) -> Limit {
        if time.is_none() && nodes.is_none() {
            return Limit {
                time: 0.0,
                nodes: 0.0,
            };
        }
        Limit {
            time: time.unwrap_or(0.0),
            nodes: nodes.unwrap_or(0.0),
        }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Node")
            .field("state", &self.state)
            .field("value", &self.value)
            .field("visits", &self.total_visit_count)
            .field("last_move", &self.last_move)
            .field("parent", &self.parent)
            .finish()
    }
}

impl Node {
    fn new(
        state: Board,
        value: f32,
        priors: HashMap<ChessMove, f32>,
        parent: Option<Rc<RefCell<Node>>>,
        last_move: Option<Rc<ChessMove>>,
    ) -> Node {
        let mut children = HashMap::new();
        let mut branches = HashMap::new();
        for action in MoveGen::new_legal(&state) {
            // Unwrap is not recommended but we don't want an error to pass silently
            let prior = priors.get(&action).unwrap();
            branches.insert(action, Branch::new(*prior));
        }
        Node {
            state,
            value,
            priors,
            parent,
            last_move,
            total_visit_count: 1.0,
            branches,
            children,
        }
    }

    fn moves(&self) -> Vec<&ChessMove> {
        self.branches.keys().collect()
    }

    fn add_child(&mut self, action: Rc<ChessMove>, child_node: Rc<RefCell<Node>>) {
        // Add error handling for existing keys
        // Currently will silently overwrite value but it should not be allowed
        self.children.insert(action, child_node);
    }

    fn has_child(&self, action: &ChessMove) -> bool {
        self.children.contains_key(action)
    }

    fn get_child(&self, action: &ChessMove) -> &Rc<RefCell<Node>> {
        self.children.get(action).unwrap()
    }

    fn expected_value(&self, action: &ChessMove) -> f32 {
        let branch = self.branches.get(action).unwrap();
        if branch.visit_count == 0.0 {
            return 0.0;
        }
        branch.total_value / branch.visit_count
    }

    fn prior(&self, action: &ChessMove) -> f32 {
        self.branches.get(action).unwrap().prior
    }

    fn visit_count(&self, action: &ChessMove) -> f32 {
        match self.branches.get(action) {
            Some(b) => b.visit_count,
            None => 0.0,
        }
    }

    fn record_visit(&mut self, action: &ChessMove, value: f32) {
        let branch = self.branches.get_mut(action).unwrap();
        branch.visit_count += 1.0;
        branch.total_value += value;
        self.total_visit_count += 1.0;
    }

    fn check_visit_counts(&self, rounds: f32) -> bool {
        let mut branches: Vec<_> = self.branches.values().collect();
        branches.sort_by(|a, b| OrderedFloat(b.visit_count).cmp(&OrderedFloat(a.visit_count)));
        let remaining_rounds = rounds - self.total_visit_count;
        branches[0].visit_count >= branches[1].visit_count + remaining_rounds
    }

    fn check_visit_ratio(&self, factor: f32, minimum: f32) -> bool {
        if self.total_visit_count < minimum {
            return false;
        }
        let mut branches: Vec<_> = self.branches.values().collect();
        let branch = branches
            .iter()
            .max_by_key(|b| OrderedFloat(b.visit_count))
            .unwrap();
        branch.visit_count > self.total_visit_count * factor
    }
}

impl Tree {
    fn new(evaluator: Evaluator, temperature: f32, noise: f32) -> Tree {
        Tree {
            evaluator,
            c: temperature,
            noise,
            rng: thread_rng(),
        }
    }

    fn create_node(
        &mut self,
        state: Board,
        action: Option<Rc<ChessMove>>,
        parent: Option<Rc<RefCell<Node>>>,
    ) -> Node {
        let mut priors = self.evaluator.prior(state);
        let value = self.evaluator.evaluate(state);

        // Add Dirichlet noise
        if self.noise != 0.0 {
            let dirichlet =
                Dirichlet::new_with_size(self.noise, MoveGen::new_legal(&state).len()).unwrap();
            let samples = dirichlet.sample(&mut self.rng);
            let mut new_priors: HashMap<ChessMove, f32> = HashMap::new();
            for ((action, value), noise) in priors.iter().zip(samples) {
                new_priors.insert(*action, (value * 0.5) + (noise * 0.5));
            }
            priors = new_priors;
        }

        Node::new(state, value, priors, parent, action)
    }

    fn select_branch(&self, node: &Node) -> ChessMove {
        let total_n = node.total_visit_count;

        let score_branch = |action: &ChessMove| {
            let q = node.expected_value(action);
            let p = node.prior(action);
            let n = node.visit_count(action);
            q + self.c * p * (total_n.ln() / (n + 0.0000001)).sqrt()
        };

        **node
            .moves()
            .iter()
            .max_by_key(|m| OrderedFloat(score_branch(m)))
            .unwrap()
    }

    fn search(&mut self, state: Board, limit: Option<Limit>) -> Vec<(ChessMove, f32)> {
        let limit = limit.unwrap_or(Limit::new(None, None));
        let mut i = 0.0;
        let start_time = Instant::now();
        let root = Rc::new(RefCell::new(self.create_node(state, None, None)));
        loop {
            let mut node = Rc::clone(&root);
            let mut next_move = Rc::new(self.select_branch(&node.borrow()));

            while node.borrow().has_child(&next_move) {
                let new_node = Rc::clone(node.borrow().get_child(&next_move));
                node = new_node;
                next_move = Rc::new(self.select_branch(&node.borrow()));
            }

            let new_state = node.borrow().state.make_move_new(*next_move);
            let child_node = Rc::new(RefCell::new(self.create_node(
                new_state,
                Some(Rc::clone(&next_move)),
                Some(Rc::clone(&node)),
            )));
            node.borrow_mut()
                .add_child(Rc::clone(&next_move), Rc::clone(&child_node));

            let mut action = Rc::clone(&next_move);
            let mut value = -child_node.borrow().value;
            loop {
                node.borrow_mut().record_visit(&action, value);
                action = Rc::clone(match node.borrow().last_move.as_ref() {
                    Some(m) => m,
                    None => break,
                });
                let new_node = Rc::clone(match node.borrow().parent.as_ref() {
                    Some(n) => n,
                    None => break,
                });
                node = new_node;
                value = -value;
            }

            if root.borrow().check_visit_ratio(0.5, 1000.0) {
                break;
            }

            if limit.nodes > 0.0 {
                if i >= limit.nodes || root.borrow().check_visit_counts(limit.nodes) {
                    break;
                } else {
                    i += 1.0;
                }
            }
            if limit.time > 0.0 {
                if start_time.elapsed().as_secs_f32() >= limit.time {
                    break;
                }
            }
        }

        let mut results = vec![];
        for action in root.borrow().moves() {
            results.push((*action, root.borrow().visit_count(action)));
        }
        results
    }
}
