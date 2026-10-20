use crate::*;

struct CpuDrawFlight { player:usize, from_discard:bool, card_id:u32, started:f64, duration:f64 }

struct CpuDiscardFlight { card:Card, player:usize, origin_index:usize, origin_count:usize, started:f64, duration:f64 }

struct HumanDrawFlight { card_id:u32, from_discard:bool, started:f64 }
struct HumanDiscardFlight { card:Card, from_x:f32, from_y:f32, started:f64 }

pub(super) fn draw_card_flights(g:&Game,sw:f32,sh:f32) {
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
            let start_x=if flight.from_discard {sw*0.56-40.0}else{sw*0.39-55.0};
            let start_y=535.0-36.0;
            let t=((get_time()-flight.started)/0.60).clamp(0.0,1.0) as f32;
            let eased=t*t*(3.0-2.0*t);
            let x=start_x+(end_x-start_x)*eased;
            let y=start_y+(end_y-start_y)*eased-50.0*(4.0*t*(1.0-t));
            let _=card_ui(card,x,y,w,105.0,false);
        }
    }
    // Animate the arriving card above the board, without duplicating its hand back.
    if let Some(flight)=&g.cpu_draw_flight {
        let t=((get_time()-flight.started)/flight.duration).clamp(0.0,1.0) as f32;
        let eased=t*t*(3.0-2.0*t);
        let start_x=if flight.from_discard {1600.0*0.56-40.0}else{1600.0*0.39-55.0};
        let start_y=535.0-36.0;
        let cpu_x=28.0+(flight.player-1) as f32*(1600.0-50.0)/3.0;
        let count=g.players[flight.player].hand.len();
        let step=if count>1 {column_hand_width(count,65.0)/(count-1) as f32}else{0.0};
        let index=g.players[flight.player].hand.iter().position(|c|c.id==flight.card_id).unwrap_or(count.saturating_sub(1));
        let end_x=cpu_x+index as f32*step;
        let end_y=119.0;
        let x=start_x+(end_x-start_x)*eased;
        let y=start_y+(end_y-start_y)*eased-50.0*(4.0*t*(1.0-t));
        if flight.from_discard {
            if let Some(card)=g.players[flight.player].hand.iter().find(|c|c.id==flight.card_id).copied(){
                let _=card_ui(card,x,y,65.0,87.0,false);
            }
        }else{
            cpu_card_back(x,y,65.0,87.0);
        }
    }
    // A button-triggered human discard flies from its sorted hand slot.
    if let Some(flight)=&g.human_discard_flight {
        let t=((get_time()-flight.started)/0.60).clamp(0.0,1.0) as f32;
        let eased=t*t*(3.0-2.0*t);
        let x=flight.from_x+(sw*0.56-40.0-flight.from_x)*eased;
        let y=flight.from_y+(535.0-36.0-flight.from_y)*eased-50.0*(4.0*t*(1.0-t));
        let _=card_ui(flight.card,x,y,75.0+7.0*t,105.0+5.0*t,false);
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
        let to_x=sw*0.56-40.0;
        let to_y=535.0-36.0;
        let x=from_x+(to_x-from_x)*eased;
        let y=from_y+(to_y-from_y)*eased-50.0*(4.0*t*(1.0-t));
        let _=card_ui(flight.card,x,y,65.0+17.0*t,87.0+23.0*t,false);
    }
}

pub(super) fn expire_human_flights(g:&mut Game) {
    if g.human_draw_flight.as_ref().is_some_and(|a|get_time()-a.started>=0.60) {g.human_draw_flight=None;}
    if g.human_discard_flight.as_ref().is_some_and(|a|get_time()-a.started>=0.60) {g.human_discard_flight=None;}
}
