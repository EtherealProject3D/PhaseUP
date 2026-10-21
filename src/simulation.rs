//! Headless, seeded CPU-vs-CPU round simulations for regression and balancing.
//! These use the production AI decision functions and phase/hit rules without UI.
use crate::*;

#[derive(Default,Debug)]
struct Stats { rounds:u32,completed:u32,turns:u32,phases:u32,wilds_laid:u32,skips:u32,wins:[u32;5] }

fn random(state:&mut u64)->u64 {
    *state=state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state>>32
}
fn shuffle(cards:&mut [Card],state:&mut u64) {
    for i in (1..cards.len()).rev() {
        let j=(random(state) as usize)%(i+1);
        cards.swap(i,j);
    }
}
fn refill(deck:&mut Vec<Card>,discard:&mut Vec<Card>,state:&mut u64) {
    if deck.is_empty() && discard.len()>1 {
        let top=discard.pop().unwrap();
        deck.append(discard);
        shuffle(deck,state);
        discard.push(top);
    }
}
fn run_seed(seed:u64,round_limit:u32,turn_limit:u32)->Stats {
    let levels=[Difficulty::Novice,Difficulty::Easy,Difficulty::Normal,Difficulty::Hard,Difficulty::Expert];
    let mut state=seed;
    let mut stats=Stats::default();
    // Rotate the difficulty positions to reduce first-seat advantage.
    for round in 0..round_limit {
        let mut deck=make_deck();
        shuffle(&mut deck,&mut state);
        let mut hands=vec![Vec::<Card>::new();4];
        let mut tables=vec![Vec::<Vec<Card>>::new();4];
        let mut laid=[false;4];
        let mut phase=[0usize;4];
        let mut difficulty=[Difficulty::Normal;4];
        for p in 0..4 {
            difficulty[p]=levels[((round as usize)+p)%5];
            for _ in 0..10 {hands[p].push(deck.pop().unwrap());}
        }
        let mut discard=vec![deck.pop().unwrap()];
        let mut skipped=None;
        let mut winner=None;
        for turn in 0..turn_limit {
            let p=(turn as usize)%4;
            stats.turns+=1;
            if skipped==Some(p) {skipped=None;continue;}
            refill(&mut deck,&mut discard,&mut state);
            let visible:Vec<(usize,Vec<Vec<Card>>)>=(0..4).filter(|&j|laid[j])
                .map(|j|(phase[j],tables[j].clone())).collect();
            let take=discard.last().is_some_and(|&top| {
                if laid[p] {
                    cpu_should_take_visible_hit(top,phase[p],&tables[p],&visible,difficulty[p])
                }else{
                    cpu_should_take_discard_for_level(&hands[p],top,phase[p],false,&tables[p],difficulty[p])
                }
            });
            let drawn=if take {discard.pop()}else{deck.pop()};
            if let Some(card)=drawn {hands[p].push(card);}else{break;}
            if !laid[p] {
                if let Some(mut groups)=cpu_choose_phase(&hands[p],phase[p],difficulty[p]) {
                    order_table_groups(&mut groups,phase[p]);
                    let ids:Vec<u32>=groups.iter().flatten().map(|c|c.id).collect();
                    hands[p].retain(|c|!ids.contains(&c.id));
                    stats.wilds_laid+=groups.iter().flatten().filter(|c|c.kind==Kind::Wild).count() as u32;
                    tables[p]=groups;laid[p]=true;stats.phases+=1;
                }
            }
            if laid[p] {
                loop {
                    let mut hit=None;
                    'find: for (index,&card) in hands[p].iter().enumerate() {
                        for target in 0..4 {
                            if !laid[target] {continue;}
                            for group in 0..tables[target].len() {
                                if can_hit(&tables[target][group],card,phase[target],group) {
                                    hit=Some((index,target,group));break 'find;
                                }
                            }
                        }
                    }
                    let Some((index,target,group))=hit else{break;};
                    let card=hands[p].remove(index);
                    tables[target][group].push(card);
                    if requirement(phase[target],group)=="run" {order_run(&mut tables[target][group]);}
                }
            }
            if hands[p].is_empty() {winner=Some(p);break;}
            let next=(p+1)%4;
            let index=(0..hands[p].len()).max_by_key(|&j| {
                let card=hands[p][j];
                let penalty=(0..4).filter(|&target|target!=p).map(|target| {
                    let is_next=target==next;
                    cpu_discard_opponent_risk(card,phase[target],laid[target],&tables[target],
                        hands[target].len(),is_next)
                        *cpu_opponent_threat_multiplier(difficulty[p],laid[target],hands[target].len(),is_next)
                }).sum::<i32>();
                let skip=if card.kind==Kind::Skip {
                    cpu_skip_discard_adjustment(difficulty[p],laid[next],hands[next].len())
                }else{0};
                cpu_discard_priority_for_level(&hands[p],j,phase[p],difficulty[p])
                    -penalty*cpu_defense_weight(difficulty[p])+skip
            }).unwrap();
            let card=hands[p].remove(index);
            discard.push(card);
            if card.kind==Kind::Skip {stats.skips+=1;skipped=Some(next);}
            if hands[p].is_empty() {winner=Some(p);break;}
        }
        stats.rounds+=1;
        if let Some(p)=winner {
            stats.completed+=1;
            stats.wins[difficulty[p].level() as usize]+=1;
        }
    }
    stats
}

#[test]
fn seeded_cpu_rounds_complete_without_illegal_moves() {
    let stats=run_seed(0x50484153455550,8,250);
    assert_eq!(stats.rounds,8);
    assert!(stats.completed>0,"simulation never finished a round: {stats:?}");
    assert!(stats.phases>0,"no CPU laid a phase: {stats:?}");
    println!("CPU simulation: {stats:?}");
}

#[test]
fn seeded_cpu_simulation_is_reproducible() {
    let a=run_seed(20261020,4,200);
    let b=run_seed(20261020,4,200);
    assert_eq!(format!("{a:?}"),format!("{b:?}"));
}
