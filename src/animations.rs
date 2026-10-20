use crate::*;

pub(super) struct CpuDrawFlight { pub(super) player:usize, pub(super) from_discard:bool, pub(super) card_id:u32, pub(super) started:f64, pub(super) duration:f64 }

pub(super) struct CpuDiscardFlight { pub(super) card:Card, pub(super) player:usize, pub(super) origin_index:usize, pub(super) origin_count:usize, pub(super) started:f64, pub(super) duration:f64 }

pub(super) struct CpuPhaseFlight {
    pub(super) player:usize,
    pub(super) cards:Vec<(Card,usize,usize)>,
    pub(super) started:f64,
    pub(super) per_card:f64,
}
impl CpuPhaseFlight {
    pub(super) fn duration(&self)->f64 {self.cards.len() as f64*self.per_card}
    pub(super) fn visible(&self,group:usize,index:usize,now:f64)->bool {
        self.cards.iter().position(|(_,g,k)|*g==group&&*k==index)
            .is_none_or(|n|now-self.started>=(n+1) as f64*self.per_card)
    }
}

pub(super) struct CpuHitFlight {
    pub(super) card:Card,
    pub(super) player:usize,
    pub(super) target:usize,
    pub(super) group:usize,
    pub(super) origin_index:usize,
    pub(super) origin_count:usize,
    pub(super) started:f64,
    pub(super) duration:f64,
}

pub(super) struct HumanPhaseFlight {
    pub(super) cards:Vec<(Card,usize,usize,f32,f32)>,
    pub(super) started:f64,
    pub(super) per_card:f64,
}
impl HumanPhaseFlight {
    pub(super) fn duration(&self)->f64 {self.cards.len() as f64*self.per_card}
    pub(super) fn visible(&self,card_id:u32,now:f64)->bool {
        self.cards.iter().position(|(card,_,_,_,_)|card.id==card_id)
            .is_none_or(|n|now-self.started>=(n+1) as f64*self.per_card)
    }
}

pub(super) struct HumanDrawFlight { pub(super) card_id:u32, pub(super) from_discard:bool, pub(super) started:f64 }
pub(super) struct HumanDiscardFlight { pub(super) card:Card, pub(super) from_x:f32, pub(super) from_y:f32, pub(super) started:f64 }

pub(super) fn draw_card_flights(g:&Game,sw:f32,sh:f32) {
    // Reveal CPU phase cards in sequence, flying from their former hand area.
    if let Some(flight)=&g.cpu_phase_flight {
        let elapsed=get_time()-flight.started;
        if elapsed>=0.0 && elapsed<flight.duration() {
            let index=(elapsed/flight.per_card) as usize;
            if let Some((card,group,slot))=flight.cards.get(index).copied() {
                let p=&g.players[flight.player];
                let count=p.table.iter().map(|g|g.len()).sum::<usize>()+p.hand.len();
                let origin_x=28.0+(flight.player-1) as f32*(sw-50.0)/3.0;
                let step=if count>1 {column_hand_width(count,65.0)/(count-1) as f32}else{0.0};
                let start_x=origin_x+(count.saturating_sub(1) as f32*step)/2.0;
                let start_y=119.0;
                let base_x=24.0+(flight.player-1) as f32*((sw-50.0)/3.0);
                let width=if p.table.len()==1 {((sw-50.0)/3.0-8.0).max(95.0)}else{305.0};
                let dest_x=base_x+group as f32*312.0;
                let len=p.table[group].len();
                let spacing=if len>1 {((width-12.0-55.0).max(0.0)/(len-1) as f32).min(56.0)}else{0.0};
                let end_x=dest_x+6.0+slot as f32*spacing;
                let end_y=207.0+35.0+12.0+26.0;
                let t=((elapsed-index as f64*flight.per_card)/flight.per_card).clamp(0.0,1.0) as f32;
                let eased=t*t*(3.0-2.0*t);
                let x=start_x+(end_x-start_x)*eased;
                let y=start_y+(end_y-start_y)*eased-30.0*(4.0*t*(1.0-t));
                let _=card_ui(card,x,y,55.0+10.0*(1.0-t),84.0+3.0*(1.0-t),false);
            }
        }
    }
    // CPU hit: move the played card from its former hand slot to the table.
    if let Some(flight)=&g.cpu_hit_flight {
        let t=((get_time()-flight.started)/flight.duration).clamp(0.0,1.0) as f32;
        let eased=t*t*(3.0-2.0*t);
        let cpu_x=28.0+(flight.player-1) as f32*(sw-50.0)/3.0;
        let step=if flight.origin_count>1 {column_hand_width(flight.origin_count,65.0)/(flight.origin_count-1) as f32}else{0.0};
        let from_x=cpu_x+flight.origin_index as f32*step;
        let from_y=119.0;
        let p=&g.players[flight.target];
        if let Some(cards)=p.table.get(flight.group) {
            let x=if flight.target==0 {sw/2.0-400.0}else{24.0+(flight.target-1) as f32*((sw-50.0)/3.0)};
            let y=if flight.target==0 {if game_canvas_height()<1150.0 {game_canvas_height()-522.0}else{game_canvas_height()-544.0}}else{if game_canvas_height()<1150.0 {224.0}else{242.0}};
            let width=if flight.target==0 {if p.table.len()==1 {800.0}else{397.0}}
                else if p.table.len()==1 {((sw-50.0)/3.0-8.0).max(95.0)}else{305.0};
            let x=x+flight.group as f32*if flight.target==0 {353.0}else{252.0};
            let slot=cards.iter().position(|c|c.id==flight.card.id).unwrap_or(cards.len().saturating_sub(1));
            let spacing=if cards.len()>1 {((width-12.0-55.0).max(0.0)/(cards.len()-1) as f32).min(56.0)}else{0.0};
            let to_x=x+6.0+slot as f32*spacing;
            let to_y=y+12.0+26.0;
            let px=from_x+(to_x-from_x)*eased;
            let py=from_y+(to_y-from_y)*eased-45.0*(4.0*t*(1.0-t));
            let _=card_ui(flight.card,px,py,65.0-10.0*eased,87.0-3.0*eased,false);
        }
    }
    // Human phase: move cards from their staging slots to the laid phase.
    if let Some(flight)=&g.human_phase_flight {
        let elapsed=get_time()-flight.started;
        if elapsed>=0.0 && elapsed<flight.duration() {
            let index=(elapsed/flight.per_card) as usize;
            if let Some((card,group,slot,from_x,from_y))=flight.cards.get(index).copied() {
                let p=&g.players[0];
                if let Some(cards)=p.table.get(group) {
                    let width=if p.table.len()==1 {800.0}else{397.0};
                    let step=if cards.len()>1 {((width-12.0-55.0)/(cards.len()-1) as f32).min(56.0)}else{0.0};
                    let to_x=sw/2.0-400.0+group as f32*403.0+6.0+slot as f32*step;
                    let to_y=(if game_canvas_height()<1150.0 {game_canvas_height()-522.0}else{game_canvas_height()-544.0})+12.0+26.0;
                    let t=((elapsed-index as f64*flight.per_card)/flight.per_card).clamp(0.0,1.0) as f32;
                    let eased=t*t*(3.0-2.0*t);
                    let x=from_x+(to_x-from_x)*eased;
                    let y=from_y+(to_y-from_y)*eased-35.0*(4.0*t*(1.0-t));
                    let _=card_ui(card,x,y,50.0+5.0*eased,76.0+8.0*eased,false);
                }
            }
        }
    }
    // Human draw: show the face-up card flying to its sorted hand slot.
    if let Some(flight)=&g.human_draw_flight {
        if let Some(card)=g.players[0].hand.iter().find(|c|c.id==flight.card_id).copied() {
            let mut sorted:Vec<Card>=g.players[0].hand.iter().copied()
                .filter(|c|!g.staged.iter().any(|bucket|bucket.contains(&c.id))).collect();
            sorted.sort_by_key(|c|sort_key(*c,g.sort));
            let n=sorted.len();
            let w=((sw-48.0)/(n.max(1) as f32)).min(75.0);
            let spacing=if n>1 {((sw-48.0-w)/(n-1) as f32).min(81.0)}else{0.0};
            let left=(sw-(spacing*(n.saturating_sub(1) as f32)+w))/2.0;
            let index=sorted.iter().position(|c|c.id==flight.card_id).unwrap_or(n.saturating_sub(1));
            let end_x=left+index as f32*spacing;
            let end_y=sh-166.0;
            let start_x=if flight.from_discard {sw/2.0+88.0}else{sw/2.0-200.0};
            let start_y=if game_canvas_height()<1150.0 {404.0}else{485.0};
            let t=((get_time()-flight.started)/0.60).clamp(0.0,1.0) as f32;
            let eased=t*t*(3.0-2.0*t);
            let x=start_x+(end_x-start_x)*eased;
            let y=start_y+(end_y-start_y)*eased-50.0*(4.0*t*(1.0-t));
            let _=card_ui(card,x,y,112.0+(w-112.0)*eased,150.0+(105.0-150.0)*eased,false);
        }
    }
    // Animate the arriving card above the board, without duplicating its hand back.
    if let Some(flight)=&g.cpu_draw_flight {
        let t=((get_time()-flight.started)/flight.duration).clamp(0.0,1.0) as f32;
        let eased=t*t*(3.0-2.0*t);
        let start_x=if flight.from_discard {1920.0/2.0+88.0}else{1920.0/2.0-200.0};
        let start_y=if game_canvas_height()<1150.0 {404.0}else{485.0};
        let cpu_x=28.0+(flight.player-1) as f32*(1920.0-50.0)/3.0;
        let count=g.players[flight.player].hand.len();
        let step=if count>1 {column_hand_width(count,65.0)/(count-1) as f32}else{0.0};
        let index=g.players[flight.player].hand.iter().position(|c|c.id==flight.card_id).unwrap_or(count.saturating_sub(1));
        let end_x=cpu_x+index as f32*step;
        let end_y=119.0;
        let x=start_x+(end_x-start_x)*eased;
        let y=start_y+(end_y-start_y)*eased-50.0*(4.0*t*(1.0-t));
        if flight.from_discard {
            if let Some(card)=g.players[flight.player].hand.iter().find(|c|c.id==flight.card_id).copied(){
                let _=card_ui(card,x,y,112.0+(65.0-112.0)*eased,150.0+(87.0-150.0)*eased,false);
            }
        }else{
            cpu_card_back(x,y,112.0+(65.0-112.0)*eased,150.0+(87.0-150.0)*eased);
        }
    }
    // A button-triggered human discard flies from its sorted hand slot.
    if let Some(flight)=&g.human_discard_flight {
        let t=((get_time()-flight.started)/0.60).clamp(0.0,1.0) as f32;
        let eased=t*t*(3.0-2.0*t);
        let x=flight.from_x+(sw/2.0+88.0-flight.from_x)*eased;
        let y=flight.from_y+((if game_canvas_height()<1150.0 {404.0}else{485.0})-flight.from_y)*eased-50.0*(4.0*t*(1.0-t));
        let _=card_ui(flight.card,x,y,75.0+(112.0-75.0)*eased,105.0+(150.0-105.0)*eased,false);
    }
    // CPU discards fly face-up from the former hand slot to the discard pile.
    if let Some(flight)=&g.cpu_discard_flight {
        let t=((get_time()-flight.started)/flight.duration).clamp(0.0,1.0) as f32;
        let eased=t*t*(3.0-2.0*t);
        let cpu_x=28.0+(flight.player-1) as f32*(sw-50.0)/3.0;
        let step=if flight.origin_count>1 {
            column_hand_width(flight.origin_count,65.0)/(flight.origin_count-1) as f32
        }else{0.0};
        let from_x=cpu_x+flight.origin_index as f32*step;
        let from_y=119.0;
        let to_x=sw/2.0+88.0;
        let to_y=if game_canvas_height()<1150.0 {404.0}else{485.0};
        let x=from_x+(to_x-from_x)*eased;
        let y=from_y+(to_y-from_y)*eased-50.0*(4.0*t*(1.0-t));
        let _=card_ui(flight.card,x,y,65.0+(112.0-65.0)*eased,87.0+(150.0-87.0)*eased,false);
    }
}

pub(super) fn expire_human_flights(g:&mut Game) {
    if g.human_phase_flight.as_ref().is_some_and(|a|get_time()-a.started>=a.duration()) {g.human_phase_flight=None;}
    if g.human_draw_flight.as_ref().is_some_and(|a|get_time()-a.started>=0.60) {g.human_draw_flight=None;}
    if g.human_discard_flight.as_ref().is_some_and(|a|get_time()-a.started>=0.60) {g.human_discard_flight=None;}
}
