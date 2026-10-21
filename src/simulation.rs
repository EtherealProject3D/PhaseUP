//! Headless, seeded CPU-vs-CPU round simulations for regression and balancing.
//! These use the production AI decision functions and phase/hit rules without UI.
use crate::*;

#[derive(Default,Debug)]
struct Stats { rounds:u32,completed:u32,turns:u32,phases:u32,wilds_laid:u32,skips:u32,wins:[u32;3], games:u32, game_wins:[u32;3], game_rounds:u32, stalled_rounds:u32, exhausted_rounds:u32, max_round_turns:u32, seat_wins:[u32;4], round_entries:[u32;3], game_entries:[u32;3], longest_stall_phase:[usize;4], stall_phase_counts:[u32;10], stall_unlaid:[u32;3], stall_discard_draws:u32, stall_stock_draws:u32 }

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
    let levels=[Difficulty::Easy,Difficulty::Normal,Difficulty::Hard,Difficulty::Hard];
    let mut state=seed;
    let mut stats=Stats::default();
    let mut progress=[0usize;4];
    let mut scores=[0u32;4];
    let mut game_round=0u32;
    let mut game_index=0usize;
    // Rotate difficulty seats between completed games.
    for _round in 0..round_limit {
        let mut deck=Vec::new();
        let mut id=0u32;
        for color in 0..4 {for n in 1..=12 {for _ in 0..2 {
            deck.push(Card{id,kind:Kind::Number(n,color)});id+=1;
        }}}
        for _ in 0..8 {deck.push(Card{id,kind:Kind::Wild});id+=1;}
        for _ in 0..4 {deck.push(Card{id,kind:Kind::Skip});id+=1;}
        shuffle(&mut deck,&mut state);
        let mut hands=vec![Vec::<Card>::new();4];
        let mut tables=vec![Vec::<Vec<Card>>::new();4];
        let mut laid=[false;4];
        let phase=progress;
        let mut difficulty=[Difficulty::Normal;4];
        for p in 0..4 {
            difficulty[p]=levels[(game_index+p)%levels.len()];
            for _ in 0..10 {hands[p].push(deck.pop().unwrap());}
        }
        for &level in &difficulty {stats.round_entries[(level.level()-1) as usize]+=1;}
        let mut discard=vec![deck.pop().unwrap()];
        let mut skipped=None;
        let mut winner=None;
        let mut exhausted=false;
        let mut round_turns=0u32;
        let mut discard_draws=0u32;
        let mut stock_draws=0u32;
        for turn in 0..turn_limit {
            let p=(turn as usize+game_index)%4;
            stats.turns+=1;
            round_turns+=1;
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
            let drawn=if take {discard_draws+=1;discard.pop()}else{stock_draws+=1;deck.pop()};
            if let Some(card)=drawn {hands[p].push(card);}else{exhausted=true;break;}
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
                    -cpu_bounded_defense_penalty(penalty,difficulty[p])+skip
            }).unwrap();
            let card=hands[p].remove(index);
            discard.push(card);
            if card.kind==Kind::Skip {stats.skips+=1;skipped=Some(next);}
            if hands[p].is_empty() {winner=Some(p);break;}
        }
        stats.rounds+=1;
        stats.max_round_turns=stats.max_round_turns.max(round_turns);
        if let Some(p)=winner {
            stats.seat_wins[p]+=1;
            stats.completed+=1;
            stats.wins[(difficulty[p].level()-1) as usize]+=1;
            for j in 0..4 {
                scores[j]+=hands[j].iter().map(|&card|card_points(card)).sum::<u32>();
                if laid[j] {progress[j]+=1;}
            }
            game_round+=1;
            if progress.iter().any(|&v|v>=10) {
                let champion=(0..4).filter(|&j|progress[j]>=10).min_by_key(|&j|(scores[j],j)).unwrap();
                stats.games+=1;
                for &level in &difficulty {stats.game_entries[(level.level()-1) as usize]+=1;}
                stats.game_wins[(difficulty[champion].level()-1) as usize]+=1;
                stats.game_rounds+=game_round;
                progress=[0;4];scores=[0;4];game_round=0;game_index+=1;
            }
        } else {
            stats.stalled_rounds+=1;
            stats.stall_discard_draws+=discard_draws;
            stats.stall_stock_draws+=stock_draws;
            for j in 0..4 {
                if phase[j]<10 {stats.stall_phase_counts[phase[j]]+=1;}
                if !laid[j] {stats.stall_unlaid[(difficulty[j].level()-1) as usize]+=1;}
            }
            if round_turns>=stats.max_round_turns {stats.longest_stall_phase=progress;}
            if exhausted {stats.exhausted_rounds+=1;}
            // An unfinished round cannot advance phases or award points.
            // Reset the partial game so its next round does not masquerade
            // as a valid continuation.
            progress=[0;4];scores=[0;4];game_round=0;game_index+=1;
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

#[test]
fn ten_phase_games_complete_and_track_winners() {
    let s=run_seed(0xFACE2026,120,350);
    assert!(s.games>0,"no complete games: {s:?}");
    assert_eq!(s.game_wins.iter().sum::<u32>(),s.games);
    assert!(s.game_rounds>=s.games*10);
    println!("Full game CPU balance: {s:?}");
    print_balance(&s);
}

#[test]
fn ten_phase_game_results_are_reproducible() {
    let a=run_seed(0xDEADBEEF,100,350);
    let b=run_seed(0xDEADBEEF,100,350);
    assert_eq!(format!("{a:?}"),format!("{b:?}"));
}

#[test]
fn simulation_reports_stalls_and_rotates_starting_seat() {
    let stats=run_seed(0xABCD1234,30,300);
    assert_eq!(stats.completed+stats.stalled_rounds,stats.rounds);
    assert!(stats.exhausted_rounds<=stats.stalled_rounds);
    assert_eq!(stats.seat_wins.iter().sum::<u32>(),stats.completed);
    assert!(stats.max_round_turns<=300);
    println!("CPU diagnostics: {stats:?}");
}

fn print_balance(stats:&Stats) {
    println!("Balance summary: {} rounds, {} completed games, {} stalls ({} exhausted), max {} turns",
        stats.rounds,stats.games,stats.stalled_rounds,stats.exhausted_rounds,stats.max_round_turns);
    for (i,name) in ["Easy","Normal","Hard"].iter().enumerate() {
        let round_rate=if stats.round_entries[i]>0 {100.0*stats.wins[i] as f64/stats.round_entries[i] as f64}else{0.0};
        let game_rate=if stats.game_entries[i]>0 {100.0*stats.game_wins[i] as f64/stats.game_entries[i] as f64}else{0.0};
        println!("{name:7}: rounds {:4}/{:4} ({round_rate:5.1}%), games {:3}/{:3} ({game_rate:5.1}%)",
            stats.wins[i],stats.round_entries[i],stats.game_wins[i],stats.game_entries[i]);
    }
    println!("Seat wins: {:?}; longest stall starting phases: {:?}",stats.seat_wins,stats.longest_stall_phase);
    println!("Stall diagnostics: phases {:?}, unlaid Easy/Normal/Hard {:?}, discard draws {}, stock draws {}",stats.stall_phase_counts,stats.stall_unlaid,stats.stall_discard_draws,stats.stall_stock_draws);
}

#[test]
#[ignore = "long-running balancing sample; run explicitly with cargo test cpu_large_balance_sample -- --ignored --nocapture"]
fn cpu_large_balance_sample() {
    // Five batches rotate all difficulty groups, without imposing fragile win-rate assertions.
    for batch in 0..5 {
        let stats=run_seed(0xBADA5500+batch,300,350);
        println!("Batch {batch}:");
        print_balance(&stats);
    }
}

#[test]
fn participation_counts_match_simulated_rounds() {
    let s=run_seed(0x12345678,16,300);
    assert_eq!(s.round_entries.iter().sum::<u32>(),s.rounds*4);
    assert_eq!(s.game_entries.iter().sum::<u32>(),s.games*4);
    assert!(s.wins.iter().zip(s.round_entries).all(|(&wins,entries)|wins<=entries));
}
