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
 
// Choose a visible discard only when it improves the CPU's hand.
// Never take a Skip from the discard pile.
pub(super) fn cpu_should_take_discard(hand:&[Card], top:Card, phase:usize, laid:bool, table:&[Vec<Card>])->bool {
    if top.kind==Kind::Skip {return false;}
    if top.kind==Kind::Wild {return true;}
    if laid {
        return table.iter().enumerate().any(|(group, cards)|
            super::can_hit(cards,top,phase.min(9),group));
    }
    if phase>=super::PHASES.len() {return false;}
    let mut with_top=hand.to_vec();
    with_top.push(top);
    let top_index=with_top.len()-1;
    // A card is useful if it has better retention value than the weakest
    // existing card. Use the same phase-aware heuristic as CPU discarding.
    let top_priority=cpu_discard_priority(&with_top,top_index,phase);
    let weakest_existing=(0..hand.len())
        .map(|i|cpu_discard_priority(&with_top,i,phase))
        .max();
    weakest_existing.is_some_and(|weakest|top_priority<weakest)
}
