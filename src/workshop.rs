//! Card fixtures for the isolated Full Board Preview; never applied to the live game.
use crate::*;

#[derive(Clone,Copy)]
pub(super) enum PreviewTarget { Hand, Stage(usize), Laid(usize) }
fn next_id(g:&Game)->u32 {
    g.players.iter().flat_map(|p|p.hand.iter().chain(p.table.iter().flatten()))
        .chain(g.deck.iter()).chain(g.discard.iter()).map(|c|c.id).max().unwrap_or(60000).max(60000)+1
}
pub(super) fn target_cards(g:&Game,player:usize,target:PreviewTarget)->Vec<Card>{
    match target {
        PreviewTarget::Hand=>g.players[player].hand.clone(),
        PreviewTarget::Stage(group)=>g.staged[group].iter().filter_map(|id|g.players[0].hand.iter().find(|c|c.id==*id).copied()).collect(),
        PreviewTarget::Laid(group)=>g.players[player].table.get(group).cloned().unwrap_or_default(),
    }
}
pub(super) fn target_available(g:&Game,player:usize,target:PreviewTarget)->bool {
    match target {
        PreviewTarget::Hand=>true,
        PreviewTarget::Stage(group)=>player==0 && group<PHASES[g.players[0].phase.min(9)].len(),
        PreviewTarget::Laid(group)=>group<PHASES[g.players[player].phase.min(9)].len(),
    }
}
fn remove_id(g:&mut Game,player:usize,target:PreviewTarget,id:u32){
    match target {
        PreviewTarget::Hand|PreviewTarget::Stage(_)=>{
            g.players[player].hand.retain(|c|c.id!=id);
            if player==0 {g.unstage_card(id);g.selected.retain(|v|*v!=id);}
        },
        PreviewTarget::Laid(group)=>{
            if let Some(cards)=g.players[player].table.get_mut(group){cards.retain(|c|c.id!=id);}
        },
    }
    g.drag=None;g.staged_drag=None;g.pile_drag=None;
}
pub(super) fn remove_card(g:&mut Game,player:usize,target:PreviewTarget,kind:Kind)->bool {
    if !target_available(g,player,target){return false;}
    if let Some(card)=target_cards(g,player,target).iter().rev().find(|c|c.kind==kind).copied(){
        remove_id(g,player,target,card.id);true
    }else{false}
}
pub(super) fn clear_target(g:&mut Game,player:usize,target:PreviewTarget){
    for card in target_cards(g,player,target){remove_id(g,player,target,card.id);}
}
pub(super) fn add_card(g:&mut Game,player:usize,target:PreviewTarget,kind:Kind)->bool {
    if !target_available(g,player,target) || target_cards(g,player,target).len()>=40{return false;}
    let card=Card{id:next_id(g),kind};
    match target {
        PreviewTarget::Hand=>g.players[player].hand.push(card),
        PreviewTarget::Stage(group)=>{
            g.stage=Stage::Play;g.turn=0;g.players[0].laid=false;g.players[0].table.clear();
            g.players[0].hand.push(card);g.stage_card(card.id,group);
        },
        PreviewTarget::Laid(group)=>{
            let p=&mut g.players[player];p.laid=true;
            while p.table.len()<=group {p.table.push(Vec::new());}
            p.table[group].push(card);
            if requirement(p.phase.min(9),group)=="run" && valid_group(&p.table[group],"run") {order_run(&mut p.table[group]);}
        },
    }
    true
}
pub(super) fn choose_phase(g:&mut Game,player:usize,phase:usize){
    if player==0 {
        for group in 0..2 {clear_target(g,0,PreviewTarget::Stage(group));}
        g.stage=Stage::Play;g.turn=0;
    }
    let p=&mut g.players[player];p.phase=phase.min(9);p.laid=false;p.table.clear();
}
pub(super) fn fill_staged_phase(g:&mut Game,rank:u8,color:u8){
    for group in 0..2 {clear_target(g,0,PreviewTarget::Stage(group));}
    let phase=g.players[0].phase.min(9);
    for (group,&count) in PHASES[phase].iter().enumerate(){
        let run=requirement(phase,group)=="run";
        let start=rank.clamp(1,12).min((13-count) as u8);
        for slot in 0..count {
            let n=if run {start+slot as u8}else{(rank.clamp(1,12)-1+group as u8)%12+1};
            add_card(g,0,PreviewTarget::Stage(group),Kind::Number(n,color.min(3)));
        }
    }
}
