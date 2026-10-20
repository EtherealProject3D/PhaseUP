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
pub(super) fn phase_name(p: usize) -> &'static str {
    const N: [&str;10] = ["2 sets of 3","set of 3 + run of 4","set of 4 + run of 4","run of 7","run of 8","run of 9","2 sets of 4","7 of one color","set of 5 + set of 2","set of 5 + set of 3"];
    N[p.min(9)]
}
