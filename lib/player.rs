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

pub mod player {
	use std::vec;
	use std::slice::Iter;
	use crate::card::card::*;

	#[no_mangle]
	pub struct Player {
		name: String,
		is_dealer: bool,
		hands: Vec<Hand>,
		balance: i32,
		standing: i32
	}

	#[no_mangle]
	#[derive(Clone)]
	pub struct Hand {
		cards: Vec<Card>,
		surrendered: bool,
		is_set: bool,
		wager: i32
	}

	impl Player {
		pub fn new(name: String, is_dealer: bool, balance: i32) -> Player {
			Player {
				name, is_dealer, hands: Vec::with_capacity(2), balance, standing: 0
			}
		}

		fn win(&mut self, amt: i32) {
			self.balance += amt;
			self.standing += amt;
		}

		fn lose(&mut self, amt: i32) {
			self.win(-amt);
		}

		pub fn hand_iter(&self) -> Iter<Hand> {
			self.hands.iter()
		}

		pub fn get_hand_count(&self) -> usize {
			self.hands.len()
		}

		pub fn get_hand_at(&self, idx: usize) -> Hand {
			self.hands[idx].clone()
		}

		pub fn get_name(&self) -> &str {
			&*self.name
		}

		pub fn get_balance(&self) -> i32 {
			self.balance
		}

		pub fn get_standing(&self) -> i32 {
			self.standing
		}

		fn get_playing_hand(&self) -> &Hand {
			for hand in &self.hands {
				if !hand.get_is_set() {
					return hand;
				}
			}
			panic!("Player has no hands");
		}

		fn get_playing_hand_mut(&mut self) -> &mut Hand {
			for hand in &mut self.hands {
				if !hand.get_is_set() {
					return hand;
				}
			}
			panic!("Player has no hands");
		}

		pub fn first_hand_value(&self) -> u32 {
			if self.has_busted() {
				return 0
			}
			self.hands[0].value(false)
		}

		pub fn surrender(&mut self) {
			self.get_playing_hand_mut().surrender();
			self.lose(self.get_playing_hand().get_wager() / 2);
		}

		pub fn has_lost(&self) -> bool {
			for hand in &self.hands {
				if !hand.lost() {
					return false;
				}
			}
			true
		}

		pub fn stand(&mut self) {
			self.get_playing_hand_mut().set();
		}

		pub fn bet(&mut self, wager: i32, deck: &mut Deck) {
			self.hands.push(Hand::new(wager, deck));
		}

		pub fn split(&mut self, deck: &mut Deck) -> bool {
			match self.get_playing_hand_mut().split(deck) {
				Some(newhand) => {
					self.hands.push(newhand);
					true
				},
				None => false
			}
		}

		pub fn double(&mut self, deck: &mut Deck) -> bool {
			self.get_playing_hand_mut().double_wager(deck)
		}

		pub fn hit(&mut self, deck: &mut Deck) -> bool {
			let hand = self.get_playing_hand_mut();
			let busted = hand.hit(deck);
			if busted {
				hand.set();
			}
			busted
		}

		pub fn has_busted(&self) -> bool {
			for hand in &self.hands {
				if hand.busted() {
					return true;
				}
			}
			false
		}

		pub fn hand_is_soft(&self) -> bool {
			self.get_playing_hand().is_soft()
		}

		pub fn play_as_dealer(&mut self, mut deck: &mut Deck) -> u32 {
			let hand = &mut self.hands[0];
			while hand.value(false) < 17 {
				hand.hit(&mut deck);
			}
			match self.has_busted() {
				true => 0,
				false => self.first_hand_value()
			}
		}

		pub fn is_playing(&self) -> bool {
			for hand in &self.hands {
				if !hand.get_is_set() {
					return true;
				}
			}
			false
		}

		pub fn game_over(&mut self, dealer_value: u32) {
			if self.is_dealer {
				self.hands.clear();
				return;
			}
			let mut total_delta: i32 = 0;
			for hand in &mut self.hands {
				let value = hand.value(false);
				let wager = hand.get_wager();
				if value > dealer_value && !hand.lost() {
					total_delta += wager;
				} else if value < dealer_value || hand.lost() {
					total_delta -= wager;
				}
			}
			self.win(total_delta);
			self.hands.clear();
		}
	}

	impl Hand {
		pub fn new(wager: i32, deck: &mut Deck) -> Hand {
			let mut hand = Hand { cards: Vec::with_capacity(11), surrendered: false, is_set: false, wager };
			for _ in 0..2 {
				hand.cards.push(deck.next_card())
			}
			hand
		}

		pub fn card_iter(&self) -> Iter<Card> {
			self.cards.iter()
		}

		pub fn get_card_count(&self) -> usize {
			self.cards.len()
		}

		pub fn get_card_at(&self, idx: usize) -> Card {
			self.cards[idx].clone()
		}

		pub fn set(&mut self) {
			self.is_set = true;
		}

		pub fn get_is_set(&self) -> bool {
			self.is_set
		}

		pub fn surrender(&mut self) {
			self.surrendered = true;
		}

		pub fn lost(&self) -> bool {
			self.surrendered || self.busted()
		}

		pub fn split(&mut self, deck: &mut Deck) -> Option<Hand> {
			if self.cards.len() == 2 {
				if self.cards[0].score() == self.cards[1].score() {
					let card = self.cards[1];
					self.cards[1] = deck.next_card();
					return Some(Hand {
						cards: vec![card, deck.next_card()], surrendered: false, is_set: false, wager: self.wager
					});
				}
			}
			None
		}

		pub fn double_wager(&mut self, deck: &mut Deck) -> bool {
			self.wager *= 2;
			self.is_set = true;
			self.hit(deck)
		}

		pub fn hit(&mut self, deck: &mut Deck) -> bool {
			self.cards.push(deck.next_card());
			self.busted()
		}

		pub fn busted(&self) -> bool {
			self.value(false) > 21
		}

		pub fn get_wager(&self) -> i32 {
			self.wager
		}

		pub fn value(&self, get_aces: bool) -> u32 {
			let mut total: u32 = 0;
			let mut aces: u32 = 0;
			for card in &self.cards {
				let value = card.score();
				if value == 11 {
					aces += 1
				}
				total += value;
			}
			while total > 21 && aces > 0 {
				total -= 10;
				aces -= 1;
			}
			match get_aces {
				true => aces,
				false => total
			}
		}

		pub fn is_soft(&self) -> bool {
			self.value(true) > 0
		}
	}
}
