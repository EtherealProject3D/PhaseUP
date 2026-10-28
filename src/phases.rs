pub(super) const PHASES: [&[usize]; 10] = [
    &[3,3], &[3,4], &[4,4], &[7], &[8], &[9], &[4,4], &[7], &[5,2], &[5,3],
];
pub(super) fn requirement(phase: usize, group: usize) -> &'static str {
    match (phase,group) {
        (0,_)|(6,_)|(8,_)|(9,_) => "set",
        (1,0)|(2,0) => "set",
        (1,1)|(2,1)|(3,_)|(4,_)|(5,_) => "run",
        (7,_) => "color",
        _ => "set"
    }
}
// A group heading describes its original requirement, even after extra hits.
pub(super) fn requirement_label(phase:usize,group:usize)->String {
    let phase=phase.min(PHASES.len()-1);
    let group=group.min(PHASES[phase].len()-1);
    format!("{} OF {}",requirement(phase,group).to_uppercase(),PHASES[phase][group])
}

#[cfg(test)] mod requirement_heading_tests {
    use super::*;
    #[test] fn headings_describe_each_phase_requirement(){
        for (phase,expected) in [
            (0,vec!["SET OF 3","SET OF 3"]),(1,vec!["SET OF 3","RUN OF 4"]),
            (2,vec!["SET OF 4","RUN OF 4"]),(3,vec!["RUN OF 7"]),
            (4,vec!["RUN OF 8"]),(5,vec!["RUN OF 9"]),
            (6,vec!["SET OF 4","SET OF 4"]),(7,vec!["COLOR OF 7"]),
            (8,vec!["SET OF 5","SET OF 2"]),(9,vec!["SET OF 5","SET OF 3"]),
        ] {for (group,text) in expected.iter().enumerate(){assert_eq!(requirement_label(phase,group),*text);}}
    }
}
pub(super) fn phase_name(p: usize) -> &'static str {
    const N: [&str;10] = ["2 sets of 3","set of 3 + run of 4","set of 4 + run of 4","run of 7","run of 8","run of 9","2 sets of 4","7 of one color","set of 5 + set of 2","set of 5 + set of 3"];
    N[p.min(9)]
}
 
use super::{Card, Kind};

pub(super) fn valid_group(cards: &[Card], kind: &str) -> bool {
    if cards.is_empty() || cards.iter().any(|c| c.kind==Kind::Skip) { return false; }
    let normals: Vec<(u8,u8)> = cards.iter().filter_map(|c| if let Kind::Number(n,col)=c.kind {Some((n,col))} else {None}).collect();
    if normals.is_empty() { return false; } // require at least one natural card
    match kind {
        "set" => normals.iter().all(|(n,_)| *n==normals[0].0),
        "color" => normals.iter().all(|(_,c)| *c==normals[0].1),
        "run" => {
            let mut nums: Vec<u8> = normals.iter().map(|(n,_)|*n).collect(); nums.sort_unstable();
            if nums.windows(2).any(|w|w[0]==w[1]) { return false; }
            let length=cards.len() as u8;
            (1..=13-length).any(|start|nums.iter().all(|n|*n>=start && *n<start+length))
        }, _=>false
    }
}
// Arrange a laid run numerically, placing wilds into missing-number slots.
pub(super) fn order_run(cards:&mut Vec<Card>) {
    let len=cards.len();
    if len==0 || len>12{return;}
    let numbers:Vec<u8>=cards.iter().filter_map(|c|if let Kind::Number(n,_)=c.kind{Some(n)}else{None}).collect();
    if numbers.is_empty(){return;}
    let Some(start)=(1..=13-len as u8).find(|start|numbers.iter().all(|n|*n>=*start&&(*n as usize)<(*start as usize+len)))else{return;};
    let mut ordered=Vec::with_capacity(len);
    let mut wilds=cards.iter().copied().filter(|c|c.kind==Kind::Wild);
    for value in start..start+len as u8 {
        if let Some(card)=cards.iter().find(|c|matches!(c.kind,Kind::Number(n,_) if n==value)).copied(){ordered.push(card);}
        else if let Some(wild)=wilds.next(){ordered.push(wild);}
    }
    if ordered.len()==len {*cards=ordered;}
}
pub(super) fn order_table_groups(groups:&mut [Vec<Card>],phase:usize){
    for (i,group) in groups.iter_mut().enumerate(){if requirement(phase,i)=="run"{order_run(group);}}
}
pub(super) fn partition_phase(cards: &[Card], phase: usize) -> Option<Vec<Vec<Card>>> {
    let sizes=PHASES[phase]; if cards.len()!=sizes.iter().sum::<usize>() {return None;}
    fn recurse(cards: &[Card], phase: usize, g: usize) -> Option<Vec<Vec<Card>>> {
        if g==PHASES[phase].len() {return if cards.is_empty(){Some(vec![])}else{None};}
        let n=cards.len(); let count=PHASES[phase][g];
        if count>n {return None;}
        // Enumerate subsets (at most 18 cards for a phase; the full hand is normally 10).
        fn choose(cards:&[Card], phase:usize,g:usize,start:usize,need:usize,indices:&mut Vec<usize>) -> Option<Vec<Vec<Card>>> {
            if need==0 {
                let mut chosen=Vec::new(); let mut rest=Vec::new();
                for (i,c) in cards.iter().enumerate() {if indices.contains(&i){chosen.push(*c)}else{rest.push(*c)}}
                if valid_group(&chosen,requirement(phase,g)) {if let Some(mut tail)=recurse(&rest,phase,g+1){let mut result=vec![chosen];result.append(&mut tail);return Some(result);}}
                return None;
            }
            if start+need>cards.len(){return None;}
            for i in start..=cards.len()-need {indices.push(i);if let Some(x)=choose(cards,phase,g,i+1,need-1,indices){return Some(x)}indices.pop();}
            None
        }
        choose(cards,phase,g,0,count,&mut Vec::new())
    }
    recurse(cards,phase,0)
}
// Human players may lay larger valid groups than the phase minimum.
pub(super) fn partition_phase_flexible(cards:&[Card],phase:usize)->Option<Vec<Vec<Card>>>{
    let minimum:usize=PHASES[phase].iter().sum();
    if cards.len()<minimum{return None;}
    fn recurse(cards:&[Card],phase:usize,group:usize)->Option<Vec<Vec<Card>>>{
        if group==PHASES[phase].len(){return if cards.is_empty(){Some(Vec::new())}else{None};}
        let remaining_min:usize=PHASES[phase][group+1..].iter().sum();
        if cards.len()<PHASES[phase][group]+remaining_min{return None;}
        fn choose(cards:&[Card],phase:usize,group:usize,start:usize,need:usize,indices:&mut Vec<usize>)->Option<Vec<Vec<Card>>>{
            if need==0{
                let mut chosen=Vec::new();let mut rest=Vec::new();
                for (i,c) in cards.iter().enumerate(){if indices.contains(&i){chosen.push(*c)}else{rest.push(*c)}}
                if valid_group(&chosen,requirement(phase,group)){
                    if let Some(mut tail)=recurse(&rest,phase,group+1){
                        let mut result=vec![chosen];result.append(&mut tail);return Some(result);
                    }
                }
                return None;
            }
            if start+need>cards.len(){return None;}
            for i in start..=cards.len()-need{
                indices.push(i);
                if let Some(result)=choose(cards,phase,group,i+1,need-1,indices){return Some(result);}
                indices.pop();
            }
            None
        }
        for count in PHASES[phase][group]..=cards.len()-remaining_min {
            if let Some(groups)=choose(cards,phase,group,0,count,&mut Vec::new()){return Some(groups);}
        }
        None
    }
    recurse(cards,phase,0)
}
pub(super) fn can_hit(group: &[Card], card: Card, phase:usize, group_idx:usize) -> bool {
    if card.kind==Kind::Skip {return false;}
    let mut c=group.to_vec();c.push(card);
    valid_group(&c,requirement(phase,group_idx))
}
