use macroquad::prelude::get_time;
use crate::*;

impl Game {
    pub(super) fn cpu_action(&mut self) {
        if self.stage!=Stage::Cpu {return;}
        if self.turn>3 {
            self.turn=0;self.stage=Stage::Draw;self.turn_count+=1;
            self.cpu_skip=None;
            self.drawn_card=None;self.message.clear();return;
        }
        let i=self.turn;
        if self.cpu_skip==Some(i) {
            self.log_move(format!("{} was skipped",self.cpu_name(i)));
            self.cpu_skip=None;
            self.turn+=1;self.cpu_step=0;return;
        }
        match self.cpu_step {
            0 => {
                let use_discard=self.discard.last().is_some_and(|c|c.kind!=Kind::Skip && matches!(c.kind,Kind::Wild));
                let card=if use_discard {self.discard.pop()}else{self.draw_deck()};
                if let Some(c)=card {
                    self.players[i].hand.push(c);
                    if self.cpu_speed!=0 {
                        let duration=match self.cpu_speed {1=>0.30,3=>1.05,_=>0.60};
                        self.cpu_draw_flight=Some(CpuDrawFlight {
                            player:i,from_discard:use_discard,card_id:c.id,
                            started:get_time(),duration
                        });
                    }
                    self.log_move(format!("{} drew from {}",self.cpu_name(i),if use_discard {"discard"}else{"deck"}));
                }
                self.cpu_step=1;
            }
            1 => {
                if !self.players[i].laid && self.players[i].phase<PHASES.len() {
                    let needed:usize=PHASES[self.players[i].phase].iter().sum();
                    if needed<=self.players[i].hand.len() {
                        let hand=self.players[i].hand.clone();
                        fn search(hand:&[Card],needed:usize,start:usize,idx:&mut Vec<usize>,phase:usize)->Option<Vec<Vec<Card>>>{
                            if idx.len()==needed {
                                let subset:Vec<Card>=idx.iter().map(|j|hand[*j]).collect();
                                return partition_phase(&subset,phase);
                            }
                            for j in start..hand.len() {
                                idx.push(j);
                                if let Some(g)=search(hand,needed,j+1,idx,phase){return Some(g);}
                                idx.pop();
                            }
                            None
                        }
                        if let Some(mut groups)=search(&hand,needed,0,&mut Vec::new(),self.players[i].phase) {
                            order_table_groups(&mut groups,self.players[i].phase);
                            let ids:Vec<u32>=groups.iter().flatten().map(|c|c.id).collect();
                            Self::remove_ids(&mut self.players[i].hand,&ids);
                            if self.cpu_speed!=0 {
                                let per_card=match self.cpu_speed {1=>0.12,3=>0.34,_=>0.20};
                                let cards=groups.iter().enumerate().flat_map(|(g,group)|group.iter().copied().enumerate().map(move |(k,card)|(card,g,k))).collect();
                                self.cpu_phase_flight=Some(CpuPhaseFlight{player:i,cards,started:get_time(),per_card});
                            }
                            self.players[i].table=groups;self.players[i].laid=true;
                            self.log_move(format!("{} laid phase {}",self.cpu_name(i),self.players[i].phase+1));
                        }
                    }
                }
                self.cpu_step=2;
            }
            2 => {
                let mut hit=None;
                if self.players[i].laid {
                    'cards: for (j,&card) in self.players[i].hand.iter().enumerate() {
                        for target in 0..4 {
                            if self.players[target].laid {
                                for group in 0..self.players[target].table.len() {
                                    if can_hit(&self.players[target].table[group],card,self.players[target].phase.min(9),group) {
                                        hit=Some((j,target,group,card));break 'cards;
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((j,target,group,card))=hit {
                    self.players[i].hand.remove(j);
                    self.players[target].table[group].push(card);
                    if requirement(self.players[target].phase.min(9),group)=="run" {
                        order_run(&mut self.players[target].table[group]);
                    }
                    self.log_move(format!("{} hit {} with {}",self.cpu_name(i),self.cpu_name(target),card_text(card.kind)));
                } else {self.cpu_step=3;}
            }
            _ => {
                if self.players[i].hand.is_empty(){self.end_round(i);return;}
                // Prefer discards that cannot be added to an opponent's laid phase.
                // Retain the existing hand-building heuristic as the tie-breaker.
                let idx=(0..self.players[i].hand.len()).max_by_key(|&j| {
                    let card=self.players[i].hand[j];
                    let mut penalty=0;
                    for target in 0..self.players.len() {
                        if target==i || !self.players[target].laid {continue;}
                        let hittable=self.players[target].table.iter().enumerate().any(|(group_idx,group)| {
                            can_hit(group,card,self.players[target].phase.min(9),group_idx)
                        });
                        if hittable {
                            // The human player gets the largest defensive priority.
                            penalty+=if target==0 {400}else{250};
                        }
                    }
                    cpu_discard_priority(&self.players[i].hand,j,self.players[i].phase)-penalty
                }).unwrap();
                let count_before=self.players[i].hand.len();
                let card=self.players[i].hand.remove(idx);
                if self.cpu_speed!=0 {
                    let duration=match self.cpu_speed {1=>0.30,3=>1.05,_=>0.60};
                    self.cpu_discard_flight=Some(CpuDiscardFlight{
                        card,player:i,origin_index:idx,origin_count:count_before,
                        started:get_time(),duration
                    });
                }
                self.discard.push(card);
                self.log_move(format!("{} discarded {}",self.cpu_name(i),card_text(card.kind)));
                if self.players[i].hand.is_empty(){self.end_round(i);return;}
                if card.kind==Kind::Skip {
                    let target=(i+1)%4;
                    self.log_move(format!("{} skipped {}",self.cpu_name(i),self.cpu_name(target)));
                    if target==0 {
                        // The last CPU skipped the human. Start the next CPU
                        // cycle without exposing a playable human turn.
                        self.turn_count+=1;
                        self.turn=1;
                        self.cpu_skip=None;
                        self.cpu_draw_flight=None;
                        self.cpu_next_at=get_time()+0.75;
                    } else {
                        self.cpu_skip=Some(target);
                        self.turn=target;
                    }
                }else{
                    self.turn+=1;
                }
                self.cpu_step=0;
            }
        }
    }

    pub(super) fn update_cpu(&mut self) {
        if self.cpu_draw_flight.as_ref().is_some_and(|a|get_time()-a.started>=a.duration) {self.cpu_draw_flight=None;}
        if self.cpu_discard_flight.as_ref().is_some_and(|a|get_time()-a.started>=a.duration) {self.cpu_discard_flight=None;}
        if self.cpu_phase_flight.as_ref().is_some_and(|a|get_time()-a.started>=a.duration()) {self.cpu_phase_flight=None;}
        if self.stage!=Stage::Cpu{return;}
        if self.cpu_speed==0 {
            if get_time()>=self.cpu_next_at {
                while self.stage==Stage::Cpu {self.cpu_action();}
            }
        }else if get_time()>=self.cpu_next_at {
            let previous_turn=self.turn;
            self.cpu_action();
            let delay=match self.cpu_speed {1=>0.35,3=>1.35,_=>0.75};
            // Give each CPU's final discard time to land before the next
            // player begins. Keep the normal pacing between actions intact.
            let handoff_delay=if self.turn!=previous_turn && self.stage==Stage::Cpu {
                match self.cpu_speed {1=>0.48,3=>1.40,_=>0.78}
            }else{delay};
            let phase_delay=self.cpu_phase_flight.as_ref()
                .map(|flight|(flight.started+flight.duration()-get_time()+0.10).max(0.0))
                .unwrap_or(0.0);
            self.cpu_next_at=get_time()+handoff_delay.max(phase_delay);
        }
    }
}
