//! Flights use the same saved boxes and row geometry as the mobile board.
use crate::*;
use layout::{designer as d, Rect};
fn fly(card: Option<Card>, from: Rect, to: Rect, t: f32) {
    let t = t.clamp(0., 1.);
    let e = t * t * (3. - 2. * t);
    let r = Rect {
        x: from.x + (to.x - from.x) * e,
        y: from.y + (to.y - from.y) * e - 40. * 4. * t * (1. - t),
        w: from.w + (to.w - from.w) * e,
        h: from.h + (to.h - from.h) * e,
    };
    if let Some(c) = card {
        let _ = card_ui(c, r.x, r.y, r.w, r.h, false);
    } else {
        cpu_card_back(r.x, r.y, r.w, r.h);
    }
}
fn pile(discard: bool) -> Rect {
    let r = d::element(if discard { "discard" } else { "draw" }).unwrap();
    let h = r.h.min(r.w / 0.68);
    let w = h * 0.68;
    Rect {
        x: r.x + (r.w - w) / 2.,
        y: r.y + (r.h - h) / 2.,
        w,
        h,
    }
}
pub(super) fn draw(g: &Game) {
    let now = get_time();
    if let Some(f) = &g.human_draw_flight {
        let mut cards: Vec<_> = g.players[0]
            .hand
            .iter()
            .copied()
            .filter(|c| !g.staged.iter().any(|v| v.contains(&c.id)))
            .collect();
        cards.sort_by_key(|c| sort_key(*c, g.sort));
        if let Some(i) = cards.iter().position(|c| c.id == f.card_id) {
            if let Some(to) = d::hand_card(cards.len(), i) {
                fly(
                    Some(cards[i]),
                    pile(f.from_discard),
                    to,
                    ((now - f.started) / f.duration) as f32,
                );
            }
        }
    }
    if let Some(f) = &g.human_discard_flight {
        if let Some(r) = d::hand_card(g.players[0].hand.len() + 1, 0) {
            fly(
                Some(f.card),
                Rect {
                    x: f.from_x,
                    y: f.from_y,
                    ..r
                },
                pile(true),
                ((now - f.started) / f.duration) as f32,
            );
        }
    }
    if let Some(f) = &g.cpu_draw_flight {
        let p = &g.players[f.player];
        if let Some(to) = d::cpu_hand(f.player, p.hand.len().saturating_sub(1), p.hand.len()) {
            fly(
                if f.from_discard {
                    p.hand.iter().find(|c| c.id == f.card_id).copied()
                } else {
                    None
                },
                pile(f.from_discard),
                to,
                ((now - f.started) / f.duration) as f32,
            );
        }
    }
    if let Some(f) = &g.cpu_discard_flight {
        if let Some(from) = d::cpu_hand(f.player, f.origin_index, f.origin_count) {
            fly(
                Some(f.card),
                from,
                pile(true),
                ((now - f.started) / f.duration) as f32,
            );
        }
    }
    if let Some(f) = &g.cpu_hit_flight {
        let p = &g.players[f.target];
        if let Some(cards) = p.table.get(f.group) {
            let index = cards.iter().position(|c| c.id == f.card.id).unwrap_or(0);
            if let (Some(from), Some(to)) = (
                d::cpu_hand(f.player, f.origin_index, f.origin_count),
                d::laid_card(f.target, f.group, p.table.len(), index, cards.len()),
            ) {
                fly(
                    Some(f.card),
                    from,
                    to,
                    ((now - f.started) / f.duration) as f32,
                );
            }
        }
    }
    if let Some(f) = &g.human_phase_flight {
        let elapsed = now - f.started;
        if elapsed >= 0. && elapsed < f.duration() {
            let index = (elapsed / f.per_card) as usize;
            if let Some(&(card, group, slot, x, y)) = f.cards.get(index) {
                let p = &g.players[0];
                if let Some(to) = d::laid_card(0, group, p.table.len(), slot, p.table[group].len())
                {
                    let source =
                        layout::staging_card(group, slot, p.table.len(), p.table[group].len());
                    fly(
                        Some(card),
                        Rect { x, y, ..source },
                        to,
                        ((elapsed - index as f64 * f.per_card) / f.per_card) as f32,
                    );
                }
            }
        }
    }
    if let Some(f) = &g.cpu_phase_flight {
        let elapsed = now - f.started;
        if elapsed >= 0. && elapsed < f.duration() {
            let index = (elapsed / f.per_card) as usize;
            if let Some(&(card, group, slot)) = f.cards.get(index) {
                let p = &g.players[f.player];
                if let (Some(from), Some(to)) = (
                    d::cpu_hand(f.player, 0, p.hand.len() + f.cards.len()),
                    d::laid_card(f.player, group, p.table.len(), slot, p.table[group].len()),
                ) {
                    fly(
                        Some(card),
                        from,
                        to,
                        ((elapsed - index as f64 * f.per_card) / f.per_card) as f32,
                    );
                }
            }
        }
    }
}
