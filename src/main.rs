use std::cmp::{min, max};
use std::io::stdin;
use std::env::args;

#[derive(Clone)]
struct GameState {
    history: Vec<(u64, Dir)>,
    upper_limit: u64,
}

// Half open
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    lower: u64,
    higher: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    High,
    Low,
}

use Dir::*;

impl Dir {
    fn opposite(self) -> Self {
        match self {
            High => Low,
            Low => High,
        }
    }
}

impl Range {
    fn new(lower: u64, higher: u64) -> Range {
        Range {
            lower: lower,
            higher: higher,
        }
    }
    fn clamp_lower(&self, clamp: u64) -> Range {
        Range {
            lower: max(self.lower, clamp),
            higher: self.higher,
        }
    }
    fn clamp_higher(&self, clamp: u64) -> Range {
        Range {
            lower: self.lower,
            higher: min(self.higher, clamp),
        }
    }
    fn len(&self) -> u64 {
        self.higher.saturating_sub(self.lower)
    }
}

impl GameState {
    fn new(upper_limit: u64) -> GameState {
        GameState {
            history: vec![],
            upper_limit: upper_limit,
        }
    }
    fn store_guess(&mut self, value: u64, response: Dir) -> Result<(), &str> {
        if value >= self.upper_limit {
            Err("Value too large")
        } else {
            self.history.push((value, response));
            Ok(())
        }
    }
    fn possibilities(&self) -> Vec<(Range, Option<usize>)> {
        let mut lies: Vec<Option<usize>> = (0..self.history.len()).map(|num| Some(num)).collect();
        lies.push(None);
        let lies = lies;
        lies.iter()
            .map(|&lie| {
                let mut range = Range::new(0, self.upper_limit);
                for (index, &(guess, response)) in self.history.iter().enumerate() {
                    let truth = if lie == Some(index) {
                        response.opposite()
                    } else {
                        response
                    };
                    match truth {
                        High => range = range.clamp_lower(guess),
                        Low => range = range.clamp_higher(guess),
                    }
                }
                (range, lie)
            })
            .collect()
    }
}

fn simple_value(game: &GameState) -> u64 {
    game.possibilities().iter().map(|&(range, _)| range.len()).sum()
}

fn better_value(game: &GameState) -> u64 {
    let multiplier = (simple_value(game) as f64).log2() - 1 as f64;
    game.possibilities().iter().map(|&(range, lie)| if lie.is_none() {
        range.len() as f64 * multiplier
    } else {
        range.len() as f64
    }).sum::<f64>() as u64
}        

fn adversarial_response(value: &Fn(&GameState) -> u64, game: &GameState, guess: u64) -> Dir {
    let mut game_high = game.clone();
    game_high.store_guess(guess, High).unwrap();
    let mut game_low = game.clone();
    game_low.store_guess(guess, Low).unwrap();
    let high_remaining: u64 = value(&game_high);
    let low_remaining: u64 = value(&game_low);
    if high_remaining > low_remaining {
        High
    } else {
        Low
    }
}

#[derive(PartialEq, Eq)]
enum GameResult {
    Ongoing,
    Finished(u64),
    Impossible,
}

use GameResult::*;

fn result(poss: Vec<(Range, Option<usize>)>) -> GameResult {
    let ranges: Vec<Range> = poss.iter().map(|&(range, _)| range)
        .filter(|range| range.len() > 0)
        .collect();
    if ranges.iter().any(|range| range.len() > 1) {
        Ongoing
    } else {
        if let Some(first) = ranges.first() {
            if ranges.iter().all(|range| range == first) {
                Finished(first.lower)
            } else {
                Ongoing
            }
        } else {
            Impossible
        }
    }
}

fn play_game(upper_limit: u64, opponent: &Fn(&GameState, u64) -> Dir) {
    let mut game = GameState::new(upper_limit);
    println!(
        "Guess the number, with up to one lie, out of {}",
        upper_limit
    );
    while result(game.possibilities()) == Ongoing {
        println!(
            "{}: What number do you want to know if it's less than?",
            game.history.len()
        );
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read stdin");
        match input.trim().parse::<u64>() {
            Err(_) => println!("Input could not be parsed as a number in range"),
            Ok(guess) => {
                if guess >= upper_limit {
                    println!("Guesses must be less than {}", upper_limit);
                } else {
                    let response = opponent(&game, guess);
                    if response == High {
                        println!("Greater than or equal to {}", guess);
                    } else {
                        println!("Less than {}\n", guess);
                    }
                    game.store_guess(guess, response).expect("Already checked guess was legal");
                }
            }
        }
    }
    if let Finished(answer) = result(game.possibilities()) {
        println!("You got it in {} guesses", game.history.len());
        println!("It was {}", answer);
        let poss_lies: Vec<Option<usize>> = game.possibilities().iter()
            .filter(|&&(range, _)| range.len() > 0)
            .map(|&(_, lie)| lie)
            .collect();
        println!(
            "The opponent could have lied on question(s) {:?}",
            poss_lies
        );
    }
}

fn main() {
    let upper_limit = args().nth(1).map_or(10, |arg| arg.parse().unwrap());
    play_game(upper_limit, &|game, guess| adversarial_response(&better_value, game, guess));
}
