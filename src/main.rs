use macroquad::prelude::*;

mod config;
mod ui;
mod util;
mod world;

use config::*;
use ui::*;
use util::*;
use world::*;

#[macroquad::main("EcoSim")]
async fn main() {
    let cpu_threads = num_cpus::get();

    let mut mode = SimMode::Home;

    let mut seed = gen_seed(cpu_threads);
    let mut rng = rng_from_seed(seed);

    let mut layout = Layout::compute(screen_width(), screen_height());

    let mut set = SimSettings::default();
    let mut world = World::new(&mut rng, set);

    let tuning = SimTuning::default();
    let mut hist = StatsHistory::new();
    let mut ui = UiState::new();

    let mut acc = 0.0f32;
    let mut steps = 0u64;
    let mut speed = 1.0f32;

    let mut tracked: Option<TrackTarget> = None;

    loop {
        let expected_w = layout.world_w_px + layout.panel_w;
        if (screen_width() - expected_w).abs() > 0.5 || (screen_height() - layout.world_h_px).abs() > 0.5 {
            layout = Layout::compute(screen_width(), screen_height());
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            speed = (speed * 1.25).min(8.0);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            speed = (speed / 1.25).max(0.10);
        }

        clear_background(BG);

        match mode {
            SimMode::Home => {
                home_input(&mut ui, &mut set);
                home_mouse_input(&layout, &mut ui, &mut set);

                if let Some(s) = home_seed_input(&mut ui) {
                    seed = s;
                    rng = rng_from_seed(seed);
                }

                if is_key_pressed(KeyCode::N) {
                    seed = gen_seed(cpu_threads);
                    rng = rng_from_seed(seed);
                    ui.seed_buf.clear();
                }

                draw_home(&layout, &ui, set, cpu_threads, seed);

                if is_key_pressed(KeyCode::Enter) {
                    world = World::new(&mut rng, set);
                    hist = StatsHistory::new();
                    ui.log.clear();
                    ui.seed_buf.clear();
                    acc = 0.0;
                    steps = 0;
                    tracked = None;
                    mode = SimMode::Running;
                }
            }
            SimMode::Running => {
                handle_pick(&layout, &world, &mut tracked);

                if is_key_pressed(KeyCode::S) {
                    let ok = world.save(SAVE_PATH, set);
                    ui.log_push(if ok { "saved".to_string() } else { "save failed".to_string() });
                }
                if is_key_pressed(KeyCode::L) {
                    if let Some((w, s2)) = World::load(SAVE_PATH) {
                        world = w;
                        set = s2;
                        hist = StatsHistory::new();
                        acc = 0.0;
                        tracked = None;
                        ui.log_push("loaded".to_string());
                    } else {
                        ui.log_push("load failed".to_string());
                    }
                }

                let frame_dt = get_frame_time().min(0.10);
                acc += frame_dt * speed;

                let mut n = 0u32;
                let mut did = false;
                let mut last_d = Deltas {
                    herb_birth: 0,
                    herb_death: 0,
                    pred_birth: 0,
                    pred_death: 0,
                };

                while acc >= tuning.fixed_dt && n < tuning.max_steps_per_frame {
                    last_d = world.step(&mut rng, set, tuning.fixed_dt);
                    steps += 1;
                    acc -= tuning.fixed_dt;
                    n += 1;
                    did = true;
                }

                let alpha = (acc / tuning.fixed_dt).clamp(0.0, 1.0);

                world.draw(&layout, alpha, tracked);

                if did {
                    let c = world.counts();
                    hist.push(steps, c, last_d);
                    tick_events(&mut ui, c, last_d);
                }

                let tinfo = tracked.and_then(|t| world.tracked_info(t));
                draw_panel(&layout, &world, &hist, &ui, mode, steps, seed, speed, set, tinfo);

                if is_key_pressed(KeyCode::Space) {
                    mode = SimMode::Paused;
                }
                if is_key_pressed(KeyCode::R) {
                    world = World::new(&mut rng, set);
                    hist = StatsHistory::new();
                    ui.log.clear();
                    acc = 0.0;
                    steps = 0;
                    tracked = None;
                    ui.log_push("restart".to_string());
                }
                if is_key_pressed(KeyCode::N) {
                    seed = gen_seed(cpu_threads);
                    rng = rng_from_seed(seed);
                    world = World::new(&mut rng, set);
                    hist = StatsHistory::new();
                    ui.log.clear();
                    acc = 0.0;
                    steps = 0;
                    tracked = None;
                    ui.log_push("new seed".to_string());
                }
            }
            SimMode::Paused => {
                handle_pick(&layout, &world, &mut tracked);

                if is_key_pressed(KeyCode::S) {
                    let ok = world.save(SAVE_PATH, set);
                    ui.log_push(if ok { "saved".to_string() } else { "save failed".to_string() });
                }
                if is_key_pressed(KeyCode::L) {
                    if let Some((w, s2)) = World::load(SAVE_PATH) {
                        world = w;
                        set = s2;
                        hist = StatsHistory::new();
                        acc = 0.0;
                        tracked = None;
                        ui.log_push("loaded".to_string());
                    } else {
                        ui.log_push("load failed".to_string());
                    }
                }

                world.draw(&layout, 1.0, tracked);

                let tinfo = tracked.and_then(|t| world.tracked_info(t));
                draw_panel(&layout, &world, &hist, &ui, mode, steps, seed, speed, set, tinfo);

                draw_pause_overlay(&layout);

                if is_key_pressed(KeyCode::Space) {
                    mode = SimMode::Running;
                }
                if is_key_pressed(KeyCode::R) {
                    world = World::new(&mut rng, set);
                    hist = StatsHistory::new();
                    ui.log.clear();
                    acc = 0.0;
                    steps = 0;
                    tracked = None;
                    ui.log_push("restart".to_string());
                    mode = SimMode::Running;
                }
                if is_key_pressed(KeyCode::N) {
                    seed = gen_seed(cpu_threads);
                    rng = rng_from_seed(seed);
                    world = World::new(&mut rng, set);
                    hist = StatsHistory::new();
                    ui.log.clear();
                    acc = 0.0;
                    steps = 0;
                    tracked = None;
                    ui.log_push("new seed".to_string());
                    mode = SimMode::Running;
                }
                if is_key_pressed(KeyCode::Enter) {
                    mode = SimMode::Home;
                }
            }
        }

        next_frame().await;
    }
}

fn handle_pick(layout: &Layout, world: &World, tracked: &mut Option<TrackTarget>) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if mx >= 0.0 && mx < layout.world_w_px && my >= 0.0 && my < layout.world_h_px {
            let wp = vec2(mx / CELL, my / CELL);
            *tracked = world.pick_target(wp);
        }
    }
}

fn tick_events(ui: &mut UiState, c: Counts, d: Deltas) {
    let mut tag = 0u8;

    if c.herbs == 0 {
        tag = 1;
        if ui.last_tag != tag {
            ui.log_push("herb extinction".to_string());
        }
    } else if c.preds == 0 {
        tag = 2;
        if ui.last_tag != tag {
            ui.log_push("pred extinction".to_string());
        }
    } else if c.plants_avg > 0.82 {
        tag = 3;
        if ui.last_tag != tag {
            ui.log_push("plant bloom".to_string());
        }
    } else if c.plants_avg < 0.18 {
        tag = 4;
        if ui.last_tag != tag {
            ui.log_push("plant crash".to_string());
        }
    }

    ui.last_tag = tag;

    let _ = d;
}
