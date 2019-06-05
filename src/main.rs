extern crate strum;
extern crate strum_macros;

extern crate num_derive;

mod dealer_prob;
mod types;

use types::{Card, Deck, DeckIterator, Hand, HandValue};

fn get_hand_value(hand: &Hand) -> HandValue {
    let mut sum: HandValue = HandValue::Hard(0);
    for card in hand.iter() {
        sum += *card;
    }
    sum
}

fn generate_hand(all_hands: &mut Vec<Hand>, current_hand: Hand, iter: &DeckIterator) {
    let mut i = *iter;
    let mut card: Option<Card> = None;
    loop {
        card = match (i.next(), card) {
            (Some(iter_card), None) => Some(iter_card),
            (Some(iter_card), Some(prev_card)) if iter_card != prev_card => Some(iter_card),
            (None, _) => break,
            _ => continue,
        };
        match get_hand_value(&current_hand) + card.unwrap() {
            HandValue::Hard(x) if x > 21 => continue,
            v @ _ => {
                let mut new_hand = current_hand.clone();
                new_hand.push(card.unwrap());
                all_hands.push(new_hand.clone());
                match v {
                    HandValue::Hard(21) => continue,
                    _ => generate_hand(all_hands, new_hand, &i),
                };
            }
        };
    }
}

fn generate_all_hands() -> Vec<Hand> {
    let mut ret = Vec::new();
    let deck = Deck::new(1);
    generate_hand(&mut ret, Vec::new(), &deck.iter());
    ret
}

use dealer_prob::calculate_dealer_prob;

fn main() {
    let mut d = Deck::new(1);
    println!("{:?}", calculate_dealer_prob(&d));
}
