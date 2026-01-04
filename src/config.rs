use macroquad::prelude::*;


pub const W: i32 = 120;
pub const H: i32 = 80;
pub const CELL: f32 = 8.0;


pub const PANEL_W: f32 = 420.0;
pub const PAD: f32 = 14.0;

// save/load
pub const SAVE_PATH: &str = "ecosim_save.bin";

// colors
pub const BG: Color = Color::new(0.06, 0.07, 0.10, 1.0);
pub const PANEL_BG: Color = Color::new(0.08, 0.09, 0.13, 1.0);
pub const GRID_BG: Color = Color::new(0.05, 0.06, 0.09, 1.0);
pub const LINE: Color = Color::new(0.20, 0.22, 0.30, 1.0);

pub const TXT: Color = Color::new(0.93, 0.94, 0.98, 1.0);
pub const SUB: Color = Color::new(0.70, 0.72, 0.80, 1.0);

pub const C_PLANT: Color = Color::new(0.20, 0.90, 0.35, 1.0);
pub const C_HERB: Color = Color::new(0.98, 0.85, 0.15, 1.0);
pub const C_PRED: Color = Color::new(0.95, 0.25, 0.20, 1.0);

pub const C_OK: Color = Color::new(0.30, 0.90, 0.55, 1.0);
pub const C_WARN: Color = Color::new(0.95, 0.85, 0.15, 1.0);
pub const C_BAD: Color = Color::new(0.95, 0.25, 0.20, 1.0);

#[derive(Clone, Copy)]
pub struct SimSettings {
    pub init_herbs: u32,
    pub init_preds: u32,
    pub plant_grow: u8,
    pub plant_spread: f32,
    pub herb_speed: f32,
    pub pred_speed: f32,
    pub herb_met: f32,
    pub pred_met: f32,
    pub eat_radius: f32,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            init_herbs: 900,
            init_preds: 40,
            plant_grow: 5,
            plant_spread: 0.30,
            herb_speed: 0.22,
            pred_speed: 0.32,
            herb_met: 0.014,
            pred_met: 0.020,
            eat_radius: 0.75,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SimTuning {
    pub fixed_dt: f32,
    pub max_steps_per_frame: u32,
}

impl Default for SimTuning {
    fn default() -> Self {
        Self {
            fixed_dt: 1.0 / 60.0,
            max_steps_per_frame: 8,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SimMode {
    Home,
    Running,
    Paused,
}

pub struct Layout {
    pub world_w_px: f32,
    pub world_h_px: f32,
    pub panel_x: f32,
    pub panel_w: f32,
}

impl Layout {
    pub fn compute(sw: f32, sh: f32) -> Self {
        let panel_w = PANEL_W;
        let world_w = (sw - panel_w).max(320.0);
        Self {
            world_w_px: world_w,
            world_h_px: sh,
            panel_x: world_w,
            panel_w,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Counts {
    pub plants_avg: f32,
    pub herbs: u32,
    pub preds: u32,
    pub herb_e_avg: f32,
    pub pred_e_avg: f32,
}

#[derive(Clone, Copy)]
pub struct Deltas {
    pub herb_birth: u32,
    pub herb_death: u32,
    pub pred_birth: u32,
    pub pred_death: u32,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Herb,
    Pred,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TrackTarget {
    pub kind: TrackKind,
    pub id: u32,
}
