// Core gameplay operations shared by human and CPU turns.
// Keep rendering, input handling and animation logic outside this module.
use crate::*;

impl Game {
    pub(super) fn deal(&mut self) {
        self.move_log.clear();
        self.deck=make_deck();self.discard.clear();self.selected.clear();self.staged=[vec![],vec![]];self.drag=None;self.staged_drag=None;self.pile_drag=None;self.drawn_card=None;self.human_draw_flight=None;self.cpu_discard_flight=None;self.human_discard_flight=None;self.cpu_draw_flight=None;self.cpu_phase_flight=None;self.cpu_hit_flight=None;self.turn=0;self.stage=Stage::Draw;self.cpu_step=0;self.cpu_skip=None;self.turn_count=1;self.round+=1;
        for p in &mut self.players {p.hand.clear();p.laid=false;p.table.clear();}
        for _ in 0..10 {for i in 0..4 {let c=self.deck.pop().unwrap();self.players[i].hand.push(c);}}
        self.discard.push(self.deck.pop().unwrap());
        self.message=String::new(); // Turn guidance is displayed in the banner above the hand.
    }

    pub(super) fn draw_deck(&mut self)->Option<Card> {
        if self.deck.is_empty() && self.discard.len()>1 {
            let top=self.discard.pop().unwrap();
            self.deck.append(&mut self.discard);
            shuffle(&mut self.deck);
            self.discard.push(top);
        }
        self.deck.pop()
    }

    pub(super) fn remove_ids(hand:&mut Vec<Card>, ids:&[u32])->Vec<Card> {
        let mut out=Vec::new();
        let mut i=0;
        while i<hand.len() {
            if ids.contains(&hand[i].id) {out.push(hand.remove(i));}
            else {i+=1;}
        }
        out
    }

    pub(super) fn end_round(&mut self,winner:usize) {
        for p in &mut self.players {
            p.score+=p.hand.iter().map(|c|card_points(*c)).sum::<u32>();
            if p.laid {p.phase+=1;}
        }
        self.turn=winner;
        self.log_move(format!("{} went out!",self.cpu_name(winner)));
        if self.players.iter().any(|p|p.phase>=10) {
            self.stage=Stage::GameOver;
            self.message=format!("Game complete! {} finished all 10 phases",self.cpu_name(winner));
        } else {
            self.stage=Stage::RoundOver;
            self.message=format!("{} went out! Scores updated. Continue for next round.",self.cpu_name(winner));
        }
    }
}
