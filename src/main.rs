extern crate indexmap;
extern crate strum_macros;
extern crate lazy_static;


mod dealer_prob;
//mod sorted_hashmap;
mod types;

use lib_dealer::DealerProb;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::map::IndexMap;

use std::ops::Deref;
use std::ops::DerefMut;

use strum::IntoEnumIterator;
use types::{Card, CardMap, Deck, DeckIterator, Hand, HandValue};
#[derive(Debug)]
struct HandEV {
    hand: Hand,
    hand_value: HandValue,
    stand: CardMap<f64>,
    hit: CardMap<f64>,
    double: Option<CardMap<f64>>,
    split: Option<CardMap<f64>>,
}

fn get_hand_value(hand: &Hand) -> HandValue {
    let mut sum: HandValue = HandValue::Hard(0);
    for card in hand.iter() {
        sum += card;
    }
    sum
}

fn generate_hand(
    all_hands: &mut IndexMap<Hand, Rc<RefCell<HandEV>>>,
    current_hand: &mut Hand,
    iter: &DeckIterator,
) {
    let mut i = *iter;
    let mut card: Option<Card> = None;
    loop {
        card = match (i.next(), card) {
            (Some(iter_card), None) => Some(iter_card),
            (Some(iter_card), Some(prev_card)) if iter_card != prev_card => Some(iter_card),
            (None, _) => break,
            _ => continue,
        };
        let hand_value = get_hand_value(&current_hand) + card.unwrap();
        match hand_value {
            HandValue::Hard(x) if x > 21 => continue,
            v @ _ => {
                current_hand.add_cards(&[card.unwrap()]);
                all_hands.insert(
                    current_hand.clone(),
                    Rc::new(RefCell::new(HandEV {
                        hand: current_hand.clone(),
                        hand_value,
                        stand: CardMap::new(),
                        hit: CardMap::new(),
                        double: None,
                        split: None,
                    })),
                );
                if v != HandValue::Hard(21) {
                    generate_hand(all_hands, current_hand, &i);
                };
                current_hand.remove_cards(&[card.unwrap()]);
            }
        };
    }
}

fn generate_all_hands(deck: &Deck) -> IndexMap<Hand, Rc<RefCell<HandEV>>> {
    let mut ret = IndexMap::new();
    generate_hand(&mut ret, &mut Hand::new(), &deck.iter());
    ret
}

fn get_stand_ev(deck: &Deck, hand: &Hand, hand_value: HandValue) -> CardMap<f64> {
    let mut ev = CardMap::new();
    for (
        c,
        DealerProb {
            p_17,
            p_18,
            p_19,
            p_20,
            p_21,
            p_bust,
            p_bj,
        },
    ) in dealer_prob::calculate_dealer_prob(deck).iter()
    {
        if deck.get_count_of_card(c) == 0 {
            continue;
        }

        if hand.is_blackjack() {
            ev.set(c, 1.5 * (1.0 - p_bj));
            continue;
        }

        ev.set(
            c,
            p_bust - p_bj
                + match hand_value {
                    HandValue::Hard(17) | HandValue::Soft(17) => -p_18 - p_19 - p_20 - p_21,
                    HandValue::Hard(18) | HandValue::Soft(18) => p_17 - p_19 - p_20 - p_21,
                    HandValue::Hard(19) | HandValue::Soft(19) => p_17 + p_18 - p_20 - p_21,
                    HandValue::Hard(20) | HandValue::Soft(20) => p_17 + p_18 + p_19 - p_21,
                    HandValue::Hard(21) | HandValue::Soft(21) => p_17 + p_18 + p_19 + p_20,
                    _ => -p_17 - p_18 - p_19 - p_20 - p_21,
                },
        );
    }

    ev
}

fn get_hit_ev(
    deck: &Deck,
    all_hands: &IndexMap<Hand, Rc<RefCell<HandEV>>>,
    hand: &Hand,
    hand_value: HandValue,
) -> CardMap<f64> {
    let mut ev = CardMap::new();

    for up_card in deck.rank_iter() {

        if hand_value == HandValue::Hard(21) {
            ev.set(up_card, -1.0);
            continue;
        }
        let mut possible_card_count = 0;
        let new_deck = (deck - up_card).unwrap();
        for card in new_deck.rank_iter() {
            if card == up_card && new_deck.get_count_of_card(card) == 1 {
                continue;
            }
            possible_card_count += new_deck.get_count_of_card(card);
            ev.set(
                up_card,
                ev[up_card].unwrap_or(0.0)
                    + new_deck.get_count_of_card(card) as f64
                        * match hand_value + card {
                            HandValue::Hard(x) if x > 21 => -1.0,
                            _ => {
                                let hit_hand = all_hands.get(&(hand + card)).unwrap().borrow();
                                hit_hand.hit[up_card]
                                    .unwrap()
                                    .max(hit_hand.stand[up_card].unwrap())
                            }
                        },
            )

        }
        if let Some(x) = ev[up_card] {
            ev.set(up_card, x / possible_card_count as f64);
        }
    }
    ev
}

fn get_double_ev(
    deck: &Deck,
    all_hands: &IndexMap<Hand, Rc<RefCell<HandEV>>>,
    hand: &Hand,
    hand_value: HandValue,
) -> Option<CardMap<f64>> {
    if hand.get_count() != 2 || hand.is_blackjack() {
        return None;
    };

    let mut ev = CardMap::new();

    for up_card in deck.rank_iter() {
        let new_deck = (deck - up_card).unwrap();
        match up_card {
            Card::Ace => {
                ev.set(
                    Card::Ace,
                    new_deck.get_count_of_card(Card::Ten) as f64 / new_deck.get_count() as f64,
                );
            }
            Card::Ten => {
                ev.set(
                    Card::Ten,
                    new_deck.get_count_of_card(Card::Ace) as f64 / new_deck.get_count() as f64,
                );
            }
            _ => (),
        }
        for card in new_deck.rank_iter() {
            if card == up_card && new_deck.get_count_of_card(card) == 1 {
                continue;
            }
            ev.set(
                up_card,
                ev[up_card].unwrap_or(0.0)
                    + match hand_value + card {
                        HandValue::Hard(x) if x > 21 => -2.0,
                        _ => {
                            all_hands.get(&(hand + card)).unwrap().borrow().stand[up_card].unwrap()
                                * 2.0
                        }
                    } * (new_deck.get_count_of_card(card) as f64 / new_deck.get_count() as f64),
            )

        }
    }
    Some(ev)
}

fn get_split_ev_inner(
    deck: &Deck,
    all_hands: &IndexMap<Hand, Rc<RefCell<HandEV>>>,
    pair_card: Card,
    recurse: bool,
) -> CardMap<f64> {

    let mut ev = CardMap::new();
    let split_hands: IndexMap<Hand, Rc<RefCell<HandEV>>> = all_hands
        .iter()
        .filter_map(|(h, hev)| {
            if h.is_subset(deck) && h.get_count_of_card(pair_card) > 0 {
                Some((
                    h.clone(),
                    Rc::new(RefCell::new(HandEV {
                        hand: h.clone(),
                        hand_value: hev.borrow().hand_value,
                        stand: CardMap::default(),
                        hit: CardMap::default(),
                        double: None,
                        split: None,
                    })),
                ))
            } else {
                None
            }
        })
        .collect();

    for hand_ev in split_hands.values() {
        let hand = &hand_ev.borrow().hand;
        let new_deck = (deck - hand).unwrap();
        let hand_value = hand_ev.borrow().hand_value;
        let stand = get_stand_ev(&new_deck, hand, hand_value);
        let stand = get_hit_ev(&new_deck, &split_hands, hand, hand_value);

        if recurse {
            let other_hand_ev = get_split_ev_inner(&new_deck, &split_hands, pair_card, false);
        }
    }
    ev
}

fn get_split_ev(
    deck: &Deck,
    all_hands: &IndexMap<Hand, Rc<RefCell<HandEV>>>,
    hand: &Hand,
) -> Option<CardMap<f64>> {

    let pair_card = hand.iter().nth(0).unwrap();
    if hand.get_count() != 2 || pair_card != hand.iter().nth(1).unwrap() {
        return None;
    };
    Some(get_split_ev_inner(deck, all_hands, pair_card, true))
}

fn process_hand(
    deck: &Deck,
    all_hands: &IndexMap<Hand, Rc<RefCell<HandEV>>>,
    hand_ev: &RefCell<HandEV>,
) {
    let mut stand;
    let mut hit;
    let mut double;
    let mut split;
    {
        let hand = &hand_ev.borrow().hand;
        let hand_value = hand_ev.borrow().hand_value;

        let deck = (deck - hand).unwrap();
        stand = get_stand_ev(&deck, hand, hand_value);
        hit = get_hit_ev(&deck, all_hands, hand, hand_value);
        double = get_double_ev(&deck, all_hands, hand, hand_value);
        split = get_split_ev(&deck, all_hands, hand);
    }

    let mut hand_ev = hand_ev.borrow_mut();
    let hand_ev = hand_ev.deref_mut();
    hand_ev.stand = stand;
    hand_ev.hit = hit;
    hand_ev.double = double;
    hand_ev.split = split;
}


fn main() {
    let deck = Deck::generate(1);
    let mut hands = generate_all_hands(&deck);
    hands.sort_by(|_, a, _, b| {
        match (a.borrow().hand_value, b.borrow().hand_value) {
            // We must process all the soft values before doing any of the hard
            // values <= 10, because a hard 10 + ace is a soft 21
            (HandValue::Soft(_), HandValue::Hard(x)) if x <= 10 => cmp::Ordering::Greater,
            (HandValue::Hard(x), HandValue::Soft(_)) if x <= 10 => cmp::Ordering::Less,
            (h_a, h_b) => h_a.cmp(&h_b),
        }
        .reverse()
    });
    for hand in hands.values() {
        process_hand(&deck, &hands, hand);
    }
    for h in hands.values().filter(|x| x.borrow().hand.get_count() == 2) {
        println!("{:?}", h.borrow());
    }
    /*println!(
        "{:?}",
        hands
            .get(&(&(&Hand::new() + Card::Three) + Card::Eight))
            .unwrap()
    )*/
    /*for h in hands_ordered.iter() {
        println!("{:?}", h.borrow());
    }*/
}
