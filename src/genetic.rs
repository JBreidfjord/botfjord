use chess::{Color, Game, GameResult, ALL_COLORS, ALL_PIECES, ALL_SQUARES};
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
    Rng,
};
use std::{collections::HashMap, fs::write, sync::Arc};

use crate::{eval::Evaluator, mcts::start_search};

fn generate_initial_population(population_size: usize) -> Vec<Arc<Evaluator>> {
    let mut rng = rand::thread_rng();
    let dist: Uniform<isize> = Uniform::new_inclusive(-100, 100);

    let mut population = vec![];
    for _ in 0..population_size {
        let mut evaluator = Evaluator::empty();

        for color in ALL_COLORS {
            let mut early_color_map = HashMap::new();
            let mut end_color_map = HashMap::new();

            for piece in ALL_PIECES {
                let mut early_piece_map = HashMap::new();
                let mut end_piece_map = HashMap::new();

                for square in ALL_SQUARES {
                    early_piece_map.insert(square, dist.sample(&mut rng));
                    end_piece_map.insert(square, dist.sample(&mut rng));
                }
                early_color_map.insert(piece, early_piece_map);
                end_color_map.insert(piece, end_piece_map);
            }
            evaluator.early_maps.insert(color, early_color_map);
            evaluator.end_maps.insert(color, end_color_map);
        }
        population.push(Arc::new(evaluator));
    }

    population
}

fn boundary_mutation(individual: &Evaluator, mutation_rate: f32, n_mutations: usize) -> Evaluator {
    let mut rng = rand::thread_rng();
    let dist = Uniform::new_inclusive(0.0, 1.0);
    let mut mutated_child = individual.clone();
    for _ in 0..n_mutations {
        if dist.sample(&mut rng) <= mutation_rate {
            if dist.sample(&mut rng) >= 0.5 {
                *mutated_child
                    .early_maps
                    .get_mut(ALL_COLORS.choose(&mut rng).unwrap())
                    .unwrap()
                    .get_mut(ALL_PIECES.choose(&mut rng).unwrap())
                    .unwrap()
                    .get_mut(ALL_SQUARES.choose(&mut rng).unwrap())
                    .unwrap() = if dist.sample(&mut rng) >= 0.5 {
                    100
                } else {
                    -100
                }
            } else {
                *mutated_child
                    .end_maps
                    .get_mut(ALL_COLORS.choose(&mut rng).unwrap())
                    .unwrap()
                    .get_mut(ALL_PIECES.choose(&mut rng).unwrap())
                    .unwrap()
                    .get_mut(ALL_SQUARES.choose(&mut rng).unwrap())
                    .unwrap() = if dist.sample(&mut rng) >= 0.5 {
                    100
                } else {
                    -100
                }
            }
        }
    }
    mutated_child
}

fn averaged_crossover(parent_a: &Evaluator, parent_b: &Evaluator) -> Vec<Arc<Evaluator>> {
    let mut rng = rand::thread_rng();
    let dist = Uniform::new_inclusive(0.0, 1.0);

    let mut child_a = Evaluator::new();
    let mut child_b = Evaluator::new();

    for color in ALL_COLORS {
        let parent_a_early_color_map = &parent_a.early_maps[&color];
        let parent_a_end_color_map = &parent_a.end_maps[&color];
        let parent_b_early_color_map = &parent_b.early_maps[&color];
        let parent_b_end_color_map = &parent_b.end_maps[&color];

        let mut child_a_early_color_map = HashMap::new();
        let mut child_a_end_color_map = HashMap::new();
        let mut child_b_early_color_map = HashMap::new();
        let mut child_b_end_color_map = HashMap::new();

        for piece in ALL_PIECES {
            let parent_a_early_piece_map = &parent_a_early_color_map[&piece];
            let parent_a_end_piece_map = &parent_a_end_color_map[&piece];
            let parent_b_early_piece_map = &parent_b_early_color_map[&piece];
            let parent_b_end_piece_map = &parent_b_end_color_map[&piece];

            let mut child_a_early_piece_map = HashMap::new();
            let mut child_a_end_piece_map = HashMap::new();
            let mut child_b_early_piece_map = HashMap::new();
            let mut child_b_end_piece_map = HashMap::new();

            for square in ALL_SQUARES {
                let parent_a_early_square_value = parent_a_early_piece_map[&square];
                let parent_a_end_square_value = parent_a_end_piece_map[&square];
                let parent_b_early_square_value = parent_b_early_piece_map[&square];
                let parent_b_end_square_value = parent_b_end_piece_map[&square];

                let early_weight_factor = dist.sample(&mut rng);
                let end_weight_factor = dist.sample(&mut rng);
                let early_a_value = parent_a_early_square_value as f32 * early_weight_factor
                    + parent_b_early_square_value as f32 * (1.0 - early_weight_factor);
                let end_a_value = parent_a_end_square_value as f32 * end_weight_factor
                    + parent_b_end_square_value as f32 * (1.0 - end_weight_factor);
                let early_b_value = parent_b_early_square_value as f32 * early_weight_factor
                    + parent_a_early_square_value as f32 * (1.0 - early_weight_factor);
                let end_b_value = parent_b_early_square_value as f32 * early_weight_factor
                    + parent_a_early_square_value as f32 * (1.0 - early_weight_factor);

                child_a_early_piece_map.insert(square, early_a_value as isize);
                child_a_end_piece_map.insert(square, end_a_value as isize);
                child_b_early_piece_map.insert(square, early_b_value as isize);
                child_b_end_piece_map.insert(square, end_b_value as isize);
            }
            child_a_early_color_map.insert(piece, child_a_early_piece_map);
            child_a_end_color_map.insert(piece, child_a_end_piece_map);
            child_b_early_color_map.insert(piece, child_b_early_piece_map);
            child_b_end_color_map.insert(piece, child_b_end_piece_map);
        }
        child_a.early_maps.insert(color, child_a_early_color_map);
        child_a.end_maps.insert(color, child_a_end_color_map);
        child_b.early_maps.insert(color, child_b_early_color_map);
        child_b.end_maps.insert(color, child_b_end_color_map);
    }

    vec![Arc::new(child_a), Arc::new(child_b)]
}

fn generate_new_population(
    current_population: Arc<Vec<Arc<Evaluator>>>,
    survival_rate: f32,
    mutation_rate: f32,
    n_mutations: usize,
) -> Vec<Arc<Evaluator>> {
    let population_size = current_population.len();
    let fitness = population_fitness(&current_population);
    let mut pop_and_fit: Vec<(Arc<Evaluator>, usize)> = current_population
        .to_vec()
        .into_iter()
        .zip(fitness)
        .collect();
    pop_and_fit.sort_by_key(|x| x.1);
    pop_and_fit.reverse();

    let number_of_children =
        (population_size as f32 - (population_size as f32 * survival_rate)) as usize;
    let reproducers: Vec<_> = pop_and_fit.iter().take(number_of_children).collect();
    let mut group_a = vec![];
    let mut group_b = vec![];
    for (i, (reproducer, _)) in reproducers.iter().enumerate() {
        if i < number_of_children / 2 {
            group_a.push(reproducer);
        } else {
            group_b.push(reproducer);
        }
    }

    let mut children = vec![];
    for (a, b) in group_a.iter().zip(&group_b) {
        for child in averaged_crossover(a, b) {
            children.push(Arc::new(boundary_mutation(
                &child,
                mutation_rate,
                n_mutations,
            )));
        }
    }

    for (survivor, _) in pop_and_fit
        .into_iter()
        .take(population_size - number_of_children)
    {
        children.push(survivor);
    }
    assert_eq!(children.len(), population_size);

    children
}

fn population_fitness(population: &Vec<Arc<Evaluator>>) -> Vec<usize> {
    let mut fitness = vec![0; population.len()];
    for (i, individual) in population.iter().enumerate() {
        for (j, competitor) in population.iter().enumerate() {
            if i == j {
                continue;
            } else {
                let result = simulate_game(Arc::clone(individual), Arc::clone(competitor));
                fitness[i] += result;
                if result == 1 {
                    fitness[j] += result;
                }
            }
        }
    }

    println!("{:?}", fitness);
    fitness
}

fn simulate_game(individual: Arc<Evaluator>, competitor: Arc<Evaluator>) -> usize {
    let mut rng = rand::thread_rng();
    let flip = rng.gen::<bool>();

    let mut game = Game::new();
    let mut players = HashMap::new();

    if flip {
        players.insert(Color::White, individual);
        players.insert(Color::Black, competitor);
    } else {
        players.insert(Color::White, competitor);
        players.insert(Color::Black, individual);
    }

    while game.result().is_none() {
        let action = start_search(
            game.current_position(),
            Arc::clone(&players[&game.side_to_move()]),
            0.05,
            10.0,
            32,
        )
        .iter()
        .max_by_key(|x| x.1)
        .unwrap()
        .0;

        game.make_move(action);

        if game.can_declare_draw() {
            game.declare_draw();
        } else if game.actions().len() > 100 {
            break;
        }
    }

    println!(
        "{:?} ({})",
        game.result().unwrap_or(GameResult::Stalemate),
        game.actions().len()
    );
    match game.result() {
        Some(GameResult::WhiteCheckmates) => {
            if flip {
                2
            } else {
                0
            }
        }
        Some(GameResult::BlackCheckmates) => {
            if flip {
                0
            } else {
                2
            }
        }
        Some(GameResult::Stalemate) | Some(GameResult::DrawDeclared) => 1,
        _ => 0,
    }
}

pub fn run_ga(
    population_size: usize,
    survival_rate: f32,
    mutation_rate: f32,
    n_mutations: usize,
    n_generations: usize,
) -> Vec<Arc<Evaluator>> {
    let mut population = generate_initial_population(population_size);
    for _ in 0..n_generations {
        population = generate_new_population(
            Arc::new(population),
            survival_rate,
            mutation_rate,
            n_mutations,
        );
    }

    write("genetic_evaluator", format!("{:?}", population.to_vec())).expect("Failed to write");
    population.to_vec()
}
