// Core gameplay operations shared by human and CPU turns.
// Keep rendering, input handling and animation logic outside this module.
use crate::*;

impl Game {
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
