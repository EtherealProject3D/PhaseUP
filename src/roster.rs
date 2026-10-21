// Fixed CPU identities. Difficulty and personality are independent dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Difficulty { Novice, Easy, Normal, Hard, Expert }
impl Difficulty {
    pub(super) fn next(self)->Self {match self {Self::Novice=>Self::Easy,Self::Easy=>Self::Normal,Self::Normal=>Self::Hard,Self::Hard=>Self::Expert,Self::Expert=>Self::Novice}}
    pub(super) fn key(self)->&'static str {match self {Self::Novice=>"novice",Self::Easy=>"easy",Self::Normal=>"normal",Self::Hard=>"hard",Self::Expert=>"expert"}}
    pub(super) fn from_key(s:&str)->Self {match s {"novice"=>Self::Novice,"easy"=>Self::Easy,"hard"=>Self::Hard,"expert"=>Self::Expert,_=>Self::Normal}}
    pub(super) fn level(self)->i32 {match self {Self::Novice=>0,Self::Easy=>1,Self::Normal=>2,Self::Hard=>3,Self::Expert=>4}}
    pub(super) fn name(self)->&'static str {match self {Self::Novice=>"Novice",Self::Easy=>"Easy",Self::Normal=>"Normal",Self::Hard=>"Hard",Self::Expert=>"Expert"}}
}
#[derive(Clone, Copy, Debug)]
pub(super) enum PlayStyle { Casual, Aggressive, Conservative, Balanced, Opportunist, Defensive, Analytical, Strategic, Adaptive }
impl PlayStyle {
    pub(super) fn name(self)->&'static str {match self {Self::Casual=>"Casual",Self::Aggressive=>"Aggressive",Self::Conservative=>"Conservative",Self::Balanced=>"Balanced",Self::Opportunist=>"Opportunist",Self::Defensive=>"Defensive",Self::Analytical=>"Analytical",Self::Strategic=>"Strategic",Self::Adaptive=>"Adaptive"}}
}
#[derive(Clone, Copy, Debug)]
pub(super) struct CpuIdentity {
    pub(super) name: &'static str,
    pub(super) difficulty: Difficulty,
    pub(super) style: PlayStyle,
}
pub(super) const ROSTER: [CpuIdentity;9] = [
    CpuIdentity{name:"Mike",difficulty:Difficulty::Novice,style:PlayStyle::Casual},
    CpuIdentity{name:"Sarah",difficulty:Difficulty::Easy,style:PlayStyle::Aggressive},
    CpuIdentity{name:"Dave",difficulty:Difficulty::Easy,style:PlayStyle::Conservative},
    CpuIdentity{name:"Lisa",difficulty:Difficulty::Normal,style:PlayStyle::Balanced},
    CpuIdentity{name:"Chris",difficulty:Difficulty::Normal,style:PlayStyle::Opportunist},
    CpuIdentity{name:"Kevin",difficulty:Difficulty::Hard,style:PlayStyle::Defensive},
    CpuIdentity{name:"Rachel",difficulty:Difficulty::Hard,style:PlayStyle::Analytical},
    CpuIdentity{name:"John",difficulty:Difficulty::Expert,style:PlayStyle::Strategic},
    CpuIdentity{name:"Alex",difficulty:Difficulty::Expert,style:PlayStyle::Adaptive},
];
// Select three unique opponents with no more than one difficulty step between
// any two. The chosen target is a preference, not a guaranteed exact level.
pub(super) fn select_opponents(target:Difficulty)->[CpuIdentity;3] {
    let mut pool:Vec<CpuIdentity>=ROSTER.iter().copied()
        .filter(|cpu|(cpu.difficulty.level()-target.level()).abs()<=1).collect();
    // Shuffle candidates using the game's already-seeded Macroquad RNG.
    for i in (1..pool.len()).rev() {
        let j=macroquad::rand::gen_range(0,(i+1) as i32) as usize;
        pool.swap(i,j);
    }
    // Ensure all three opponents are mutually within one difficulty step.
    let mut best:Option<[CpuIdentity;3]>=None;
    let mut best_score=i32::MAX;
    for a in 0..pool.len() {
        for b in a+1..pool.len() {
            for c in b+1..pool.len() {
                let three=[pool[a],pool[b],pool[c]];
                let lo=three.iter().map(|p|p.difficulty.level()).min().unwrap();
                let hi=three.iter().map(|p|p.difficulty.level()).max().unwrap();
                if hi-lo>1 {continue;}
                let score=three.iter().map(|p|(p.difficulty.level()-target.level()).abs()).sum();
                if score<best_score {best_score=score;best=Some(three);}
            }
        }
    }
    // All levels have at least three eligible candidates.
    best.expect("CPU roster must support three opponents per difficulty")
}
