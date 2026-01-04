use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::config::*;
use crate::util::fmt_compact;
use crate::world::{TrackedInfo, World};

pub struct StatsHistory {
    cap: usize,
    steps: VecDeque<u64>,
    plants: VecDeque<f32>,
    herbs: VecDeque<u32>,
    preds: VecDeque<u32>,
    hb: VecDeque<u32>,
    hd: VecDeque<u32>,
    pb: VecDeque<u32>,
    pd: VecDeque<u32>,
    he: VecDeque<f32>,
    pe: VecDeque<f32>,
}

impl StatsHistory {
    pub fn new() -> Self {
        Self {
            cap: 520,
            steps: VecDeque::new(),
            plants: VecDeque::new(),
            herbs: VecDeque::new(),
            preds: VecDeque::new(),
            hb: VecDeque::new(),
            hd: VecDeque::new(),
            pb: VecDeque::new(),
            pd: VecDeque::new(),
            he: VecDeque::new(),
            pe: VecDeque::new(),
        }
    }

    pub fn push(&mut self, s: u64, c: Counts, d: Deltas) {
        self.steps.push_back(s);
        self.plants.push_back(c.plants_avg);
        self.herbs.push_back(c.herbs);
        self.preds.push_back(c.preds);
        self.hb.push_back(d.herb_birth);
        self.hd.push_back(d.herb_death);
        self.pb.push_back(d.pred_birth);
        self.pd.push_back(d.pred_death);
        self.he.push_back(c.herb_e_avg);
        self.pe.push_back(c.pred_e_avg);

        while self.steps.len() > self.cap {
            self.steps.pop_front();
            self.plants.pop_front();
            self.herbs.pop_front();
            self.preds.pop_front();
            self.hb.pop_front();
            self.hd.pop_front();
            self.pb.pop_front();
            self.pd.pop_front();
            self.he.pop_front();
            self.pe.pop_front();
        }
    }

    fn len(&self) -> usize {
        self.steps.len()
    }

    fn max_agents_recent(&self) -> u32 {
        let mut m = 1u32;
        for &v in self.herbs.iter() {
            if v > m {
                m = v;
            }
        }
        for &v in self.preds.iter() {
            if v > m {
                m = v;
            }
        }
        m
    }

    fn max_flow_recent(&self) -> u32 {
        let mut m = 1u32;
        for &v in self.hb.iter() {
            if v > m {
                m = v;
            }
        }
        for &v in self.hd.iter() {
            if v > m {
                m = v;
            }
        }
        for &v in self.pb.iter() {
            if v > m {
                m = v;
            }
        }
        for &v in self.pd.iter() {
            if v > m {
                m = v;
            }
        }
        m
    }

    fn max_energy_recent(&self) -> f32 {
        let mut m = 0.1f32;
        for &v in self.he.iter() {
            if v > m {
                m = v;
            }
        }
        for &v in self.pe.iter() {
            if v > m {
                m = v;
            }
        }
        m
    }
}

pub struct UiState {
    pub sel: usize,
    pub log: VecDeque<String>,
    pub log_cap: usize,
    pub last_tag: u8,
    pub seed_buf: String,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            sel: 0,
            log: VecDeque::new(),
            log_cap: 10,
            last_tag: 0,
            seed_buf: String::new(),
        }
    }

    pub fn log_push(&mut self, s: String) {
        self.log.push_front(s);
        while self.log.len() > self.log_cap {
            self.log.pop_back();
        }
    }
}

pub fn home_input(ui: &mut UiState, set: &mut SimSettings) {
    let n = 9usize;

    if is_key_pressed(KeyCode::Up) {
        ui.sel = (ui.sel + n - 1) % n;
    }
    if is_key_pressed(KeyCode::Down) {
        ui.sel = (ui.sel + 1) % n;
    }

    let dec = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
    let inc = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
    if dec {
        apply_home_adjust(set, ui.sel, -1.0);
    }
    if inc {
        apply_home_adjust(set, ui.sel, 1.0);
    }
}

pub fn home_seed_input(ui: &mut UiState) -> Option<u64> {
    while let Some(ch) = get_char_pressed() {
        if ch.is_ascii_digit() {
            if ui.seed_buf.len() < 20 {
                ui.seed_buf.push(ch);
            }
        }
    }

    if is_key_pressed(KeyCode::Backspace) {
        ui.seed_buf.pop();
    }

    if ui.seed_buf.is_empty() {
        return None;
    }

    ui.seed_buf.parse::<u64>().ok()
}

pub fn home_mouse_input(layout: &Layout, ui: &mut UiState, set: &mut SimSettings) {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }

    let (mx, my) = mouse_position();

    let x = PAD * 2.0;
    let y = layout.world_h_px * 0.30;
    let w = layout.world_w_px - PAD * 4.0;
    let h = layout.world_h_px * 0.60;

    let rows_top = y + 140.0;
    let footer_h = 44.0;
    let rows_h = (h - (rows_top - y) - footer_h).max(0.0);

    let row_h = 34.0;
    let max_rows = (rows_h / row_h).floor() as usize;
    let show_rows = 9usize.min(max_rows);

    if mx < x || mx > x + w || my < rows_top || my > rows_top + show_rows as f32 * row_h {
        return;
    }

    let i = ((my - rows_top) / row_h).floor() as usize;
    if i >= show_rows {
        return;
    }

    ui.sel = i;

    let btn_w = 34.0;
    let btn_h = row_h - 10.0;
    let by = rows_top + i as f32 * row_h + 5.0;

    let minus_x = x + w - 2.0 * btn_w - 18.0;
    let plus_x = x + w - btn_w - 12.0;

    if in_rect(mx, my, minus_x, by, btn_w, btn_h) {
        apply_home_adjust(set, i, -1.0);
    } else if in_rect(mx, my, plus_x, by, btn_w, btn_h) {
        apply_home_adjust(set, i, 1.0);
    }
}

fn in_rect(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn apply_home_adjust(set: &mut SimSettings, idx: usize, dir: f32) {
    match idx {
        0 => set.init_herbs = ((set.init_herbs as i32) + (20.0 * dir) as i32).clamp(0, 12000) as u32,
        1 => set.init_preds = ((set.init_preds as i32) + (2.0 * dir) as i32).clamp(0, 3000) as u32,
        2 => set.plant_grow = ((set.plant_grow as i32) + (1.0 * dir) as i32).clamp(0, 12) as u8,
        3 => set.plant_spread = (set.plant_spread + 0.03 * dir).clamp(0.0, 0.95),
        4 => set.herb_speed = (set.herb_speed + 0.02 * dir).clamp(0.04, 0.70),
        5 => set.pred_speed = (set.pred_speed + 0.02 * dir).clamp(0.04, 0.85),
        6 => set.herb_met = (set.herb_met + 0.002 * dir).clamp(0.001, 0.060),
        7 => set.pred_met = (set.pred_met + 0.002 * dir).clamp(0.001, 0.080),
        8 => set.eat_radius = (set.eat_radius + 0.05 * dir).clamp(0.15, 2.00),
        _ => {}
    }
}

pub fn draw_home(layout: &Layout, ui: &UiState, set: SimSettings, cpu_threads: usize, seed: u64) {
    let cx = layout.world_w_px * 0.5;

    draw_text_center("ecosim", cx, layout.world_h_px * 0.13, 64.0, TXT);

    draw_text_center(
        "tiny ecosystem sim: plants regrow/spread, herbs eat plants, preds hunt herbs",
        cx,
        layout.world_h_px * 0.18,
        20.0,
        SUB,
    );

    draw_text_center("enter: start   esc: quit", cx, layout.world_h_px * 0.22, 20.0, SUB);
    draw_text_center(
        "up/down + left/right (or a/d): edit settings",
        cx,
        layout.world_h_px * 0.25,
        20.0,
        SUB,
    );

    let seed_line = if ui.seed_buf.is_empty() {
        format!("seed: {}  (type digits to override)", seed)
    } else {
        format!("seed: {}  (typing: {})", seed, ui.seed_buf)
    };
    draw_text_center(&seed_line, cx, layout.world_h_px * 0.28, 18.0, SUB);

    let (pl, pc) = perf_label(set, cpu_threads);

    let x = PAD * 2.0;
    let y = layout.world_h_px * 0.30;
    let w = layout.world_w_px - PAD * 4.0;
    let h = layout.world_h_px * 0.60;

    draw_rectangle(x, y, w, h, Color::new(0.06, 0.07, 0.10, 1.0));
    draw_rectangle_lines(x, y, w, h, 2.0, LINE);

    draw_text("intro", x + 18.0, y + 26.0, 20.0, SUB);
    draw_text(
        "click an agent to track it (shows in panel + gets highlighted)",
        x + 18.0,
        y + 52.0,
        18.0,
        SUB,
    );
    draw_text(
        "s: save   l: load   space: pause   r: restart   n: new seed   +/-: speed",
        x + 18.0,
        y + 74.0,
        18.0,
        SUB,
    );

    draw_text("settings", x + 18.0, y + 110.0, 20.0, SUB);

    let rows = [
        format!("init herbs: {}", set.init_herbs),
        format!("init preds: {}", set.init_preds),
        format!("plant grow: {}", set.plant_grow),
        format!("plant spread: {:.2}", set.plant_spread),
        format!("herb speed: {:.2}", set.herb_speed),
        format!("pred speed: {:.2}", set.pred_speed),
        format!("herb metabolism: {:.3}", set.herb_met),
        format!("pred metabolism: {:.3}", set.pred_met),
        format!("eat radius: {:.2}", set.eat_radius),
    ];

    let rows_top = y + 140.0;
    let footer_h = 44.0;
    let rows_h = (h - (rows_top - y) - footer_h).max(0.0);

    let row_h = 34.0;
    let text_sz = 20.0;
    let text_base = rows_top + row_h * 0.72;

    let max_rows = (rows_h / row_h).floor() as usize;
    let show_rows = rows.len().min(max_rows);

    let bx = x + 14.0;
    let bw = w - 28.0;

    let (mx, my) = mouse_position();

    for i in 0..show_rows {
        let ry = rows_top + i as f32 * row_h;

        if i == ui.sel {
            draw_rectangle(bx, ry + 3.0, bw, row_h - 6.0, Color::new(0.12, 0.14, 0.20, 1.0));
            draw_rectangle_lines(bx, ry + 3.0, bw, row_h - 6.0, 2.0, LINE);
        }

        let btn_w = 34.0;
        let btn_h = row_h - 10.0;
        let by = ry + 5.0;

        let minus_x = x + w - 2.0 * btn_w - 18.0;
        let plus_x = x + w - btn_w - 12.0;

        let hover_minus = in_rect(mx, my, minus_x, by, btn_w, btn_h);
        let hover_plus = in_rect(mx, my, plus_x, by, btn_w, btn_h);

        let base_btn = Color::new(0.10, 0.11, 0.16, 1.0);
        let hot_btn = Color::new(0.14, 0.16, 0.24, 1.0);

        draw_rectangle(minus_x, by, btn_w, btn_h, if hover_minus { hot_btn } else { base_btn });
        draw_rectangle_lines(
            minus_x,
            by,
            btn_w,
            btn_h,
            2.0,
            if hover_minus { Color::new(0.30, 0.55, 1.0, 0.90) } else { LINE },
        );

        draw_rectangle(plus_x, by, btn_w, btn_h, if hover_plus { hot_btn } else { base_btn });
        draw_rectangle_lines(
            plus_x,
            by,
            btn_w,
            btn_h,
            2.0,
            if hover_plus { Color::new(0.30, 0.55, 1.0, 0.90) } else { LINE },
        );

        let m1 = measure_text("-", None, 20, 1.0);
        draw_text(
            "-",
            minus_x + btn_w * 0.5 - m1.width * 0.5,
            by + btn_h * 0.72,
            20.0,
            if hover_minus { TXT } else { SUB },
        );

        let m2 = measure_text("+", None, 20, 1.0);
        draw_text(
            "+",
            plus_x + btn_w * 0.5 - m2.width * 0.5,
            by + btn_h * 0.72,
            20.0,
            if hover_plus { TXT } else { SUB },
        );

        draw_text(
            &rows[i],
            x + 22.0,
            text_base + i as f32 * row_h,
            text_sz,
            if i == ui.sel { TXT } else { SUB },
        );
    }

    let perf_y = y + h - 16.0;
    draw_text(&format!("perf: {}", pl), x + 18.0, perf_y, 20.0, pc);

    draw_rectangle(layout.panel_x, 0.0, layout.panel_w, layout.world_h_px, PANEL_BG);
    draw_rectangle_lines(layout.panel_x, 0.0, layout.panel_w, layout.world_h_px, 2.0, LINE);

    let px = layout.panel_x + PAD;
    let mut py = PAD + 16.0;

    draw_text("controls", px, py, 20.0, SUB);
    py += 26.0;
    draw_text("click: track agent", px, py, 18.0, SUB);
    py += 20.0;
    draw_text("space: pause/resume", px, py, 18.0, SUB);
    py += 20.0;
    draw_text("r: restart   n: new seed", px, py, 18.0, SUB);
    py += 20.0;
    draw_text("s: save   l: load", px, py, 18.0, SUB);
    py += 20.0;
    draw_text("+/-: speed", px, py, 18.0, SUB);
    py += 30.0;

    draw_text("legend", px, py, 20.0, SUB);
    py += 26.0;
    draw_text("plants", px, py, 20.0, C_PLANT);
    py += 24.0;
    draw_text("herbivores", px, py, 20.0, C_HERB);
    py += 24.0;
    draw_text("predators", px, py, 20.0, C_PRED);
    py += 30.0;

    draw_text("panel graphs", px, py, 18.0, SUB);
    py += 22.0;
    draw_text("population / flows / plants / energy", px, py, 18.0, SUB);
}

pub fn draw_pause_overlay(layout: &Layout) {
    let w = layout.world_w_px;
    let h = layout.world_h_px;
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.30));
    draw_text_center("paused", w * 0.5, h * 0.18, 48.0, TXT);
    draw_text_center(
        "space: resume   r: restart   n: new seed   enter: home",
        w * 0.5,
        h * 0.24,
        20.0,
        SUB,
    );
}

pub fn draw_panel(
    layout: &Layout,
    world: &World,
    hist: &StatsHistory,
    ui: &UiState,
    mode: SimMode,
    steps: u64,
    seed: u64,
    speed: f32,
    set: SimSettings,
    tracked: Option<TrackedInfo>,
) {
    let x = layout.panel_x;
    let w = layout.panel_w;
    let h = layout.world_h_px;

    draw_rectangle(x, 0.0, w, h, PANEL_BG);
    draw_rectangle_lines(x, 0.0, w, h, 2.0, LINE);

    let mut cy = PAD + 10.0;

    draw_text("stats", x + PAD, cy, 30.0, TXT);
    cy += 34.0;

    let c = world.counts();
    row(x + PAD, &mut cy, "step", &fmt_compact(steps), TXT);
    row(x + PAD, &mut cy, "seed", &format!("{}", seed), SUB);
    row(
        x + PAD,
        &mut cy,
        "mode",
        match mode {
            SimMode::Home => "home",
            SimMode::Running => "run",
            SimMode::Paused => "pause",
        },
        SUB,
    );
    row(x + PAD, &mut cy, "speed", &format!("{:.2}x", speed), SUB);

    cy += 10.0;
    row_color(x + PAD, &mut cy, "plants avg", &format!("{:.3}", c.plants_avg), C_PLANT);
    row_color(x + PAD, &mut cy, "herb", &format!("{}", c.herbs), C_HERB);
    row_color(x + PAD, &mut cy, "pred", &format!("{}", c.preds), C_PRED);

    cy += 12.0;
    draw_text("tracked", x + PAD, cy, 20.0, SUB);
    cy += 26.0;

    if let Some(t) = tracked {
        let k = match t.kind {
            TrackKind::Herb => "herb",
            TrackKind::Pred => "pred",
        };
        draw_text(&format!("{} #{}", k, t.id), x + PAD, cy, 18.0, TXT);
        cy += 20.0;
        draw_text(&format!("energy {:.3}   age {}", t.e, t.age), x + PAD, cy, 18.0, SUB);
        cy += 20.0;
        draw_text(&format!("pos {:.1}, {:.1}", t.p.x, t.p.y), x + PAD, cy, 18.0, SUB);
        cy += 12.0;
    } else {
        draw_text("click an agent to track", x + PAD, cy, 18.0, SUB);
        cy += 12.0;
    }

    cy += 8.0;
    draw_text("event log", x + PAD, cy, 20.0, SUB);
    cy += 24.0;

    let mut ly = cy;
    for s in ui.log.iter() {
        draw_text(s, x + PAD, ly, 18.0, SUB);
        ly += 20.0;
        if ly > 250.0 {
            break;
        }
    }

    let gx = x + PAD;
    let gw = w - PAD * 2.0;

    let gh = 98.0;
    let gap = 10.0;

    let g4y = h - PAD - gh;
    let g3y = g4y - gap - gh;
    let g2y = g3y - gap - gh;
    let g1y = g2y - gap - gh;

    draw_graph_block(gx, g1y, gw, gh, "population", &[("herb", C_HERB), ("pred", C_PRED)]);
    draw_graph_pop(gx, g1y, gw, gh, hist);

    draw_graph_block(
        gx,
        g2y,
        gw,
        gh,
        "flows per tick",
        &[("+h", C_HERB), ("-h", C_HERB), ("+p", C_PRED), ("-p", C_PRED)],
    );
    draw_graph_flows(gx, g2y, gw, gh, hist);

    draw_graph_block(gx, g3y, gw, gh, "plants avg", &[("avg", C_PLANT)]);
    draw_graph_plants(gx, g3y, gw, gh, hist);

    draw_graph_block(gx, g4y, gw, gh, "avg energy", &[("herb", C_HERB), ("pred", C_PRED)]);
    draw_graph_energy(gx, g4y, gw, gh, hist);

    let _ = set;
}

fn perf_label(set: SimSettings, cpu_threads: usize) -> (&'static str, Color) {
    let h = set.init_herbs as f32;
    let p = set.init_preds as f32;

    let pair_cost = h * p;
    let base = (W * H) as f32 * 8.0;
    let est = pair_cost + base;

    let t = cpu_threads.max(1) as f32;
    let smooth = 220000.0 * t;
    let ok = 520000.0 * t;

    if est <= smooth {
        ("smooth", C_OK)
    } else if est <= ok {
        ("ok", C_WARN)
    } else {
        ("heavy", C_BAD)
    }
}

fn draw_graph_block(x: f32, y: f32, w: f32, h: f32, title: &str, legend: &[(&str, Color)]) {
    draw_rectangle(x, y, w, h, Color::new(0.06, 0.07, 0.10, 1.0));
    draw_rectangle_lines(x, y, w, h, 2.0, LINE);

    draw_text(title, x + 10.0, y + 20.0, 18.0, SUB);

    let mut lx = x + w - 10.0;
    for (name, col) in legend.iter().rev() {
        let m = measure_text(name, None, 16, 1.0);
        lx -= m.width;
        draw_text(name, lx, y + 20.0, 16.0, *col);
        lx -= 10.0;
    }
}

fn graph_plot_rect(x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
    let head = 32.0;
    (x, y + head, w, (h - head).max(0.0))
}

fn draw_y_ticks(x: f32, y: f32, w: f32, h: f32, vmin: f32, vmax: f32, fmt: fn(f32) -> String) {
    let tx = x + 6.0;
    for k in 0..=4 {
        let t = k as f32 / 4.0;
        let yy = y + h - t * h;
        draw_line(x, yy, x + w, yy, 1.0, Color::new(LINE.r, LINE.g, LINE.b, 0.55));
        draw_line(x, yy, x + 6.0, yy, 2.0, LINE);
        let v = vmin + (vmax - vmin) * t;
        draw_text(&fmt(v), tx, yy - 2.0, 14.0, SUB);
    }
}

fn fmt_int(v: f32) -> String {
    format!("{}", v.round() as i32)
}

fn fmt_small(v: f32) -> String {
    format!("{:.2}", v)
}

fn draw_graph_pop(x: f32, y: f32, w: f32, h: f32, hist: &StatsHistory) {
    if hist.len() < 2 {
        return;
    }
    let vmax = (hist.max_agents_recent().max(50) as f32) * 1.10;
    let (px, py, pw, ph) = graph_plot_rect(x, y, w, h);
    draw_y_ticks(px, py, pw, ph, 0.0, vmax, fmt_int);

    draw_series_u(
        px,
        py,
        pw,
        ph,
        &hist.herbs,
        |v| map_clamped(v as f32, 0.0, vmax, py + ph, py),
        C_HERB,
    );
    draw_series_u(
        px,
        py,
        pw,
        ph,
        &hist.preds,
        |v| map_clamped(v as f32, 0.0, vmax, py + ph, py),
        C_PRED,
    );
}

fn draw_graph_flows(x: f32, y: f32, w: f32, h: f32, hist: &StatsHistory) {
    if hist.len() < 2 {
        return;
    }
    let vmax = (hist.max_flow_recent().max(5) as f32) * 1.20;
    let (px, py, pw, ph) = graph_plot_rect(x, y, w, h);
    draw_y_ticks(px, py, pw, ph, 0.0, vmax, fmt_int);

    draw_series_u(px, py, pw, ph, &hist.hb, |v| map_clamped(v as f32, 0.0, vmax, py + ph, py), C_HERB);
    draw_series_u_dim(px, py, pw, ph, &hist.hd, |v| map_clamped(v as f32, 0.0, vmax, py + ph, py), C_HERB);
    draw_series_u(px, py, pw, ph, &hist.pb, |v| map_clamped(v as f32, 0.0, vmax, py + ph, py), C_PRED);
    draw_series_u_dim(px, py, pw, ph, &hist.pd, |v| map_clamped(v as f32, 0.0, vmax, py + ph, py), C_PRED);
}

fn draw_graph_plants(x: f32, y: f32, w: f32, h: f32, hist: &StatsHistory) {
    if hist.len() < 2 {
        return;
    }
    let (px, py, pw, ph) = graph_plot_rect(x, y, w, h);
    draw_y_ticks(px, py, pw, ph, 0.0, 1.0, fmt_small);
    draw_series_f(px, py, pw, ph, &hist.plants, |v| map_clamped(v, 0.0, 1.0, py + ph, py), C_PLANT);
}

fn draw_graph_energy(x: f32, y: f32, w: f32, h: f32, hist: &StatsHistory) {
    if hist.len() < 2 {
        return;
    }
    let vmax = hist.max_energy_recent().max(0.5) * 1.10;
    let (px, py, pw, ph) = graph_plot_rect(x, y, w, h);
    draw_y_ticks(px, py, pw, ph, 0.0, vmax, fmt_small);
    draw_series_f(px, py, pw, ph, &hist.he, |v| map_clamped(v, 0.0, vmax, py + ph, py), C_HERB);
    draw_series_f(px, py, pw, ph, &hist.pe, |v| map_clamped(v, 0.0, vmax, py + ph, py), C_PRED);
}

fn map_clamped(v: f32, a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
    let t = if (a1 - a0).abs() < 1e-6 { 0.0 } else { (v - a0) / (a1 - a0) };
    let t = t.clamp(0.0, 1.0);
    b0 + (b1 - b0) * t
}

fn draw_series_u<F: Fn(u32) -> f32>(x: f32, y: f32, w: f32, h: f32, data: &VecDeque<u32>, to_y: F, col: Color) {
    let n = data.len();
    if n < 2 {
        return;
    }

    let mut prev = data[0];
    for i in 1..n {
        let cur = data[i];
        let t0 = (i - 1) as f32 / (n - 1) as f32;
        let t1 = i as f32 / (n - 1) as f32;

        let x0 = x + t0 * w;
        let x1 = x + t1 * w;

        let y0 = to_y(prev).clamp(y, y + h);
        let y1 = to_y(cur).clamp(y, y + h);

        draw_line(x0, y0, x1, y1, 2.0, col);
        prev = cur;
    }
}

fn draw_series_u_dim<F: Fn(u32) -> f32>(x: f32, y: f32, w: f32, h: f32, data: &VecDeque<u32>, to_y: F, col: Color) {
    let n = data.len();
    if n < 2 {
        return;
    }

    let dim = Color::new(col.r, col.g, col.b, 0.55);

    let mut prev = data[0];
    for i in 1..n {
        let cur = data[i];
        let t0 = (i - 1) as f32 / (n - 1) as f32;
        let t1 = i as f32 / (n - 1) as f32;

        let x0 = x + t0 * w;
        let x1 = x + t1 * w;

        let y0 = to_y(prev).clamp(y, y + h);
        let y1 = to_y(cur).clamp(y, y + h);

        draw_line(x0, y0, x1, y1, 2.0, dim);
        prev = cur;
    }
}

fn draw_series_f<F: Fn(f32) -> f32>(x: f32, y: f32, w: f32, h: f32, data: &VecDeque<f32>, to_y: F, col: Color) {
    let n = data.len();
    if n < 2 {
        return;
    }

    let mut prev = data[0];
    for i in 1..n {
        let cur = data[i];
        let t0 = (i - 1) as f32 / (n - 1) as f32;
        let t1 = i as f32 / (n - 1) as f32;

        let x0 = x + t0 * w;
        let x1 = x + t1 * w;

        let y0 = to_y(prev).clamp(y, y + h);
        let y1 = to_y(cur).clamp(y, y + h);

        draw_line(x0, y0, x1, y1, 2.0, col);
        prev = cur;
    }
}

fn row(x: f32, cy: &mut f32, k: &str, v: &str, c: Color) {
    draw_text(k, x, *cy, 18.0, SUB);
    draw_text(v, x + 150.0, *cy, 18.0, c);
    *cy += 22.0;
}

fn row_color(x: f32, cy: &mut f32, k: &str, v: &str, c: Color) {
    draw_text(k, x, *cy, 18.0, SUB);
    draw_text(v, x + 150.0, *cy, 18.0, c);
    *cy += 22.0;
}

fn draw_text_center(s: &str, cx: f32, cy: f32, sz: f32, col: Color) {
    let m = measure_text(s, None, sz as u16, 1.0);
    draw_text(s, cx - m.width * 0.5, cy, sz, col);
}
