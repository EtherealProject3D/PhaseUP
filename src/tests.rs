use crate::*;
use std::collections::HashSet;

fn number(id:u32,n:u8,color:u8)->Card {Card{id,kind:Kind::Number(n,color)}}
fn wild(id:u32)->Card {Card{id,kind:Kind::Wild}}
fn skip(id:u32)->Card {Card{id,kind:Kind::Skip}}
fn set(start:u32,n:u8,count:usize)->Vec<Card> {(0..count).map(|i|number(start+i as u32,n,(i%4) as u8)).collect()}
fn run(start:u32,first:u8,count:usize)->Vec<Card> {(0..count).map(|i|number(start+i as u32,first+i as u8,(i%4) as u8)).collect()}

#[test]
fn deck_has_correct_distribution_and_unique_ids() {
    let deck=make_deck();
    assert_eq!(deck.len(),108);
    let ids:HashSet<u32>=deck.iter().map(|c|c.id).collect();
    assert_eq!(ids.len(),108);
    for color in 0..4 {
        for n in 1..=12 {
            assert_eq!(deck.iter().filter(|c|c.kind==Kind::Number(n,color)).count(),2);
        }
    }
    assert_eq!(deck.iter().filter(|c|c.kind==Kind::Wild).count(),8);
    assert_eq!(deck.iter().filter(|c|c.kind==Kind::Skip).count(),4);
}

#[test]
fn all_ten_phase_definitions_are_consistent() {
    let expected=[(vec![3,3],vec!["set","set"]),(vec![3,4],vec!["set","run"]),
        (vec![4,4],vec!["set","run"]),(vec![7],vec!["run"]),
        (vec![8],vec!["run"]),(vec![9],vec!["run"]),
        (vec![4,4],vec!["set","set"]),(vec![7],vec!["color"]),
        (vec![5,2],vec!["set","set"]),(vec![5,3],vec!["set","set"])];
    for (phase,(sizes,kinds)) in expected.iter().enumerate() {
        assert_eq!(PHASES[phase],sizes.as_slice(),"phase {}",phase+1);
        assert!(!phase_name(phase).is_empty());
        for (group,kind) in kinds.iter().enumerate() {assert_eq!(requirement(phase,group),*kind);}
    }
}

#[test]
fn sets_accept_matching_numbers_and_wilds_but_not_skips() {
    assert!(valid_group(&[number(1,7,0),number(2,7,1),wild(3)],"set"));
    assert!(!valid_group(&[number(1,7,0),number(2,8,1)],"set"));
    assert!(!valid_group(&[wild(1),wild(2),wild(3)],"set"));
    assert!(!valid_group(&[number(1,7,0),skip(2)],"set"));
}

#[test]
fn runs_require_distinct_numbers_within_a_contiguous_window() {
    assert!(valid_group(&run(1,3,7),"run"));
    assert!(valid_group(&[number(1,3,0),wild(2),number(3,5,1),number(4,6,2)],"run"));
    assert!(!valid_group(&[number(1,3,0),number(2,3,1),number(3,4,2)],"run"));
    assert!(!valid_group(&[number(1,1,0),number(2,5,1),number(3,9,2)],"run"));
    assert!(!valid_group(&[number(1,12,0),number(2,1,1)],"run"));
}

#[test]
fn color_groups_ignore_number_but_require_same_color() {
    assert!(valid_group(&[number(1,1,2),number(2,12,2),wild(3)],"color"));
    assert!(!valid_group(&[number(1,1,2),number(2,12,1)],"color"));
    assert!(!valid_group(&[wild(1),wild(2)],"color"));
}

#[test]
fn all_ten_phases_can_be_partitioned() {
    let examples:Vec<Vec<Card>>=vec![
        [set(1,2,3),set(10,8,3)].concat(),
        [set(1,2,3),run(10,4,4)].concat(),
        [set(1,2,4),run(10,4,4)].concat(),
        run(1,2,7),run(1,2,8),run(1,2,9),
        [set(1,2,4),set(10,8,4)].concat(),
        (0..7).map(|i|number(i+1,(i+1) as u8,2)).collect(),
        [set(1,2,5),set(10,8,2)].concat(),
        [set(1,2,5),set(10,8,3)].concat(),
    ];
    for (phase,cards) in examples.iter().enumerate() {
        let groups=partition_phase(cards,phase).unwrap_or_else(||panic!("phase {} rejected",phase+1));
        assert_eq!(groups.len(),PHASES[phase].len());
        for (idx,group) in groups.iter().enumerate() {
            assert_eq!(group.len(),PHASES[phase][idx]);
            assert!(valid_group(group,requirement(phase,idx)));
        }
        assert!(partition_phase(&cards[..cards.len()-1],phase).is_none());
    }
}

#[test]
fn flexible_phase_accepts_extra_valid_cards() {
    let cards=[set(1,2,4),set(10,8,3)].concat();
    assert!(partition_phase(&cards,0).is_none());
    assert!(partition_phase_flexible(&cards,0).is_some());
}

#[test]
fn run_order_places_wild_in_missing_slot() {
    let mut cards=vec![number(1,6,0),wild(2),number(3,4,1),number(4,7,2)];
    order_run(&mut cards);
    assert_eq!(cards.iter().map(|c|c.kind).collect::<Vec<_>>(),
        vec![Kind::Number(4,1),Kind::Wild,Kind::Number(6,0),Kind::Number(7,2)]);
}

#[test]
fn hits_follow_group_rules() {
    let group=set(1,8,3);
    assert!(can_hit(&group,number(4,8,2),0,0));
    assert!(!can_hit(&group,number(4,9,2),0,0));
    assert!(!can_hit(&group,skip(4),0,0));
    let group=run(1,4,4);
    assert!(can_hit(&group,number(5,3,0),1,1));
    assert!(can_hit(&group,number(5,8,0),1,1));
    assert!(!can_hit(&group,number(5,6,0),1,1));
}

#[test]
fn scoring_matches_card_values() {
    assert_eq!(card_points(number(1,1,0)),5);
    assert_eq!(card_points(number(2,9,0)),5);
    assert_eq!(card_points(number(3,10,0)),10);
    assert_eq!(card_points(number(4,12,0)),10);
    assert_eq!(card_points(skip(5)),15);
    assert_eq!(card_points(wild(6)),25);
}

#[test]
fn round_end_scores_hands_and_advances_only_laid_phases() {
    let mut g=Game::new();
    g.players[0].hand=vec![number(1,2,0),wild(2)];
    g.players[1].hand=vec![skip(3)];
    g.players[2].hand=vec![number(4,11,0)];
    g.players[3].hand.clear();
    g.players[0].laid=true;
    g.players[1].laid=false;
    g.players[2].laid=true;
    g.players[3].laid=true;
    g.end_round(3);
    assert_eq!(g.players.iter().map(|p|p.score).collect::<Vec<_>>(),vec![30,15,10,0]);
    assert_eq!(g.players.iter().map(|p|p.phase).collect::<Vec<_>>(),vec![1,0,1,1]);
    assert_eq!(g.stage,Stage::RoundOver);
}

#[test]
fn dealing_starts_with_ten_cards_per_player_and_one_discard() {
    let g=Game::new();
    assert!(g.players.iter().all(|p|p.hand.len()==10));
    assert_eq!(g.discard.len(),1);
    assert_eq!(g.deck.len(),67);
    assert_eq!(g.stage,Stage::Draw);
}

#[test]
fn cpu_matchmaking_uses_unique_names_and_nearby_levels() {
    for target in [Difficulty::Novice,Difficulty::Easy,Difficulty::Normal,Difficulty::Hard,Difficulty::Expert] {
        for _ in 0..20 {
            let opponents=select_opponents(target);
            let names:HashSet<&str>=opponents.iter().map(|p|p.name).collect();
            assert_eq!(names.len(),3);
            let levels:Vec<i32>=opponents.iter().map(|p|p.difficulty.level()).collect();
            assert!(levels.iter().max().unwrap()-levels.iter().min().unwrap()<=1);
            assert!(levels.iter().all(|level|(level-target.level()).abs()<=1));
        }
    }
}
