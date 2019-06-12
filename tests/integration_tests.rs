use lib_blackjack::*;

fn create_standard_deck() -> Deck {
  Deck::from([
    Card::Ace,
    Card::Two,
    Card::Three,
    Card::Four,
    Card::Five,
    Card::Six,
    Card::Seven,
    Card::Eight,
    Card::Nine,
    Card::Nine,
    Card::Nine,
    Card::Nine,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
    Card::Ten,
  ])
}

#[test]
fn standard() {
  let ev = compute_all_hand_ev(&create_standard_deck());
  let four_six = ev.get(&Hand::from([Card::Four, Card::Six])).unwrap();
  let mut stand = CardMap::new();
  stand.set(Card::Ace, -0.7835282651072124);
  stand.set(Card::Two, -0.03219159008632694);
  stand.set(Card::Three, 0.40888331940963524);
  stand.set(Card::Five, 0.6329156223893067);
  stand.set(Card::Seven, -0.5289612921191869);
  stand.set(Card::Eight, -0.8101503759398495);
  stand.set(Card::Nine, -0.7619604566972985);
  stand.set(Card::Ten, -0.7737399053188528);
  let mut hit = CardMap::new();
  hit.set(Card::Ace, -0.48962848297213635);
  hit.set(Card::Two, 0.33716562648451204);
  hit.set(Card::Three, 0.6647902435828132);
  hit.set(Card::Five, 0.9166396383114649);
  hit.set(Card::Seven, 0.6908848919684832);
  hit.set(Card::Eight, 0.5671630055530985);
  hit.set(Card::Nine, 0.3955726735138499);
  hit.set(Card::Ten, -0.003441201041820277);
  let mut double = CardMap::new();
  double.set(Card::Ace, -0.41047553524333746);
  double.set(Card::Two, 0.6743312529690241);
  double.set(Card::Three, 1.3295804871656265);
  double.set(Card::Five, 1.8332792766229298);
  double.set(Card::Seven, 1.37245237276197);
  double.set(Card::Eight, 1.1029223385260538);
  double.set(Card::Nine, 0.7633782167837894);
  double.set(Card::Ten, 0.008256752993595018);
  assert_eq!(
    *four_six,
    HandEV {
      hand: Hand::from([Card::Four, Card::Six]),
      hand_value: HandValue::Hard(10),
      stand,
      hit: Some(hit),
      double: Some(double),
      split: None
    }
  );
}

#[test]
fn small_deck() {
  let ev = compute_all_hand_ev(&Deck::from([
    Card::Eight,
    Card::Nine,
    Card::Nine,
    Card::Nine,
    Card::Ten,
    Card::Ten,
  ]));
  println!("{:?}", ev);
}