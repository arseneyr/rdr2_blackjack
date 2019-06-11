use crate::types::{Card, CardMap, Deck};
use lazy_static::lazy_static;

use num_traits::FromPrimitive;
use std::collections::HashMap;

use std::sync::Mutex;

use lib_dealer::{calculate_dealer_prob as cdp, DealerProb};

lazy_static! {
  static ref CACHE: Mutex<HashMap<Deck, CardMap<DealerProb>>> = { Mutex::new(HashMap::new()) };
}

pub fn calculate_dealer_prob(deck: &Deck) -> CardMap<DealerProb> {
  if let Some(x) = CACHE.lock().unwrap().get(deck) {
    return x.clone();
  }
  let mut ret = CardMap::new();
  for (i, p) in cdp(<&[usize; 10]>::from(deck)).iter().enumerate() {
    ret.set(Card::from_usize(i + 1).unwrap(), *p);
  }
  CACHE.lock().unwrap().insert(deck.clone(), ret.clone());
  ret
}