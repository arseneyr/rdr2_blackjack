use lib_dealer::{calculate_dealer_prob as cdp, DealerProb};
use crate::types::{Card, Deck, CardMap};
use num_traits::FromPrimitive;

pub fn calculate_dealer_prob(deck: &Deck) -> CardMap<DealerProb> {
  let mut ret = CardMap::new();
  for (i,p) in cdp(<&[usize; 10]>::from(deck)).iter().enumerate() {
    ret.set(Card::from_usize(i + 1).unwrap(), *p);
  }
  ret
}