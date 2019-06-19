extern crate indexmap;
extern crate strum_macros;

mod dealer_prob;
mod types;

use lib_dealer::DealerProb;
use std::cell::RefCell;
use std::cmp;

use indexmap::map::IndexMap;

use dealer_prob::DealerProbCalculator;
use std::collections::HashMap;
use std::ops::Deref;

pub use types::{Card, CardMap, Deck, Hand, HandValue};

use types::DeckIterator;

#[derive(Debug, PartialEq)]
pub struct HandEV {
    hand: Hand,
    hand_value: HandValue,
    pub stand: CardMap<f64>,
    pub hit: Option<CardMap<f64>>,
    pub double: Option<CardMap<f64>>,
    pub split: Option<CardMap<f64>>,
    other_split_ev: Option<CardMap<f64>>,
}

fn generate_hand(
    all_hands: &mut IndexMap<Hand, RefCell<HandEV>>,
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
        match current_hand.get_hand_value() + card.unwrap() {
            HandValue::Hard(x) if x > 21 => continue,
            v @ _ => {
                *current_hand += card.unwrap();
                if current_hand.get_count() >= 2 {
                    all_hands.insert(
                        current_hand.clone(),
                        RefCell::new(HandEV {
                            hand: current_hand.clone(),
                            hand_value: v,
                            stand: CardMap::new(),
                            hit: None,
                            double: None,
                            split: None,
                            other_split_ev: None,
                        }),
                    );
                }
                if v != HandValue::Hard(21) {
                    generate_hand(all_hands, current_hand, &i);
                };
                *current_hand -= card.unwrap();
            }
        };
    }
}

fn generate_all_hands(deck: &Deck) -> IndexMap<Hand, RefCell<HandEV>> {
    let mut ret = IndexMap::new();
    generate_hand(&mut ret, &mut Hand::new(), &deck.iter());
    ret
}

fn get_stand_ev(
    dealer_calc: &mut DealerProbCalculator,
    deck: &Deck,
    hand: &Hand,
    hand_value: HandValue,
    is_split: bool,
) -> CardMap<f64> {
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
    ) in dealer_calc.calculate(deck).iter()
    {
        if deck.get_count_of_card(c) == 0 {
            continue;
        }

        if !is_split && hand.is_blackjack() {
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
    all_hands: &IndexMap<Hand, RefCell<HandEV>>,
    hand: &Hand,
    hand_value: HandValue,
    split_ev: Option<&CardMap<f64>>,
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
            let card_count = new_deck.get_count_of_card(card);
            match hand_value + card {
                HandValue::Hard(x) if x > 21 => {
                    possible_card_count += card_count;
                    ev.set(
                        up_card,
                        ev[up_card].unwrap_or(0.0)
                            + (-1.0 + split_ev.map_or(0.0, |o| o[up_card].unwrap_or(0.0)))
                                * card_count as f64,
                    );
                }
                _ => {
                    let hit_hand = all_hands.get(&(hand + card)).unwrap().borrow();
                    possible_card_count += card_count;
                    match (
                        hit_hand.hit.as_ref().unwrap()[up_card],
                        hit_hand.stand[up_card],
                        hit_hand.other_split_ev.as_ref(),
                    ) {
                        (Some(h), Some(s), o) => ev.set(
                            up_card,
                            ev[up_card].unwrap_or(0.0)
                                + card_count as f64
                                    * h.max(s + o.map_or(0.0, |x| x[up_card].unwrap_or(0.0))),
                        ),
                        (None, Some(s), o) => ev.set(
                            up_card,
                            ev[up_card].unwrap_or(0.0)
                                + card_count as f64
                                    * (s + o.map_or(0.0, |x| x[up_card].unwrap_or(0.0))),
                        ),
                        _ => possible_card_count -= card_count,
                    }
                }
            }
        }
        if let Some(x) = ev[up_card] {
            ev.set(up_card, x / possible_card_count as f64);
        }
    }
    ev
}

fn get_double_ev(
    deck: &Deck,
    all_hands: &IndexMap<Hand, RefCell<HandEV>>,
    hand: &Hand,
    hand_value: HandValue,
    split_ev: Option<&CardMap<f64>>,
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
            ev.set(
                up_card,
                ev[up_card].unwrap_or(0.0)
                    + match hand_value + card {
                        HandValue::Hard(x) if x > 21 => {
                            -2.0 + split_ev.map_or(0.0, |o| o[up_card].unwrap_or(0.0))
                        }
                        _ => {
                            let hit_hand = all_hands.get(&(hand + card)).unwrap().borrow();
                            hit_hand.stand[up_card].unwrap() * 2.0
                                + hit_hand
                                    .other_split_ev
                                    .as_ref()
                                    .map_or(0.0, |o| o[up_card].unwrap_or(0.0))
                        }
                    } * (new_deck.get_count_of_card(card) as f64 / new_deck.get_count() as f64),
            )

        }
    }
    Some(ev)
}

fn get_split_ev_inner(
    dealer_calc: &mut DealerProbCalculator,
    deck: &Deck,
    all_hands: &IndexMap<Hand, RefCell<HandEV>>,
    pair_card: Card,
    recurse: bool,
) -> CardMap<f64> {

    let mut ev: CardMap<f64> = CardMap::new();
    let deck = &(deck + pair_card);
    let split_hands: IndexMap<Hand, RefCell<HandEV>> = all_hands
        .iter()
        .filter_map(|(h, hev)| {
            if (deck - h).is_some() && h.get_count_of_card(pair_card) > 0 {
                Some((
                    h.clone(),
                    RefCell::new(HandEV {
                        hand: h.clone(),
                        hand_value: hev.borrow().hand_value,
                        stand: CardMap::default(),
                        hit: None,
                        double: None,
                        split: None,
                        other_split_ev: None,
                    }),
                ))
            } else {
                None
            }
        })
        .collect();

    for hand_ev in split_hands.values() {
        let stand;
        let mut hit = None;
        let mut double = None;
        let mut other_split_ev = None;
        {
            let hand_ev = hand_ev.borrow();
            let HandEV {
                hand, hand_value, ..
            } = hand_ev.deref();
            let new_deck = (deck - &*hand).unwrap();
            stand = get_stand_ev(dealer_calc, &new_deck, &hand, *hand_value, true);


            if recurse {
                other_split_ev = Some(get_split_ev_inner(
                    dealer_calc,
                    &new_deck,
                    &split_hands,
                    pair_card,
                    false,
                ));
            }
            if pair_card != Card::Ace {
                hit = Some(get_hit_ev(
                    &new_deck,
                    &split_hands,
                    &hand,
                    *hand_value,
                    other_split_ev.as_ref(),
                ));
                double = get_double_ev(
                    &new_deck,
                    &split_hands,
                    &hand,
                    *hand_value,
                    other_split_ev.as_ref(),
                );
            }
        }
        let mut hand_ev = hand_ev.borrow_mut();
        hand_ev.stand = stand;
        hand_ev.hit = hit;
        hand_ev.double = double;
        hand_ev.other_split_ev = other_split_ev;
    }
    let deck = &(deck - pair_card).unwrap();
    for up_card in deck.rank_iter() {
        let new_deck = (deck - up_card).unwrap();
        for player_card in new_deck.rank_iter() {
            let hand_ev = split_hands
                .get(&Hand::from([pair_card, player_card]))
                .unwrap()
                .borrow();
            if hand_ev.stand[up_card] == None {
                continue;
            }
            ev.set(
                up_card,
                ev[up_card].unwrap_or(0.0)
                    + (new_deck.get_count_of_card(player_card) as f64
                        / new_deck.get_count() as f64)
                        * match hand_ev.deref() {
                            HandEV {
                                stand,
                                hit: Some(h),
                                double: Some(d),
                                other_split_ev: o,
                                ..
                            } => (stand[up_card].unwrap()
                                + if let Some(o) = o {
                                    o[up_card].unwrap_or(0.0)
                                } else {
                                    0.0
                                })
                            .max(h[up_card].unwrap_or(std::f64::MIN))
                            .max(d[up_card].unwrap_or(std::f64::MIN)),
                            HandEV {
                                stand,
                                hit: Some(h),
                                other_split_ev: o,
                                ..
                            } => (stand[up_card].unwrap()
                                + if let Some(o) = o {
                                    o[up_card].unwrap_or(0.0)
                                } else {
                                    0.0
                                })
                            .max(h[up_card].unwrap_or(std::f64::MIN)),
                            HandEV {
                                stand,
                                other_split_ev: o,
                                ..
                            } => {
                                (stand[up_card].unwrap()
                                    + if let Some(o) = o {
                                        o[up_card].unwrap_or(0.0)
                                    } else {
                                        0.0
                                    })
                            }
                        },
            )
        }
    }
    ev
}

fn get_split_ev(
    dealer_calc: &mut DealerProbCalculator,
    deck: &Deck,
    all_hands: &IndexMap<Hand, RefCell<HandEV>>,
    hand: &Hand,
) -> Option<CardMap<f64>> {

    let pair_card = hand.iter().nth(0).unwrap();
    if hand.get_count() != 2 || pair_card != hand.iter().nth(1).unwrap() {
        return None;
    };
    let mut ev = get_split_ev_inner(dealer_calc, deck, all_hands, pair_card, true);

    // Account for dealer blackjack
    match (
        deck.get_count_of_card(Card::Ten),
        deck.get_count_of_card(Card::Ace),
        deck.get_count(),
    ) {
        (t, a, d) if t > 0 && a > 0 => {
            ev.set(
                Card::Ten,
                ev[Card::Ten].unwrap() + a as f64 / (d - 1) as f64,
            );
            ev.set(
                Card::Ace,
                ev[Card::Ace].unwrap() + t as f64 / (d - 1) as f64,
            );
        }
        _ => (),
    }
    Some(ev)
}

fn process_hand(
    dealer_calc: &mut DealerProbCalculator,
    deck: &Deck,
    all_hands: &IndexMap<Hand, RefCell<HandEV>>,
    hand_ev: &RefCell<HandEV>,
) {
    let stand;
    let hit;
    let double;
    let split;
    {
        let hand_ev = hand_ev.borrow();
        let HandEV {
            hand, hand_value, ..
        } = hand_ev.deref();

        let deck = (deck - hand).unwrap();
        stand = get_stand_ev(dealer_calc, &deck, hand, *hand_value, false);
        hit = get_hit_ev(&deck, all_hands, hand, *hand_value, None);
        double = get_double_ev(&deck, all_hands, hand, *hand_value, None);
        split = get_split_ev(dealer_calc, &deck, all_hands, hand);
    }

    let mut hand_ev = hand_ev.borrow_mut();
    hand_ev.stand = stand;
    hand_ev.hit = Some(hit);
    hand_ev.double = double;
    hand_ev.split = split;
}


pub fn compute_all_hand_ev(starting_deck: &Deck) -> HashMap<Hand, HandEV> {
    let mut dealer_calc = DealerProbCalculator::new();
    let mut hands = generate_all_hands(&starting_deck);
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
        process_hand(&mut dealer_calc, &starting_deck, &hands, hand);
    }
    hands
        .into_iter()
        .map(|(h, hev)| (h, hev.into_inner()))
        .collect::<HashMap<Hand, HandEV>>()
}

//0.0015485589837292632 for standard deck
//0.0032576242968898536 if split BJ pays out 3:2

pub fn compute_overall_prob(deck: &Deck, evs: &HashMap<Hand, HandEV>) -> f64 {
    let mut ret = 0.0;
    for up_card in deck.rank_iter() {
        let mut upcard_ev = 0.0;
        {
            let deck = (deck - up_card).unwrap();
            for card1 in deck.rank_iter() {
                let p = deck.get_card_prob(&card1);
                {
                    let deck = (&deck - card1).unwrap();
                    for card2 in deck.rank_iter() {
                        if let Some(ev) = evs.get(&Hand::from(&[card1, card2])) {
                            upcard_ev += p
                                * deck.get_card_prob(&card2)
                                * [
                                    ev.stand[up_card].unwrap(),
                                    ev.hit
                                        .as_ref()
                                        .map_or(std::f64::MIN, |f| f[up_card].unwrap()),
                                    ev.double
                                        .as_ref()
                                        .map_or(std::f64::MIN, |f| f[up_card].unwrap()),
                                    ev.split
                                        .as_ref()
                                        .map_or(std::f64::MIN, |f| f[up_card].unwrap()),
                                ]
                                .iter()
                                .cloned()
                                .fold(std::f64::MIN, f64::max)
                        } else {
                            continue;
                        }
                    }
                }
            }

        }
        ret += upcard_ev * deck.get_card_prob(&up_card);
    }
    ret
}
