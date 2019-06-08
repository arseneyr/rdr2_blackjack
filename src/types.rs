use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::cmp;

use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::ops;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, EnumIter, Eq, Ord, PartialEq, PartialOrd, FromPrimitive, Hash)]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Deck {
  cards: [usize; 10],
  card_count: usize,
}

#[derive(Clone, Copy)]
pub struct DeckIterator<'a>(&'a Deck, usize, usize);

pub struct RankIterator<'a>(&'a Deck, usize);

impl Deck {
  pub fn new(deck_count: usize) -> Self {
    let mut cards = [deck_count * 4; 10];
    cards[9] *= 4;
    Self {
      cards,
      card_count: deck_count * 52,
    }
  }

  pub fn iter<'a>(&'a self) -> DeckIterator<'a> {
    DeckIterator(self, 0, 0)
  }

  pub fn rank_iter<'a>(&'a self) -> RankIterator<'a> {
    RankIterator(self, 0)
  }

  pub fn get_count_of_card(&self, card: Card) -> usize {
    self.cards[card as usize - 1]
  }

  pub fn get_count(&self) -> usize {
    self.card_count
  }

  pub fn remove_cards(&mut self, cards: &[Card]) {
    for card in cards {
      self.cards[*card as usize - 1] -= 1;
      self.card_count -= 1;
    }
  }

  pub fn add_cards(&mut self, cards: &[Card]) {
    for card in cards {
      self.cards[*card as usize - 1] += 1;
      self.card_count += 1;
    }
  }
}

impl ops::Sub for &Deck {
  type Output = Option<Deck>;
  fn sub(self, rhs: Self) -> Self::Output {
    if self.card_count < rhs.card_count {
      return None;
    }

    let mut ret = Deck {
      cards: [0; 10],
      card_count: self.card_count - rhs.card_count,
    };
    for i in 0..self.cards.len() {
      if self.cards[i] < rhs.cards[i] {
        return None;
      }
      ret.cards[i] = self.cards[i] - rhs.cards[i];
    }
    Some(ret)
  }
}

impl ops::Sub<Card> for &Deck {
  type Output = Option<Deck>;
  fn sub(self, rhs: Card) -> Self::Output {
    if self.cards[rhs as usize - 1] == 0 {
      return None;
    }

    let mut ret = self.clone();
    ret.card_count -= 1;
    ret.cards[rhs as usize - 1] -= 1;
    Some(ret)
  }
}

impl ops::Add<Card> for &Deck {
  type Output = Deck;
  fn add(self, rhs: Card) -> Self::Output {

    let mut ret = self.clone();
    ret.card_count += 1;
    ret.cards[rhs as usize - 1] += 1;
    ret
  }
}

impl<'a> From<&'a Deck> for &'a [usize; 10] {
  fn from(deck: &Deck) -> &[usize; 10] {
    &deck.cards
  }
}

impl<'a> Iterator for RankIterator<'a> {
  type Item = Card;

  fn next(&mut self) -> Option<Self::Item> {
    let Self(Deck { cards, .. }, rank) = self;

    while *rank < cards.len() && cards[*rank] == 0 {
      *rank += 1;
    }

    let ret = Card::from_usize(*rank + 1);
    *rank += 1;
    ret
  }
}

impl<'a> Iterator for DeckIterator<'a> {
  type Item = Card;

  fn next(&mut self) -> Option<Self::Item> {
    let Self(Deck { cards, .. }, rank, count) = self;

    while *rank < cards.len() && cards[*rank] == 0 {
      *rank += 1;
      *count = 0;
    }

    if (*rank) >= cards.len() {
      return None;
    }

    let ret = Card::from_usize(*rank + 1).unwrap();
    *count += 1;
    if *count == cards[*rank] {
      *rank += 1;
      *count = 0;
    }

    Some(ret)
  }
}

#[derive(Copy, Clone, Debug)]
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
    match (*self, *rhs) {
      (HandValue::Soft(_), HandValue::Hard(_)) => cmp::Ordering::Less,
      (HandValue::Hard(_), HandValue::Soft(_)) => cmp::Ordering::Greater,
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

impl ops::Add<Card> for HandValue {
  type Output = Self;

  fn add(self, rhs: Card) -> Self {
    let card_value = rhs as u32;
    match self {
      HandValue::Soft(x) => {
        if x + card_value > 21 {
          HandValue::Hard(x + card_value - 10)
        } else {
          HandValue::Soft(x + card_value)
        }
      }
      HandValue::Hard(x) => {
        if card_value == 1 && x <= 10 {
          HandValue::Soft(x + 11)
        } else {
          HandValue::Hard(x + card_value)
        }
      }
    }
  }
}


impl ops::AddAssign<Card> for HandValue {
  fn add_assign(&mut self, card: Card) {
    *self = *self + card;
  }
}

pub type Hand = Deck;

impl Hand {
  pub fn is_blackjack(&self) -> bool {
    *self
      == Hand {
        cards: [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        card_count: 2,
      }
  }
}

#[derive(Debug, Clone)]
pub struct CardMap<T> {
  array: [Option<T>; 10],
}

pub struct CardMapIter<'a, T> {
  map: &'a CardMap<T>,
  index: usize,
}

pub struct CardMapIterMut<'a, T> {
  map: &'a mut CardMap<T>,
  index: usize,
}

impl<T> CardMap<T> {
  pub fn new() -> CardMap<T> {
    CardMap {
      array: Default::default(),
    }
  }

  pub fn set(&mut self, card: Card, val: T) {
    self.array[card as usize - 1] = Some(val)
  }

  pub fn iter<'a>(&'a self) -> impl Iterator<Item = (Card, &'a T)> {
    self.array.iter().enumerate().filter_map(|(i, x)| {
      if let Some(y) = x {
        Some((Card::from_usize(i + 1).unwrap(), y))
      } else {
        None
      }
    })
  }

  pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (Card, &'a mut T)> {
    self.array.iter_mut().enumerate().filter_map(|(i, x)| {
      if let Some(y) = x {
        Some((Card::from_usize(i + 1).unwrap(), y))
      } else {
        None
      }
    })
  }
}

impl<T> CardMap<T>
where
  T: Copy,
{
  pub fn fill(&mut self, val: T) {
    self.array = [Some(val); 10];
  }
}

impl<T> fmt::Display for CardMap<T>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (i, t) in self.array.iter().enumerate() {
      match t {
        None => write!(f, "{:?}: None ", Card::from_usize(i + 1).unwrap())?,
        Some(x) => write!(f, "{:?}: {:?} ", Card::from_usize(i + 1).unwrap(), *x)?,
      }
    }
    Ok(())
  }
}

impl<T> ops::Index<Card> for CardMap<T> {
  type Output = Option<T>;

  fn index(&self, card: Card) -> &Self::Output {
    &self.array[card as usize - 1]
  }
}

impl<T> Default for CardMap<T>
where
  T: Default + Copy,
{
  fn default() -> Self {
    Self {
      array: [Some(T::default()); 10],
    }
  }
}
