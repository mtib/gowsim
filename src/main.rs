use std::collections::HashMap;

mod game;

fn main() {
    histogram_length_of_game(1000000);
}

fn histogram_length_of_game(num_games: usize) {
    let mut count_map = HashMap::new();
    for _ in 0..num_games {
        let mut game1 = game::Game::new();
        loop {
            let step = game1.step();
            if step.is_none() {
                break;
            }
        }
        if let Some(before) = count_map.get(&game1.stats.turn_number) {
            count_map.insert(game1.stats.turn_number, *before + 1);
        } else {
            count_map.insert(game1.stats.turn_number, 1);
        }
    }
    let mut results: Vec<(usize, usize)> =
        count_map.clone().into_iter().map(|(k, v)| (k, v)).collect();
    results.sort_by_key(|(k, _)| *k);
    println!("length, count");
    for (length, chance) in results {
        println!("{}, {}", length, chance);
    }
}
