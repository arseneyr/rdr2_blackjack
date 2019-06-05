use crate::types::{Card, Deck, DeckIterator, Hand, HandValue};


fn get_hand_value(hand: &Hand) -> HandValue {
  let mut sum: HandValue = HandValue::Hard(0);
  for card in hand.iter() {
    sum += *card;
  }
  sum
}

fn generate_hand(all_hands: &mut Vec<Hand>, current_hand: Hand, deck: Deck) {
  let mut card: Option<Card> = None;
  let mut iter = deck.iter();
  loop {
    card = match (iter.next(), card) {
      (Some(iter_card), None) => Some(iter_card),
      (Some(iter_card), Some(prev_card)) if iter_card != prev_card => Some(iter_card),
      (None, _) => break,
      _ => continue,
    };
    match get_hand_value(&current_hand) + card.unwrap() {
      HandValue::Hard(x) if x > 21 => continue,
      HandValue::Soft(x) | HandValue::Hard(x) if x >= 17 => {
        let mut new_hand = current_hand.clone();
        new_hand.push(card.unwrap());
        all_hands.push(new_hand.clone());
        continue;
      }
      _ => {
        let mut new_hand = current_hand.clone();
        new_hand.push(card.unwrap());
        let mut new_deck = deck.clone();
        new_deck.remove_cards(&[card.unwrap()]);
        generate_hand(all_hands, new_hand, new_deck);
      }
    };
  }
}

pub fn generate_all_hands() -> Vec<Hand> {
  let deck = Deck::new(3);
  let mut ret = Vec::new();
  generate_hand(&mut ret, Hand::new(), deck);
  for hand in ret.iter_mut() {
    hand.sort();
  }
  ret.sort();
  ret.dedup();
  ret
}