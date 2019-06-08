extern crate lib_dealer;
extern crate strum_macros;

mod dealer_prob;
mod types;

use lib_dealer::DealerProb;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cmp;
use std::collections::HashMap;

use std::ops::DerefMut;
use std::rc::Rc;

use strum::IntoEnumIterator;
use types::{Card, CardMap, Deck, DeckIterator, Hand, HandValue};
#[derive(Debug)]
struct HandEV {
    hand: Hand,
    hand_value: HandValue,
    stand: CardMap<f64>,
    hit: CardMap<f64>,
}

fn get_hand_value(hand: &Hand) -> HandValue {
    let mut sum: HandValue = HandValue::Hard(0);
    for card in hand.iter() {
        sum += card;
    }
    sum
}

fn generate_hand(
    all_hands: &mut HashMap<Hand, Rc<RefCell<HandEV>>>,
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

fn generate_all_hands(deck: &Deck) -> HashMap<Hand, Rc<RefCell<HandEV>>> {
    let mut ret = HashMap::new();
    generate_hand(&mut ret, &mut Hand::new(0), &deck.iter());
    ret
}

fn get_stand_ev(
    deck: &Deck,
    hand: &Hand,
    hand_value: HandValue,
    dealer_prob: &CardMap<DealerProb>,
) -> CardMap<f64> {
    let mut ev = CardMap::default();
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
    ) in dealer_prob.iter()
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
    all_hands: &HashMap<Hand, Rc<RefCell<HandEV>>>,
    hand: &Hand,
    hand_value: HandValue,
) -> CardMap<f64> {
    let mut ret = CardMap::default();

    if hand_value == HandValue::Hard(21) {
        ret.fill(-1.0);
        return ret;
    }

    for up_card in deck.rank_iter() {
        let new_deck = (deck - up_card).unwrap();
        for card in new_deck.rank_iter() {
            let card_prob = new_deck.get_count_of_card(card) as f64 / new_deck.get_count() as f64;
            ret.set(
                up_card,
                ret[up_card].unwrap()
                    + card_prob
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

    }
    ret
}

fn process_hand(
    deck: &Deck,
    all_hands: &HashMap<Hand, Rc<RefCell<HandEV>>>,
    hand_ev: &RefCell<HandEV>,
) {
    let mut hand_ev = hand_ev.borrow_mut();
    let HandEV {
        hand_value,
        hit,
        stand,
        hand,
    } = hand_ev.deref_mut();

    let deck = (deck - &*hand).unwrap();
    *stand = get_stand_ev(
        &deck,
        &hand,
        *hand_value,
        &dealer_prob::calculate_dealer_prob(&deck),
    );
    *hit = get_hit_ev(&deck, all_hands, hand, *hand_value);
    //println!("{:?} {}", hand, dealer_prob::calculate_dealer_prob((&deck)))
}


fn main() {
    let deck = Deck::new(1);
    let mut hands = generate_all_hands(&deck);
    let mut hands_ordered: Vec<_> = hands.values().cloned().collect();
    hands_ordered.sort_unstable_by(|a, b| {
        match (a.borrow().hand_value, b.borrow().hand_value) {
            // We must process all the soft values before doing any of the hard
            // values <= 10, because a hard 10 + ace is a soft 21
            (HandValue::Soft(_), HandValue::Hard(x)) if x <= 10 => cmp::Ordering::Greater,
            (HandValue::Hard(x), HandValue::Soft(_)) if x <= 10 => cmp::Ordering::Less,
            (h_a, h_b) => h_a.cmp(&h_b),
        }
        .reverse()
    });
    for hand in hands_ordered.iter() {
        process_hand(&deck, &hands, hand);
    }
    for h in hands_ordered
        .iter()
        .skip_while(|x| x.borrow().hand_value == HandValue::Hard(21))
    {
        println!("{:?}", h.borrow());
    }
}
