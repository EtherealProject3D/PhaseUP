use super::{Card, Kind, card_points};

// CPU discard priority: retain Wilds for completing phases; prefer shedding
// isolated numbers and Skips instead of donating Wilds to the human player.
pub(super) fn cpu_discard_priority(hand:&[Card], index:usize, phase:usize)->i32 {
    let card=hand[index];
    match card.kind {
        Kind::Wild => -1000,
        Kind::Skip => 150,
        Kind::Number(n,color) => {
            let mut partners=0;
            for (j,other) in hand.iter().enumerate() {
                if j==index {continue;}
                match other.kind {
                    Kind::Number(m,c) => {
                        if n==m {partners+=3;}
                        if (n as i32-m as i32).abs()<=2 {partners+=2;}
                        if color==c {partners+=if phase==7 {4}else{1};}
                    }
                    Kind::Wild => {partners+=3;}
                    Kind::Skip => {}
                }
            }
            card_points(card) as i32*2-partners*8
        }
    }
}
