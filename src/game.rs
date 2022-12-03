use std::mem::swap;

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Number(u8),
    Jack,
    Queen,
    King,
    Ace,
}

impl Face {
    pub fn measure_strength(&self) -> usize {
        match *self {
            Face::Number(a) => a as usize,
            Face::Jack => 11,
            Face::Queen => 12,
            Face::King => 13,
            Face::Ace => 14,
        }
    }

    pub fn war_length(&self) -> usize {
        match *self {
            Face::Number(a) => a as usize,
            Face::Ace => 1,
            Face::Jack => 12,
            Face::Queen => 13,
            Face::King => 14,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    suit: Suit,
    face: Face,
}

impl Card {
    pub fn new(suit: Suit, face: Face) -> Self {
        Card { suit, face }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub draw_pile: Vec<Card>,
    pub winnings_pile: Vec<Card>,
}

impl Player {
    pub fn is_dead(&self) -> bool {
        self.count_cards() == 0
    }
    pub fn is_winner(&self) -> bool {
        self.count_cards() == 52
    }
    pub fn count_cards(&self) -> usize {
        self.draw_pile.len() + self.winnings_pile.len()
    }
    pub fn measure_strength(&self) -> usize {
        self.draw_pile
            .iter()
            .chain(self.winnings_pile.iter())
            .map(|card| card.face.measure_strength())
            .sum()
    }
    pub fn draw(&mut self) -> Option<Card> {
        if self.draw_pile.len() == 0 && self.winnings_pile.len() != 0 {
            swap(&mut self.draw_pile, &mut self.winnings_pile);
        }
        self.draw_pile.pop()
    }
}

pub fn create_standard_deck() -> Vec<Card> {
    let mut deck: Vec<Card> = Vec::with_capacity(52);

    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        for face in [
            Face::Ace,
            Face::Number(2),
            Face::Number(3),
            Face::Number(4),
            Face::Number(5),
            Face::Number(6),
            Face::Number(7),
            Face::Number(8),
            Face::Number(9),
            Face::Number(10),
            Face::Jack,
            Face::Queen,
            Face::King,
        ] {
            deck.push(Card::new(suit, face));
        }
    }

    deck
}

pub fn create_shuffled_deck(rng: &mut ThreadRng) -> Vec<Card> {
    let mut deck = create_standard_deck();
    deck.shuffle(rng);
    deck
}

#[derive(Debug, Clone)]
pub enum Event {
    GameOver {
        winning_player_id: usize,
    },
    ShortBattle {
        winning_player_id: usize,
        winning_card: Card,
        losing_card: Card,
        pot: Vec<Card>,
    },
    WarStart {
        top_cards: (Card, Card),
        expected_length: usize,
    },
    WarShortened {
        player_id_with_insufficient_cards: usize,
        length_of_war_after_shortening: usize,
        initial_length_of_war: usize,
    },
    WarEnd {
        winning_player_id: usize,
        final_top_cards: (Card, Card),
    },
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub turn_number: usize,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub players: (Player, Player),
    pub stats: Stats,
    rng: ThreadRng,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let deck = create_shuffled_deck(&mut rng);
        let mut player0 = Player {
            draw_pile: Vec::new(),
            winnings_pile: Vec::new(),
        };
        let mut player1 = Player {
            draw_pile: Vec::new(),
            winnings_pile: Vec::new(),
        };
        for (num, card) in deck.into_iter().enumerate() {
            match (num, card) {
                (a, b) if a % 2 == 0 => player0.draw_pile.push(b),
                (_, b) => player1.draw_pile.push(b),
            }
        }
        Game {
            players: (player0, player1),
            stats: Stats { turn_number: 0 },
            rng,
        }
    }
    pub fn short_print(&self) -> String {
        format!(
            "Game{{ round {} [{}:{} cards, {} total, valued {}], [{}:{} cards, {} total, valued {}] }}",
            self.stats.turn_number,
            self.players.0.draw_pile.len(),
            self.players.0.winnings_pile.len(),
            self.players.0.count_cards(),
            self.players.0.measure_strength(),
            self.players.1.draw_pile.len(),
            self.players.1.winnings_pile.len(),
            self.players.1.count_cards(),
            self.players.1.measure_strength(),
        )
    }
    pub fn step(&mut self) -> Option<Vec<Event>> {
        if self.players.0.is_dead() || self.players.1.is_dead() {
            // Game is over, nothing is going to happen (win event is emitted after the last turn)
            return None;
        }
        self.stats.turn_number += 1;

        let mut events = Vec::new();

        match (self.players.0.draw(), self.players.1.draw()) {
            (Some(a), Some(b)) if a.face != b.face => {
                let mut pot = vec![a.clone(), b.clone()];
                pot.shuffle(&mut self.rng);
                match a.face.measure_strength().cmp(&b.face.measure_strength()) {
                    std::cmp::Ordering::Less => {
                        self.players.1.winnings_pile.extend(pot.clone());
                        events.push(Event::ShortBattle {
                            winning_player_id: 1,
                            winning_card: b,
                            losing_card: a,
                            pot,
                        });
                    }
                    std::cmp::Ordering::Greater => {
                        self.players.0.winnings_pile.extend(pot.clone());
                        events.push(Event::ShortBattle {
                            winning_player_id: 0,
                            winning_card: a,
                            losing_card: b,
                            pot,
                        });
                    }
                    std::cmp::Ordering::Equal => {
                        unreachable!("Covered by war match branch")
                    }
                }
            }
            (_, None) | (None, _) => {
                // Will die in the checks after this match, likely unreachable
            }
            (Some(a), Some(b)) => {
                let mut pot = (vec![a.clone()], vec![b.clone()]);
                let mut war_events = Vec::new();
                resolve_war(self, &mut pot, &mut war_events);
                events.extend(war_events);
            }
        }

        if self.players.0.is_dead() {
            events.push(Event::GameOver {
                winning_player_id: 1,
            })
        }
        if self.players.1.is_dead() {
            events.push(Event::GameOver {
                winning_player_id: 0,
            })
        }

        Some(events)
    }
}

fn resolve_war(game: &mut Game, pot: &mut (Vec<Card>, Vec<Card>), events: &mut Vec<Event>) {
    let top_at_start = (pot.0.last().unwrap().clone(), pot.1.last().unwrap().clone());
    if top_at_start.0.face != top_at_start.1.face {
        panic!("Cards cannot start a war");
    }
    let expected_length = top_at_start.0.face.war_length();
    events.push(Event::WarStart {
        top_cards: top_at_start,
        expected_length,
    });

    for i in 1..=expected_length {
        if game.players.0.count_cards() != 0 && game.players.1.count_cards() != 0 {
            if let (Some(a), Some(b)) = (game.players.0.draw(), game.players.1.draw()) {
                pot.0.push(a);
                pot.1.push(b);
            } else {
                unreachable!("Checked players have at least one card before drawing. Drawing a card now should never fail");
            }
        } else {
            events.push(Event::WarShortened {
                player_id_with_insufficient_cards: if game.players.0.count_cards() == 0 {
                    0
                } else {
                    1
                },
                length_of_war_after_shortening: i - 1,
                initial_length_of_war: expected_length,
            });
            break;
        }
    }

    let top_at_end = (pot.0.last().unwrap().clone(), pot.1.last().unwrap().clone());
    let pot_cards = {
        if game.rng.gen_bool(0.5) {
            pot.0.iter().chain(pot.1.iter())
        } else {
            pot.1.iter().chain(pot.0.iter())
        }
    }
    .map(|c| c.clone());

    match top_at_end
        .0
        .face
        .measure_strength()
        .cmp(&top_at_end.1.face.measure_strength())
    {
        std::cmp::Ordering::Less => {
            game.players.1.winnings_pile.extend(pot_cards.clone());
            events.push(Event::WarEnd {
                winning_player_id: 1,
                final_top_cards: top_at_end,
            })
        }
        std::cmp::Ordering::Greater => {
            game.players.0.winnings_pile.extend(pot_cards.clone());
            events.push(Event::WarEnd {
                winning_player_id: 0,
                final_top_cards: top_at_end,
            })
        }
        std::cmp::Ordering::Equal => {
            if !game.players.0.is_dead() && !game.players.1.is_dead() {
                return resolve_war(game, pot, events);
            }
            if game.players.0.is_dead() {
                game.players.1.winnings_pile.extend(pot_cards.clone());
                events.push(Event::WarEnd {
                    winning_player_id: 1,
                    final_top_cards: top_at_end,
                })
            } else {
                game.players.0.winnings_pile.extend(pot_cards.clone());
                events.push(Event::WarEnd {
                    winning_player_id: 0,
                    final_top_cards: top_at_end,
                })
            }
        }
    }
}
