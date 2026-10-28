//! Reproducible captures of the real gameplay renderer, with no settings writes.
use crate::*;

pub(super) async fn run(output: &str) {
    std::fs::create_dir_all(output).expect("create capture directory");
    set_theme(Theme::Modern);
    load_menu_fonts().await;
    for (aspect, source) in [
        (
            "20_9",
            include_str!("../assets/layouts/prototype_mobile_compact_20_9.json"),
        ),
        (
            "19_5_9",
            include_str!("../assets/layouts/prototype_mobile_compact_19_5_9.json"),
        ),
    ] {
        let original: serde_json::Value = serde_json::from_str(source).unwrap();
        for expanded in [false, true] {
            for phase in [0, 3] {
                for laid in [false, true] {
                    let mut doc = original.clone();
                    doc["show_cpu_cards"] = expanded.into();
                    layout::designer::load(Some(&doc));
                    let (w, h) = layout::designer::canvas().unwrap();
                    request_new_screen_size(w * 0.5, h * 0.5);
                    next_frame().await;
                    next_frame().await;
                    let mut game = Game::new();
                    game.stage = Stage::Play;
                    game.turn = 0;
                    workshop::choose_phase(&mut game, 0, phase);
                    workshop::fill_staged_phase(&mut game, 5, 0);
                    if laid {
                        game.players[0].table = game
                            .staged
                            .iter()
                            .take(PHASES[phase].len())
                            .map(|ids| {
                                ids.iter()
                                    .map(|id| {
                                        *game.players[0].hand.iter().find(|c| c.id == *id).unwrap()
                                    })
                                    .collect()
                            })
                            .collect();
                        let ids: Vec<_> = game.staged.iter().flatten().copied().collect();
                        game.players[0].hand.retain(|c| !ids.contains(&c.id));
                        game.players[0].laid = true;
                        game.staged = [vec![], vec![]];
                    }
                    for cpu in 1..=3 {
                        game.players[cpu].laid = true;
                        game.players[cpu].table = vec![
                            vec![
                                Card {
                                    id: 9000 + cpu as u32 * 10,
                                    kind: Kind::Number(5, 0)
                                };
                                3
                            ],
                            vec![
                                Card {
                                    id: 9001 + cpu as u32 * 10,
                                    kind: Kind::Number(8, 1)
                                };
                                3
                            ],
                        ];
                    }
                    game.selected = game.players[0]
                        .hand
                        .iter()
                        .filter(|c| !game.staged.iter().any(|group| group.contains(&c.id)))
                        .take(2)
                        .map(|c| c.id)
                        .collect();
                    draw_game(&mut game, false, true, false, false);
                    let path = std::path::Path::new(output).join(format!(
                        "{aspect}_{}_{}_{}.png",
                        if expanded { "expanded" } else { "collapsed" },
                        if phase == 0 { "double" } else { "single" },
                        if laid { "laid" } else { "staged" }
                    ));
                    get_screen_data().export_png(path.to_str().unwrap());
                    next_frame().await;
                    if !laid {
                        let id = game.staged[0][0];
                        game.unstage_card(id);
                        draw_game(&mut game, false, true, false, false);
                        let building = path.with_file_name(
                            path.file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .replace("_staged.png", "_building.png"),
                        );
                        get_screen_data().export_png(building.to_str().unwrap());
                        next_frame().await;
                    }
                }
            }
        }
    }
}
