use lib_blackjack::{Card, Deck, Hand, SpecificHandEV};

fn main() {
  let mut deck = Deck::generate(1);
  deck.remove_cards(&[Card::Ace, Card::Ace, Card::Ten]);
  println!(
    "{:?}",
    SpecificHandEV::create(&deck, &Hand::from(&[Card::Ace, Card::Ace]), Card::Ten).split
  );
}