use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::cmp;
use std::fmt;

use std::mem;
use std::ops;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, EnumIter, Eq, Ord, PartialEq, PartialOrd, FromPrimitive)]
pub enum Card {
  Ace = 1,
  Two,
  Three,
  Four,
  Five,
  Six,
  Seven,
  Eight,
  Nine,
  Ten,
}

#[derive(Debug)]
pub struct Deck([usize; 10]);

#[derive(Clone)]
pub struct DeckIterator<'a>(&'a Deck, usize, usize);

impl Deck {
  pub fn new(deck_count: usize) -> Self {
    let mut ret = [deck_count * 4; 10];
    ret[9] *= 4;
    Self(ret)
  }

  pub fn iter<'a>(&'a self) -> DeckIterator<'a> {
    DeckIterator(self, 0, 0)
  }

  pub fn remove_cards(&mut self, cards: &[Card]) {
    for card in cards {
      self.0[(*card as usize) - 1] -= 1;
    }
  }

  pub fn add_cards(&mut self, cards: &[Card]) {
    for card in cards {
      self.0[(*card as usize) - 1] += 1;
    }
  }
}

impl<'a> Iterator for DeckIterator<'a> {
  type Item = Card;

  fn next(&mut self) -> Option<Self::Item> {
    let Self(Deck(deck), rank, count) = self;

    while *rank < deck.len() && deck[*rank] == 0 {
      *rank += 1;
      *count = 0;
    }

    if (*rank) >= deck.len() {
      return None;
    }

    let ret = Card::from_usize(*rank + 1).unwrap();
    *count += 1;
    if *count == deck[*rank] {
      *rank += 1;
      *count = 0;
    }

    Some(ret)
  }
}

#[derive(Copy, Clone)]
pub enum HandValue {
  Hard(u32),
  Soft(u32),
}

impl fmt::Display for HandValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      HandValue::Hard(x) => write!(f, "Hard {}", x),
      HandValue::Soft(x) => write!(f, "Soft {}", x),
    }
  }
}
impl PartialEq for HandValue {
  fn eq(&self, other: &HandValue) -> bool {
    if mem::discriminant(self) != mem::discriminant(other) {
      return false;
    }

    u32::from(*self) == u32::from(*other)
  }
}
impl Eq for HandValue {}
impl Ord for HandValue {
  fn cmp(&self, rhs: &Self) -> cmp::Ordering {
    match (self, rhs) {
      (&HandValue::Soft(_), &HandValue::Hard(_)) => cmp::Ordering::Less,
      (&HandValue::Hard(_), &HandValue::Soft(_)) => cmp::Ordering::Greater,
      _ => u32::from(*self).cmp(&u32::from(*rhs)),
    }
  }
}
impl PartialOrd for HandValue {
  fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(rhs))
  }
}

impl From<HandValue> for u32 {
  fn from(hand_value: HandValue) -> u32 {
    match hand_value {
      HandValue::Hard(x) => x,
      HandValue::Soft(x) => x,
    }
  }
}


impl ops::AddAssign<Card> for HandValue {
  fn add_assign(&mut self, card: Card) {
    let card_value = card as u32;
    match *self {
      HandValue::Soft(x) => {
        if x + card_value > 21 {
          *self = HandValue::Hard(x + card_value - 10);
        } else {
          *self = HandValue::Soft(x + card_value);
        }
      }
      HandValue::Hard(x) => {
        if card_value == 1 && x <= 10 {
          *self = HandValue::Soft(x + 11);
        } else {
          *self = HandValue::Hard(x + card_value)
        }
      }
    };
  }
}

pub type Hand = Vec<Card>;