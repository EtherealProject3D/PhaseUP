use super::{Card, Kind, SortMode};
use macroquad::prelude::Color;
use macroquad::rand::gen_range;

pub(super) fn sort_key(card: Card, mode: SortMode) -> (u8,u8,u32) {
    match card.kind {
        Kind::Number(n,color) => match mode {
            SortMode::Runs => (n,color,card.id),
            SortMode::Colors => (color,n,card.id),
            SortMode::Sets => (n,color,card.id),
        },
        Kind::Wild => (250,0,card.id),
        Kind::Skip => (251,0,card.id),
    }
}
pub(super) fn card_points(c: Card) -> u32 { match c.kind { Kind::Number(n,_) if n<=9 => 5, Kind::Number(_,_) => 10, Kind::Skip => 15, Kind::Wild => 25 } }
pub(super) fn color_for(k: Kind) -> Color { match k { Kind::Number(_,0) => Color::new(0.96,0.34,0.39,1.), Kind::Number(_,1) => Color::new(0.25,0.65,0.98,1.), Kind::Number(_,2) => Color::new(0.99,0.78,0.31,1.), Kind::Number(_,_) => Color::new(0.37,0.83,0.54,1.), Kind::Wild => Color::new(0.66,0.44,0.97,1.), Kind::Skip => Color::new(0.62,0.67,0.75,1.) } }
pub(super) fn card_text(k: Kind) -> String { match k { Kind::Number(n,_) => n.to_string(), Kind::Wild => "W".into(), Kind::Skip => "S".into() } }
pub(super) fn make_deck() -> Vec<Card> {
    let mut deck = Vec::new(); let mut id=0;
    for color in 0..4 { for n in 1..=12 { for _ in 0..2 { deck.push(Card{id,kind:Kind::Number(n,color)}); id+=1; } } }
    for _ in 0..8 { deck.push(Card{id,kind:Kind::Wild}); id+=1; }
    for _ in 0..4 { deck.push(Card{id,kind:Kind::Skip}); id+=1; }
    shuffle(&mut deck); deck
}
pub(super) fn shuffle<T>(v: &mut [T]) { for i in (1..v.len()).rev() { let j=gen_range(0,i+1); v.swap(i,j); } }
