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
    for target in [Difficulty::Easy,Difficulty::Normal,Difficulty::Hard] {
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


#[test]
fn recycling_deck_keeps_top_discard_and_preserves_card_ids() {
    let mut g=Game::new();
    let top=number(9001,9,0);
    let buried=number(9002,3,1);
    g.deck.clear();
    g.discard=vec![buried,top];
    let drawn=g.draw_deck().expect("buried discard should recycle into draw pile");
    assert_eq!(drawn.id,buried.id);
    assert_eq!(g.discard.len(),1);
    assert_eq!(g.discard[0].id,top.id);
    assert!(g.deck.is_empty());
}

#[test]
fn dealing_new_round_resets_transient_state_but_preserves_progress() {
    let mut g=Game::new();
    let first_round=g.round;
    g.players[0].phase=2;
    g.players[0].score=45;
    g.players[0].laid=true;
    g.selected.push(g.players[0].hand[0].id);
    g.staged[0].push(g.players[0].hand[0].id);
    g.turn=2;
    g.stage=Stage::RoundOver;
    g.deal();
    assert_eq!(g.round,first_round+1);
    assert_eq!(g.turn,0);
    assert_eq!(g.stage,Stage::Draw);
    assert_eq!(g.players[0].phase,2);
    assert_eq!(g.players[0].score,45);
    assert!(!g.players[0].laid);
    assert!(g.selected.is_empty());
    assert!(g.staged.iter().all(|bucket|bucket.is_empty()));
    assert!(g.players.iter().all(|p|p.hand.len()==10 && p.table.is_empty()));
    assert_eq!(g.discard.len(),1);
    assert_eq!(g.deck.len(),67);
}


#[test]
fn player_cannot_draw_twice_in_one_turn() {
    let mut g=Game::new();
    g.draw(false,false);
    assert_eq!(g.stage,Stage::Play);
    let hand=g.players[0].hand.len();
    let deck=g.deck.len();
    g.draw(false,false);
    assert_eq!(g.players[0].hand.len(),hand);
    assert_eq!(g.deck.len(),deck);
}

#[test]
fn player_cannot_discard_before_drawing() {
    let mut g=Game::new();
    g.selected=vec![g.players[0].hand[0].id];
    let hand=g.players[0].hand.len();
    let discard=g.discard.len();
    g.discard_selected();
    assert_eq!(g.stage,Stage::Draw);
    assert_eq!(g.players[0].hand.len(),hand);
    assert_eq!(g.discard.len(),discard);
}

#[test]
fn player_actions_do_nothing_during_cpu_turn() {
    let mut g=Game::new();
    g.stage=Stage::Cpu;
    g.turn=1;
    let id=g.players[0].hand[0].id;
    g.selected=vec![id];
    let hand=g.players[0].hand.len();
    let deck=g.deck.len();
    let discard=g.discard.len();
    g.draw(false,false);
    g.lay();
    g.hit(0,0);
    g.discard_selected();
    assert_eq!(g.turn,1);
    assert_eq!(g.stage,Stage::Cpu);
    assert_eq!(g.players[0].hand.len(),hand);
    assert_eq!(g.deck.len(),deck);
    assert_eq!(g.discard.len(),discard);
    assert!(!g.players[0].laid);
}

#[test]
fn player_cannot_lay_same_phase_twice() {
    let mut g=Game::new();
    g.stage=Stage::Play;
    g.turn=0;
    g.players[0].phase=0;
    g.players[0].hand=[set(8000,3,3),set(8010,6,3)].concat();
    g.selected=g.players[0].hand.iter().map(|c|c.id).collect();
    g.lay();
    assert!(g.players[0].laid);
    let table=g.players[0].table.clone();
    let hand=g.players[0].hand.len();
    g.lay();
    assert_eq!(g.players[0].hand.len(),hand);
    assert_eq!(g.players[0].table.len(),table.len());
    assert!(g.players[0].laid);
}

#[test]
fn cpu_draws_wild_but_never_skip_from_discard() {
    let hand=vec![number(1,4,0),number(2,4,1)];
    assert!(cpu_should_take_discard(&hand,wild(3),0,false,&[]));
    assert!(!cpu_should_take_discard(&hand,skip(4),0,false,&[]));
}

#[test]
fn cpu_prefers_number_that_supports_a_set() {
    let hand=vec![number(1,7,0),number(2,7,1),number(3,2,0),number(4,11,2)];
    assert!(cpu_should_take_discard(&hand,number(5,7,2),0,false,&[]));
}

#[test]
fn cpu_rejects_unhelpful_number_when_hand_is_strong() {
    let hand=vec![number(1,5,0),number(2,5,1),number(3,5,2),number(4,5,3)];
    assert!(!cpu_should_take_discard(&hand,number(5,12,1),0,false,&[]));
}

#[test]
fn cpu_with_laid_phase_takes_hittable_discard() {
    let table=vec![set(10,8,3),set(20,3,3)];
    let hand=vec![number(1,1,0)];
    assert!(cpu_should_take_discard(&hand,number(2,8,0),0,true,&table));
    assert!(!cpu_should_take_discard(&hand,number(3,9,0),0,true,&table));
}

#[test]
fn cpu_phase_aware_set_retention() {
    let hand=vec![number(1,7,0),number(2,7,1),number(3,11,2)];
    assert!(cpu_discard_priority(&hand,0,0)<cpu_discard_priority(&hand,2,0));
}

#[test]
fn cpu_phase_aware_run_retention() {
    let hand=vec![number(1,4,0),number(2,5,1),number(3,12,2)];
    assert!(cpu_discard_priority(&hand,0,3)<cpu_discard_priority(&hand,2,3));
}

#[test]
fn cpu_phase_aware_color_retention() {
    let hand=vec![number(1,2,0),number(2,9,0),number(3,11,2)];
    assert!(cpu_discard_priority(&hand,0,7)<cpu_discard_priority(&hand,2,7));
}

#[test]
fn cpu_difficulty_changes_discard_decisions() {
    let hand=vec![number(1,7,0),number(2,7,1),number(3,12,2)];
    assert!(cpu_discard_priority_for_level(&hand,0,0,Difficulty::Easy)
        <cpu_discard_priority_for_level(&hand,0,0,Difficulty::Novice));
    assert!(cpu_discard_priority_for_level(&hand,0,0,Difficulty::Normal)
        <cpu_discard_priority_for_level(&hand,0,0,Difficulty::Novice));
}

#[test]
fn cpu_difficulty_controls_defense() {
    assert_eq!(cpu_defense_weight(Difficulty::Novice),0);
    assert_eq!(cpu_defense_weight(Difficulty::Easy),0);
    assert!(cpu_defense_weight(Difficulty::Hard)>cpu_defense_weight(Difficulty::Normal));
    assert!(cpu_defense_weight(Difficulty::Expert)>cpu_defense_weight(Difficulty::Hard));
}

#[test]
fn novice_cpu_avoids_number_discard_pile_but_takes_wild() {
    let hand=vec![number(1,7,0),number(2,7,1)];
    assert!(!cpu_should_take_discard_for_level(&hand,number(3,7,2),0,false,&[],Difficulty::Novice));
    assert!(cpu_should_take_discard_for_level(&hand,wild(4),0,false,&[],Difficulty::Novice));
    assert!(!cpu_should_take_discard_for_level(&hand,skip(5),0,false,&[],Difficulty::Expert));
}

#[test]
fn cpu_difficulty_selector_cycles_three_levels() {
    let mut level=Difficulty::Easy;
    for expected in [Difficulty::Normal,Difficulty::Hard,Difficulty::Easy] {
        level=level.next();
        assert_eq!(level,expected);
    }
}

#[test]
fn cpu_difficulty_storage_keys_round_trip() {
    for level in [Difficulty::Easy,Difficulty::Normal,Difficulty::Hard] {
        assert_eq!(Difficulty::from_key(level.key()),level);
    }
    assert_eq!(Difficulty::from_key("unknown"),Difficulty::Normal);
    assert_eq!(Difficulty::from_key("novice"),Difficulty::Easy);
    assert_eq!(Difficulty::from_key("expert"),Difficulty::Hard);
}

#[test]
fn hard_cpu_saves_skip_until_next_player_is_threatening() {
    assert!(cpu_skip_discard_adjustment(Difficulty::Hard,true,2)>0);
    assert!(cpu_skip_discard_adjustment(Difficulty::Hard,true,8)<0);
    assert!(cpu_skip_discard_adjustment(Difficulty::Hard,false,2)<0);
}

#[test]
fn expert_cpu_reacts_to_imminent_finish_more_strongly() {
    assert!(cpu_skip_discard_adjustment(Difficulty::Expert,true,1)
        >cpu_skip_discard_adjustment(Difficulty::Hard,true,1));
    assert!(cpu_skip_discard_adjustment(Difficulty::Expert,false,2)<0);
    assert!(cpu_skip_discard_adjustment(Difficulty::Expert,false,8)<0);
}

#[test]
fn lower_difficulties_keep_original_skip_timing() {
    for level in [Difficulty::Novice,Difficulty::Easy,Difficulty::Normal] {
        assert_eq!(cpu_skip_discard_adjustment(level,true,1),0);
        assert_eq!(cpu_skip_discard_adjustment(level,false,10),0);
    }
}

#[test]
fn advanced_cpu_conserves_wild_when_natural_phase_exists() {
    let hand=vec![
        number(1,7,0),number(2,7,1),number(3,7,2),
        number(4,8,0),number(5,8,1),wild(6),number(7,8,2),
    ];
    let normal=cpu_choose_phase(&hand,0,Difficulty::Normal).unwrap();
    let hard=cpu_choose_phase(&hand,0,Difficulty::Hard).unwrap();
    let expert=cpu_choose_phase(&hand,0,Difficulty::Expert).unwrap();
    assert_eq!(normal.iter().flatten().filter(|c|c.kind==Kind::Wild).count(),1);
    assert_eq!(hard.iter().flatten().filter(|c|c.kind==Kind::Wild).count(),0);
    assert_eq!(expert.iter().flatten().filter(|c|c.kind==Kind::Wild).count(),0);
}

#[test]
fn advanced_cpu_still_uses_wild_when_required() {
    let hand=vec![
        number(1,7,0),number(2,7,1),wild(3),
        number(4,8,0),number(5,8,1),number(6,8,2),
    ];
    for level in [Difficulty::Hard,Difficulty::Expert] {
        let groups=cpu_choose_phase(&hand,0,level).unwrap();
        assert_eq!(groups.iter().flatten().filter(|c|c.kind==Kind::Wild).count(),1);
    }
}

#[test]
fn cpu_phase_search_rejects_invalid_and_incomplete_hands() {
    let hand=vec![number(1,1,0),number(2,2,0),number(3,3,0)];
    assert!(cpu_choose_phase(&hand,0,Difficulty::Expert).is_none());
    assert!(cpu_choose_phase(&hand,10,Difficulty::Expert).is_none());
}

#[test]
fn advanced_cpu_takes_discard_hittable_on_opponent_table() {
    let own=vec![set(100,7,3),set(110,8,3)];
    let other=vec![set(200,4,3),set(210,5,3)];
    let visible=vec![(0,own.clone()),(0,other)];
    let top=number(300,4,2);
    assert!(!cpu_should_take_visible_hit(top,0,&own,&visible,Difficulty::Easy));
    for level in [Difficulty::Normal,Difficulty::Hard,Difficulty::Expert] {
        assert!(cpu_should_take_visible_hit(top,0,&own,&visible,level));
    }
}

#[test]
fn cpu_rejects_discard_that_hits_no_visible_phase() {
    let own=vec![set(100,7,3),set(110,8,3)];
    let visible=vec![(0,own.clone())];
    for level in [Difficulty::Easy,Difficulty::Normal,Difficulty::Hard,Difficulty::Expert] {
        assert!(!cpu_should_take_visible_hit(number(300,12,2),0,&own,&visible,level));
        assert!(!cpu_should_take_visible_hit(skip(301),0,&own,&visible,level));
    }
}

#[test]
fn cpu_takes_wild_for_completed_phase() {
    let own=vec![set(100,7,3),set(110,8,3)];
    assert!(cpu_should_take_visible_hit(wild(300),0,&own,&[],Difficulty::Expert));
}

#[test]
fn expert_prioritizes_immediate_next_player_threat() {
    let next=cpu_opponent_threat_multiplier(Difficulty::Expert,true,1,true);
    let later=cpu_opponent_threat_multiplier(Difficulty::Expert,true,1,false);
    assert!(next>later);
    assert!(later>cpu_opponent_threat_multiplier(Difficulty::Expert,true,8,false));
}

#[test]
fn difficulty_scales_defensive_opponent_awareness() {
    let normal=cpu_opponent_threat_multiplier(Difficulty::Normal,true,2,true);
    let hard=cpu_opponent_threat_multiplier(Difficulty::Hard,true,2,true);
    let expert=cpu_opponent_threat_multiplier(Difficulty::Expert,true,2,true);
    assert!(normal<hard && hard<expert);
}

#[test]
fn unlaid_opponents_do_not_trigger_small_hand_threat() {
    for level in [Difficulty::Novice,Difficulty::Easy,Difficulty::Normal,Difficulty::Hard,Difficulty::Expert] {
        assert_eq!(cpu_opponent_threat_multiplier(level,false,1,true),1);
        assert_eq!(cpu_opponent_threat_multiplier(level,false,10,false),1);
    }
}

#[test]
fn advanced_cpu_takes_discard_that_completes_a_phase() {
    // Phase 1 needs two sets of three; the discard completes the second set.
    let hand=vec![
        number(1001,3,0),number(1002,3,1),number(1003,3,2),
        number(1004,7,0),number(1005,7,1),
        number(1006,1,0),number(1007,5,1),number(1008,9,2),
        number(1009,11,3),skip(1010),
    ];
    let top=number(1011,7,2);
    for level in [Difficulty::Easy,Difficulty::Hard,Difficulty::Expert] {
        assert!(cpu_should_take_discard_for_level(&hand,top,0,false,&[],level),
            "{level:?} should take a discard completing its phase");
    }
}

#[test]
fn run_window_support_rewards_unique_ranks_not_duplicates() {
    let hand=vec![
        number(2001,3,0),number(2002,4,0),number(2003,5,0),
        number(2004,5,1),number(2005,6,2),number(2006,11,3),
    ];
    assert_eq!(cpu_run_window_support(&hand,0,7),4);
    assert_eq!(cpu_run_window_support(&hand,3,7),4);
    assert!(cpu_run_window_support(&hand,0,7)>cpu_run_window_support(&hand,5,7));
}

#[test]
fn easy_run_phase_does_not_prioritize_unneeded_number_pairs() {
    let hand=vec![
        number(2101,2,0),number(2102,2,1),
        number(2103,5,0),number(2104,6,1),number(2105,7,2),
    ];
    assert!(cpu_discard_priority_for_level(&hand,0,3,Difficulty::Easy)
        >cpu_discard_priority_for_level(&hand,3,3,Difficulty::Easy));
}

#[test]
fn long_run_cpu_takes_missing_rank_in_nearly_complete_run() {
    let hand=vec![
        number(2201,2,0),number(2202,3,1),number(2203,4,2),
        number(2204,5,3),number(2205,6,0),number(2206,8,1),
        number(2207,9,2),number(2208,12,3),skip(2209),number(2210,2,1),
    ];
    let top=number(2211,7,0);
    assert!(cpu_long_run_discard_improves(&hand,top,4));
    for level in [Difficulty::Easy,Difficulty::Normal,Difficulty::Hard] {
        assert!(cpu_should_take_discard_for_level(&hand,top,4,false,&[],level));
    }
}

#[test]
fn long_run_cpu_rejects_weak_or_duplicate_window_override() {
    let hand=vec![
        number(2301,2,0),number(2302,3,1),number(2303,4,2),
        number(2304,5,3),number(2305,6,0),number(2306,10,1),
        number(2307,11,2),number(2308,12,3),skip(2309),number(2310,2,1),
    ];
    assert!(!cpu_long_run_discard_improves(&hand,number(2311,7,0),4));
    assert!(!cpu_long_run_discard_improves(&hand,number(2312,5,2),4));
    assert!(!cpu_long_run_discard_improves(&hand,skip(2313),4));
}

#[test]
fn laying_last_cards_ends_round_without_discard() {
    let mut g=Game::new();
    g.stage=Stage::Play;
    g.players[0].hand=[set(1000,3,3),set(1010,5,3)].concat();
    g.selected=g.players[0].hand.iter().map(|c|c.id).collect();
    let discard=g.discard.len();
    g.lay();
    assert!(g.players[0].hand.is_empty());
    assert_eq!(g.stage,Stage::RoundOver);
    assert_eq!(g.players[0].phase,1);
    assert_eq!(g.discard.len(),discard);
}

#[test]
fn hitting_last_card_ends_round_without_discard() {
    let mut g=Game::new();
    g.stage=Stage::Play;
    g.players[0].laid=true;
    g.players[0].table=vec![set(1000,3,3),set(1010,5,3)];
    g.players[0].hand=vec![number(1020,3,0)];
    g.selected=vec![1020];
    g.hit(0,0);
    assert!(g.players[0].hand.is_empty());
    assert_eq!(g.stage,Stage::RoundOver);
    assert_eq!(g.players[0].phase,1);
}

#[test]
fn cpu_laying_last_cards_ends_round_immediately() {
    let mut g=Game::new();
    g.stage=Stage::Cpu;g.turn=1;g.cpu_step=1;g.cpu_speed=0;
    g.players[1].hand=[set(1000,3,3),set(1010,5,3)].concat();
    g.cpu_action();
    assert!(g.players[1].hand.is_empty());
    assert_eq!(g.stage,Stage::RoundOver);
    assert_eq!(g.turn,1);
}

#[test]
fn cpu_hitting_last_card_ends_round_immediately() {
    let mut g=Game::new();
    g.stage=Stage::Cpu;g.turn=1;g.cpu_step=2;g.cpu_speed=0;
    g.players[1].laid=true;
    g.players[1].table=vec![set(1000,3,3),set(1010,5,3)];
    g.players[1].hand=vec![number(1020,3,0)];
    g.cpu_action();
    assert!(g.players[1].hand.is_empty());
    assert_eq!(g.stage,Stage::RoundOver);
    assert_eq!(g.turn,1);
}

#[test]
fn workshop_nudging_moves_without_resizing_and_resizes_without_recentering() {
    let r=layout::Rect{x:20.,y:30.,w:680.,h:117.};
    let moved=layout::adjusted(r,-1.,10.,false);
    assert_eq!((moved.x,moved.y,moved.w,moved.h),(19.,40.,680.,117.));
    let resized=layout::adjusted(r,5.,10.,true);
    assert_eq!((resized.x,resized.y,resized.w,resized.h),(20.,30.,685.,127.));
    let minimum=layout::adjusted(r,-1000.,-1000.,true);
    assert_eq!((minimum.w,minimum.h),(8.,8.));
}

#[test]
fn workshop_scene_lists_rendered_elements_and_keeps_repeated_labels_distinct() {
    layout::begin_scene();
    let first=layout::scene_key("test_label");
    let second=layout::scene_key("test_label");
    assert_ne!(first,second);
    let r=layout::Rect{x:1.,y:2.,w:40.,h:20.};
    layout::rect(&first,r);layout::rect(&first,r);layout::rect(&second,r);
    layout::rect("test_offset",r);
    layout::end_scene();
    let elements=layout::scene_items();
    assert_eq!(elements.len(),2);
    assert_eq!(elements[0].0,first);
    assert_eq!(elements[1].0,second);
    layout::begin_scene();
    assert_eq!(layout::scene_key("test_label"),first);
    assert!(layout::scene_items().is_empty());
    layout::end_scene();
}

#[test]
fn workshop_controls_activate_without_edit_mode_and_close_cleanly() {
    let mut editor=layout_editor::Editor::new();
    assert!(!layout_editor::editing());
    editor.open();
    assert!(layout_editor::editing());
    editor.open();
    assert!(layout_editor::editing());
    editor.close();
    assert!(!layout_editor::editing());
    assert_eq!(layout_editor::zoom(),1.0);
    assert_eq!(layout_editor::pan(),(0.0,0.0));
}

#[test]
fn workshop_tool_window_stays_within_viewport() {
    let moved=layout_editor::tool_panel_position(vec2(300.,200.));
    assert_eq!((moved.x,moved.y),(300.,200.));
    let top_left=layout_editor::tool_panel_position(vec2(-500.,-500.));
    assert_eq!((top_left.x,top_left.y),(0.,0.));
    let bottom_right=layout_editor::tool_panel_position(vec2(3000.,3000.));
    assert_eq!((bottom_right.x,bottom_right.y),(1450.,580.));
    assert_eq!(bottom_right.x+470.,1920.);
    assert_eq!(bottom_right.y+620.,1200.);
}

#[test]
fn compact_hand_does_not_spread_few_cards_across_a_wide_box() {
    let area=layout::Rect{x:24.,y:1037.,w:1872.,h:105.};
    let (start,width,step)=layout::compact_hand(10,area);
    assert_eq!((width,step),(75.,81.));
    let row_width=width+step*9.;
    assert_eq!(start,(1920.-row_width)/2.);
    let (single,_,single_step)=layout::compact_hand(1,area);
    assert_eq!(single_step,0.);
    assert_eq!(single,24.+(1872.-75.)/2.);
}

#[test]
fn compact_hand_fits_crowded_hands_and_respects_moved_box() {
    for count in [0,1,10,11,30,60] {
        let area=layout::Rect{x:200.,y:900.,w:1000.,h:105.};
        let (start,width,step)=layout::compact_hand(count,area);
        assert!(start.is_finite() && width.is_finite() && step.is_finite());
        assert!(step<=81.);
        assert!(start>=area.x-0.01);
        if count>0 {assert!(start+width+step*(count-1) as f32<=area.x+area.w+0.01);}
    }
}
