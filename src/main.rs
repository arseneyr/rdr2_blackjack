extern crate strum;
extern crate strum_macros;

extern crate num_derive;
mod types;

use types::{Card, Deck, DeckIterator, Hand, HandValue};

use rand::thread_rng;

fn get_hand_value(hand: &Hand) -> HandValue {
    let mut sum: HandValue = HandValue::Hard(0);
    for card in hand.iter() {
        sum += *card;
    }
    sum
}

fn create_deck() -> Vec<Card> {
    let mut v = Vec::new();
    for _ in (0..4) {
        //for card in Card::iter() {
        //    v.push(card);
        //}
    }
    v
}

fn generate_single_hand<T>(all_hands: &mut Vec<Hand>, current_hand: &Hand, iter: &T)
where
    T: Iterator<Item = Card> + Clone,
{
    let mut i = iter.clone();
    let mut card: Option<Card> = None;
    loop {
        card = match (i.next(), card) {
            (Some(iter_card), None) => Some(iter_card),
            (Some(iter_card), Some(prev_card)) if iter_card != prev_card => Some(iter_card),
            (None, _) => break,
            _ => continue,
        };

        let mut new_hand = current_hand.clone();
        new_hand.push(card.unwrap());
        match get_hand_value(&new_hand) {
            HandValue::Hard(21) => {
                all_hands.push(new_hand);
                continue;
            }
            HandValue::Hard(x) if x > 21 => continue,
            _ => {
                all_hands.push(new_hand.clone());
                generate_single_hand(all_hands, &new_hand, &i);
            }
        };

    }
}

fn generate_all_hands() -> Vec<Hand> {
    let mut ret = Vec::new();
    let deck = Deck::new(10);
    generate_single_hand(&mut ret, &Vec::new(), &deck.iter());
    ret
}

fn shuffle_deck(deck: &mut [Card]) {
    //deck.shuffle(&mut thread_rng())
}

fn main() {
    let v = generate_all_hands();
    println!("{:?}", v.len());
}
