use crate::types::{Card, CardMap, Deck};

use num_traits::FromPrimitive;
use std::collections::HashMap;

use lib_dealer::{calculate_dealer_prob as cdp, DealerProb};

pub struct DealerProbCalculator(HashMap<Deck, CardMap<DealerProb>>);

impl DealerProbCalculator {
  pub fn new() -> DealerProbCalculator {
    DealerProbCalculator(HashMap::new())
  }

  pub fn calculate(&mut self, deck: &Deck) -> &CardMap<DealerProb> {
    self.0.entry(deck.clone()).or_insert_with(|| {
      let mut ret = CardMap::new();
      for (i, p) in cdp(<&[usize; 10]>::from(deck)).iter().enumerate() {
        ret.set(Card::from_usize(i + 1).unwrap(), *p);
      }
      ret
    })
  }
}