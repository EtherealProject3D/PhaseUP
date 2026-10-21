use super::{Card, Kind, card_points};

// CPU discard priority: retain Wilds for completing phases; prefer shedding
// isolated numbers and Skips instead of donating Wilds to the human player.
pub(super) fn cpu_discard_priority(hand:&[Card], index:usize, phase:usize)->i32 {
    let card=hand[index];
    match card.kind {
        Kind::Wild => -1000,
        Kind::Skip => 150,
        Kind::Number(n,color) => {
            // Match the current phase's actual group requirements instead
            // of valuing sets, runs and colors equally on every phase.
            let wants_set=matches!(phase,0|1|2|6|8|9);
            let wants_run=matches!(phase,1|2|3|4|5);
            let wants_color=phase==7;
            let mut support=0;
            for (j,other) in hand.iter().enumerate() {
                if j==index {continue;}
                match other.kind {
                    Kind::Number(m,c) => {
                        if wants_set && n==m {support+=5;}
                        if wants_run {
                            let distance=(n as i32-m as i32).abs();
                            if distance==1 {support+=5;}
                            else if distance==2 {support+=3;}
                            else if distance==3 {support+=1;}
                        }
                        if wants_color && color==c {support+=6;}
                    }
                    Kind::Wild => {support+=4;}
                    Kind::Skip => {}
                }
            }
            card_points(card) as i32*2-support*8
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

/// Public-information risk of giving a discard to an opponent. A player who
/// has already laid their phase can immediately use a hittable card; players
/// with very few cards left are especially dangerous. No hidden hands read.
pub(super) fn cpu_discard_opponent_risk(
    card:Card, opponent_phase:usize, opponent_laid:bool,
    opponent_table:&[Vec<Card>], opponent_hand_count:usize,
    is_next_player:bool,
)->i32 {
    if card.kind==Kind::Skip || !opponent_laid {return 0;}
    let hittable=opponent_table.iter().enumerate().any(|(group, cards)|
        super::can_hit(cards,card,opponent_phase.min(9),group));
    if !hittable {return 0;}
    // The next player can take this discard immediately. A later player
    // might never see it, so their penalty is deliberately smaller.
    let mut risk=if is_next_player {500}else{150};
    if opponent_hand_count<=2 {risk+=if is_next_player {450}else{100};}
    else if opponent_hand_count<=4 {risk+=if is_next_player {200}else{50};}
    risk
}

 
// Difficulty affects decisions, not rule legality. Novice makes simple
// choices, Easy notices basic combinations, and Normal+ use phase strategy.
pub(super) fn cpu_discard_priority_for_level(hand:&[Card],index:usize,phase:usize,level:super::Difficulty)->i32 {
    let card=hand[index];
    match level {
        super::Difficulty::Novice => match card.kind {
            Kind::Wild => -1000,
            Kind::Skip => 150,
            _ => card_points(card) as i32*2,
        },
        super::Difficulty::Easy => {
            // Basic matching-number recognition, independent of phase.
            let bonus=if let Kind::Number(n,_)=card.kind {
                hand.iter().enumerate().filter(|(j,c)|*j!=index && matches!(c.kind,Kind::Number(m,_) if m==n)).count() as i32*32
            }else{0};
            cpu_discard_priority_for_level(hand,index,phase,super::Difficulty::Novice)-bonus
        },
        _ => cpu_discard_priority(hand,index,phase),
    }
}

pub(super) fn cpu_should_take_discard_for_level(
    hand:&[Card],top:Card,phase:usize,laid:bool,table:&[Vec<Card>],
    level:super::Difficulty,
)->bool {
    if top.kind==Kind::Skip {return false;}
    if top.kind==Kind::Wild {return true;}
    if laid {return cpu_should_take_discard(hand,top,phase,laid,table);}
    if level==super::Difficulty::Novice {return false;}
    if level!=super::Difficulty::Easy {return cpu_should_take_discard(hand,top,phase,laid,table);}
    let mut combined=hand.to_vec();
    combined.push(top);
    let top_idx=combined.len()-1;
    let top_score=cpu_discard_priority_for_level(&combined,top_idx,phase,level);
    (0..hand.len()).any(|i|top_score<cpu_discard_priority_for_level(&combined,i,phase,level))
}

pub(super) fn cpu_defense_weight(level:super::Difficulty)->i32 {
    match level {
        super::Difficulty::Novice|super::Difficulty::Easy => 0,
        super::Difficulty::Normal => 1,
        super::Difficulty::Hard => 2,
        super::Difficulty::Expert => 3,
    }
}


/// A Skip always targets the next player under the current rules.
/// Advanced CPUs save it when that player is safe and spend it when
/// public hand size and a laid phase signal an imminent round finish.
pub(super) fn cpu_skip_discard_adjustment(
    level:super::Difficulty,next_laid:bool,next_hand_count:usize,
)->i32 {
    match level {
        super::Difficulty::Novice|super::Difficulty::Easy|super::Difficulty::Normal => 0,
        super::Difficulty::Hard => {
            if next_laid && next_hand_count<=2 {450}
            else if next_laid && next_hand_count<=4 {180}
            else {-180}
        },
        super::Difficulty::Expert => {
            if next_laid && next_hand_count<=2 {750}
            else if next_laid && next_hand_count<=4 {300}
            else {-280}
        },
    }
}


/// Find a legal phase layout. Advanced CPUs compare all eligible card subsets
/// rather than accepting the first valid one, preserving valuable Wilds and
/// reducing the points left in hand when possible.
pub(super) fn cpu_choose_phase(
    hand:&[Card],phase:usize,level:super::Difficulty,
)->Option<Vec<Vec<Card>>> {
    if phase>=super::PHASES.len(){return None;}
    let needed:usize=super::PHASES[phase].iter().sum();
    if needed>hand.len(){return None;}
    let advanced=matches!(level,super::Difficulty::Hard|super::Difficulty::Expert);
    let mut best:Option<(i32,Vec<Vec<Card>>)>=None;
    fn search(
        hand:&[Card],phase:usize,level:super::Difficulty,advanced:bool,
        needed:usize,start:usize,indices:&mut Vec<usize>,
        best:&mut Option<(i32,Vec<Vec<Card>>)>,
    )->bool {
        if indices.len()==needed {
            let subset:Vec<Card>=indices.iter().map(|&j|hand[j]).collect();
            if let Some(groups)=super::partition_phase(&subset,phase) {
                if !advanced {
                    *best=Some((0,groups));
                    return true;
                }
                let wilds=subset.iter().filter(|c|c.kind==Kind::Wild).count() as i32;
                let points=subset.iter().map(|&c|card_points(c) as i32).sum::<i32>();
                // Hard conserves Wilds; Expert conserves them even more strongly.
                // Among equal Wild usage, shedding high-value cards reduces risk.
                let weight=if level==super::Difficulty::Expert {200}else{100};
                let score=points-wilds*weight;
                if best.as_ref().is_none_or(|(current,_)|score>*current) {
                    *best=Some((score,groups));
                }
            }
            return false;
        }
        let missing=needed-indices.len();
        if start+missing>hand.len(){return false;}
        for j in start..=hand.len()-missing {
            indices.push(j);
            if search(hand,phase,level,advanced,needed,j+1,indices,best) {return true;}
            indices.pop();
        }
        false
    }
    search(hand,phase,level,advanced,needed,0,&mut Vec::new(),&mut best);
    best.map(|(_,groups)|groups)
}
