// Copyright (C) 2019 Arc676/Alessandro Vinciguerra <alesvinciguerra@gmail.com>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation (version 3).

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::io;
use std::io::Write;

extern crate blackjack;
use blackjack::card::card::*;
use blackjack::player::player::*;

fn get_int(prompt: &str) -> i32 {
        let mut input = String::new();
        print!("{}", prompt);
        io::stdout().flush().expect("Failed to flush");
        match io::stdin().read_line(&mut input) {
                Ok(_) => {
                        let val: i32 = match input.trim().parse() {
                                Ok(val) => val,
                                Err(_) => {
                                        println!("Expected integer input");
                                        get_int(prompt)
                                }
                        };
                        val
                },
                Err(_) => {
                        println!("Failed to read");
                        get_int(prompt)
                }
        }
}

fn print_player_hand(player: &Player) {
	for (ih, hand) in player.hand_iter().enumerate() {
		println!("{}'s hand #{}: {} points", player.get_name(), ih + 1, hand.value(false));
		for (ic, card) in hand.card_iter().enumerate() {
			print!("{}{}", match ic { 0 => "", _ => ", " }, card.to_string());
		}
		println!("");
	}
}

fn main() {
	println!("Blackjack!");

	let deck_count = get_int("How many decks? ") as usize;
	let mut deck = Deck::new(deck_count);
	deck.shuffle();

	let player_count = get_int("How many players? ") as usize;
	let mut players: Vec<Player> = Vec::with_capacity(player_count);
	for _ in 0..player_count {
		let mut name = String::new();
		print!("Enter your name: ");
		io::stdout().flush().expect("Failed to flush");
		io::stdin().read_line(&mut name).expect("Failed to read");
		name.truncate(name.len() - 1);
		let initial_standing = get_int("Enter player's initial standing: ");
		let player = Player::new(name, false, initial_standing);
		players.push(player);
	}

	let mut dealer = Player::new(String::from("Dealer"), true, -1);

	loop {
		for player in players.iter_mut() {
			let bet = get_int(&format!("{}: Enter wager for this hand: ", player.get_name()));
			player.bet(bet, &mut deck);
		}
		dealer.bet(0, &mut deck);

		for player in players.iter_mut() {
			print_player_hand(player);
			loop {
				let mut input = String::new();
				print!("> ");
				io::stdout().flush().expect("Failed to flush");
				match io::stdin().read_line(&mut input) {
					Ok(_) => match input.trim() {
						"hit" => {
							if player.hit(&mut deck) {
								break;
							}
							print_player_hand(player);
						},
						"stand" => break,
						"surrender" => {
							player.surrender();
							break;
						},
						"split" => {
							if player.split(&mut deck) {
								print_player_hand(player);
							} else {
								println!("Can't split this hand");
							}
						},
						"double" => {
							player.double(&mut deck);
							break;
						},
						"help" => println!("Commands: hit, stand, surrender, split, double"),
						_ => println!("Unknown command. Type 'help' for a list of available choices.")
					},
					Err(_) => println!("Failed to read")
				}
			}
			print_player_hand(player);
		}
		let mut dealer_plays = false;
		for player in players.iter() {
			if !player.has_surrendered() && !player.has_busted() {
				dealer_plays = true;
				break;
			}
		}
		let dealer_value = match dealer_plays {
			true => {
				println!("Dealer's turn");
				dealer.play_as_dealer(&mut deck);
				print_player_hand(&dealer);
				match dealer.has_busted() {
					true => 0,
					false => dealer.first_hand_value()
				}
			},
			false => 0
		};
		for player in players.iter_mut() {
			player.game_over(dealer_value);
			println!("{}'s balance, standing: {}/{}", player.get_name(), player.get_balance(), player.get_standing());
		}
		dealer.game_over(0);
		let mut input = String::new();
		print!("Play again? [Y/n]: ");
		io::stdout().flush().expect("Failed to flush");
		match io::stdin().read_line(&mut input) {
			Ok(_) => match input.trim() {
				"n" | "N" => break,
				_ => ()
			},
			Err(_) => println!("Failed to read")
		};
	}
}
