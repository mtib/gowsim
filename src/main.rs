use clap::Parser;
use std::{
    collections::HashMap,
    fs::{write, File},
    io::Read,
    time::Instant,
};

mod game;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    num: usize,
}

fn main() {
    let args = Args::parse();
    histogram_length_of_game(args.num);
}

type State = HashMap<usize, usize>;

fn load_state_from_disk() -> State {
    fn load() -> Option<State> {
        let mut file = File::open("./state.msgp").ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;
        rmp_serde::from_slice::<State>(buffer.as_slice()).ok()
    }
    load().unwrap_or(HashMap::new())
}

fn save_state_to_disk(state: State) {
    let mut csv_data = String::new();
    csv_data.push_str("length, count\n");
    let mut results: Vec<(usize, usize)> = state.clone().into_iter().map(|(k, v)| (k, v)).collect();
    results.sort_by_key(|(k, _)| *k);
    for (length, chance) in results {
        csv_data.push_str(&format!("{}, {}\n", length, chance));
    }
    write("./state.csv", csv_data).unwrap();
    let serialized_state = rmp_serde::to_vec(&state).unwrap();
    write("./state.msgp", serialized_state).unwrap();
}

fn histogram_length_of_game(num_games: usize) {
    let mut count_map = load_state_from_disk();
    let start = Instant::now();
    struct LastUpdateState {
        instant: Instant,
        count: usize,
    }
    let mut last_update = LastUpdateState {
        instant: Instant::now(),
        count: 0,
    };
    println!("Simulating {} games", num_games);
    for i in 0..num_games {
        if i % 100000 == 0 && last_update.instant.elapsed().as_secs() >= 5 {
            let throughput_per_sec =
                (i - last_update.count + 1) as f64 / last_update.instant.elapsed().as_secs_f64();
            println!(
                "Running for {:.1}s, simulating {:0.1} games per second ({:.1}% of run complete, {:.1}m remaining)",
                start.elapsed().as_secs_f64(),
                throughput_per_sec,
                i as f64 / num_games as f64 * 100f64,
                (num_games - i) as f64 / throughput_per_sec / 60f64,
            );
            last_update = LastUpdateState {
                instant: Instant::now(),
                count: i,
            }
        }
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
    println!("Done! Saving to disk.");
    save_state_to_disk(count_map);
}
