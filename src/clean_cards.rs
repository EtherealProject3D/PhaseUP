//! Shared palette and monogram geometry for gameplay and the UI Designer.
use crate::card_model::Kind;
pub(crate) const CREAM: [u8; 3] = [250, 244, 226];
pub(crate) const NAVY: [u8; 3] = [16, 33, 52];
pub(crate) const TEAL: [u8; 3] = [30, 193, 164];
pub(crate) const P_BOWL: [[f32; 2]; 7] = [
    [0.40, 0.24],
    [0.54, 0.24],
    [0.61, 0.27],
    [0.63, 0.32],
    [0.61, 0.37],
    [0.55, 0.40],
    [0.48, 0.40],
];
pub(crate) const P_STEM: [[f32; 2]; 2] = [[0.36, 0.28], [0.36, 0.46]];
pub(crate) fn face(kind: Kind) -> [u8; 3] {
    match kind {
        Kind::Number(_, 0) => [197, 43, 51],
        Kind::Number(_, 1) => [25, 125, 193],
        Kind::Number(_, 2) => [239, 181, 48],
        Kind::Number(_, _) => [29, 151, 102],
        Kind::Wild => [113, 72, 162],
        Kind::Skip => [78, 97, 119],
    }
}
pub(crate) fn ink(kind: Kind) -> [u8; 3] {
    if matches!(kind, Kind::Number(_, 2)) {
        NAVY
    } else {
        CREAM
    }
}
pub(crate) fn mark_point(point: [f32; 2], mirrored: bool, compact: bool) -> [f32; 2] {
    let [x, y] = if mirrored {
        [1. - point[0], 1. - point[1]]
    } else {
        point
    };
    if compact {
        [0.5 + (x - 0.5) * 0.6, 0.25 + (y - 0.5) * 0.43]
    } else {
        [x, y]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monogram_is_a_two_way_mark_and_stays_inside_the_card() {
        for p in P_BOWL.into_iter().chain(P_STEM) {
            let a = mark_point(p, false, false);
            let b = mark_point(p, true, false);
            assert!((a[0] + b[0] - 1.).abs() < 0.0001 && (a[1] + b[1] - 1.).abs() < 0.0001);
            for compact in [false, true] {
                for mirrored in [false, true] {
                    assert!(mark_point(p, mirrored, compact)
                        .iter()
                        .all(|v| *v > 0. && *v < 1.));
                }
            }
        }
    }
}
