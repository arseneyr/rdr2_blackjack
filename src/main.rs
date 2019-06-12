use lib_blackjack::{compute_all_hand_ev, Deck};

fn main() {
  compute_all_hand_ev(&Deck::generate(1));
}