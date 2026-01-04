use std::fs::File;
use std::io::{Read, Write};

use macroquad::prelude::*;
use rand09::Rng;

use crate::config::*;
use crate::util::*;

#[derive(Clone, Copy)]
struct Agent {
    id: u32,
    p: Vec2,
    pp: Vec2,
    e: f32,
    age: u32,
}

pub struct TrackedInfo {
    pub kind: TrackKind,
    pub id: u32,
    pub e: f32,
    pub age: u32,
    pub p: Vec2,
}

pub struct World {
    plants: Vec<u8>,
    herbs: Vec<Agent>,
    preds: Vec<Agent>,
    next_id: u32,
}

impl World {
    pub fn new(rng: &mut impl Rng, set: SimSettings) -> Self {
        let n = (W * H) as usize;

        let mut plants = vec![0u8; n];
        for i in 0..n {
            let r: f32 = rng.random();
            plants[i] = if r < 0.55 {
                (r * 255.0) as u8
            } else {
                (180.0 + 75.0 * rng.random::<f32>()) as u8
            };
        }

        let mut next_id = 1u32;

        let mut herbs = Vec::new();
        let mut preds = Vec::new();

        for _ in 0..set.init_herbs {
            let p = vec2(rng.random::<f32>() * W as f32, rng.random::<f32>() * H as f32);
            herbs.push(Agent { id: next_id, p, pp: p, e: 1.2 + 0.6 * rng.random::<f32>(), age: 0 });
            next_id += 1;
        }

        for _ in 0..set.init_preds {
            let p = vec2(rng.random::<f32>() * W as f32, rng.random::<f32>() * H as f32);
            preds.push(Agent { id: next_id, p, pp: p, e: 1.6 + 0.8 * rng.random::<f32>(), age: 0 });
            next_id += 1;
        }

        Self { plants, herbs, preds, next_id }
    }

    pub fn counts(&self) -> Counts {
        let mut s = 0u64;
        for &v in &self.plants {
            s += v as u64;
        }
        let plants_avg = (s as f32) / (self.plants.len() as f32) / 255.0;

        let mut he = 0.0f32;
        for a in &self.herbs {
            he += a.e;
        }
        let herb_e_avg = if self.herbs.is_empty() { 0.0 } else { he / self.herbs.len() as f32 };

        let mut pe = 0.0f32;
        for a in &self.preds {
            pe += a.e;
        }
        let pred_e_avg = if self.preds.is_empty() { 0.0 } else { pe / self.preds.len() as f32 };

        Counts {
            plants_avg,
            herbs: self.herbs.len() as u32,
            preds: self.preds.len() as u32,
            herb_e_avg,
            pred_e_avg,
        }
    }

    pub fn tracked_info(&self, t: TrackTarget) -> Option<TrackedInfo> {
        match t.kind {
            TrackKind::Herb => {
                for a in &self.herbs {
                    if a.id == t.id {
                        return Some(TrackedInfo { kind: TrackKind::Herb, id: a.id, e: a.e, age: a.age, p: a.p });
                    }
                }
                None
            }
            TrackKind::Pred => {
                for a in &self.preds {
                    if a.id == t.id {
                        return Some(TrackedInfo { kind: TrackKind::Pred, id: a.id, e: a.e, age: a.age, p: a.p });
                    }
                }
                None
            }
        }
    }

    pub fn pick_target(&self, world_pos: Vec2) -> Option<TrackTarget> {
        let mut best: Option<TrackTarget> = None;
        let mut bestd = 9999.0f32;

        let r = 1.25;

        for a in &self.herbs {
            let d = toroid_dist(world_pos, a.p);
            if d < r && d < bestd {
                bestd = d;
                best = Some(TrackTarget { kind: TrackKind::Herb, id: a.id });
            }
        }

        for a in &self.preds {
            let d = toroid_dist(world_pos, a.p);
            if d < r && d < bestd {
                bestd = d;
                best = Some(TrackTarget { kind: TrackKind::Pred, id: a.id });
            }
        }

        best
    }

    pub fn step(&mut self, rng: &mut impl Rng, set: SimSettings, _dt: f32) -> Deltas {
        self.plants_step(rng, set);
        let eaten = self.preds_step(rng, set);
        self.herbs_step(rng, set);
        self.cleanup_repro(rng, set, eaten)
    }

    pub fn draw(&self, layout: &Layout, alpha: f32, tracked: Option<TrackTarget>) {
        draw_rectangle(0.0, 0.0, layout.world_w_px, layout.world_h_px, GRID_BG);

        for y in 0..H {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let v = self.plants[i] as f32 / 255.0;
                if v <= 0.01 {
                    continue;
                }
                let col = Color::new(
                    lerp(GRID_BG.r, C_PLANT.r, v),
                    lerp(GRID_BG.g, C_PLANT.g, v),
                    lerp(GRID_BG.b, C_PLANT.b, v),
                    1.0,
                );
                draw_rectangle((x as f32) * CELL, (y as f32) * CELL, CELL, CELL, col);
            }
        }

        let mut tracked_px: Option<Vec2> = None;

        for a in &self.herbs {
            let p = interp_agent(a, alpha);
            let px = p.x * CELL + CELL * 0.5;
            let py = p.y * CELL + CELL * 0.5;
            draw_circle(px, py, CELL * 0.42, C_HERB);
            if tracked == Some(TrackTarget { kind: TrackKind::Herb, id: a.id }) {
                tracked_px = Some(vec2(px, py));
            }
        }

        for a in &self.preds {
            let p = interp_agent(a, alpha);
            let px = p.x * CELL + CELL * 0.5;
            let py = p.y * CELL + CELL * 0.5;
            draw_poly(px, py, 3, CELL * 0.55, 0.0, C_PRED);
            if tracked == Some(TrackTarget { kind: TrackKind::Pred, id: a.id }) {
                tracked_px = Some(vec2(px, py));
            }
        }

        if let Some(tp) = tracked_px {
            draw_circle_lines(tp.x, tp.y, CELL * 0.78, 3.0, Color::new(0.95, 0.95, 1.0, 0.90));
            draw_circle_lines(tp.x, tp.y, CELL * 0.98, 2.0, Color::new(0.20, 0.60, 1.0, 0.65));
        }

        draw_rectangle_lines(0.0, 0.0, layout.world_w_px, layout.world_h_px, 2.0, LINE);
    }

    pub fn save(&self, path: &str, set: SimSettings) -> bool {
        let mut f = match File::create(path) {
            Ok(v) => v,
            Err(_) => return false,
        };

        if f.write_all(b"ECO3").is_err() {
            return false;
        }

        if write_i32(&mut f, W).is_err() || write_i32(&mut f, H).is_err() {
            return false;
        }

        if write_settings(&mut f, set).is_err() {
            return false;
        }

        if write_u32(&mut f, self.next_id).is_err() {
            return false;
        }

        if write_u32(&mut f, self.plants.len() as u32).is_err() {
            return false;
        }
        if f.write_all(&self.plants).is_err() {
            return false;
        }

        if write_u32(&mut f, self.herbs.len() as u32).is_err() {
            return false;
        }
        for a in &self.herbs {
            if write_agent(&mut f, a).is_err() {
                return false;
            }
        }

        if write_u32(&mut f, self.preds.len() as u32).is_err() {
            return false;
        }
        for a in &self.preds {
            if write_agent(&mut f, a).is_err() {
                return false;
            }
        }

        true
    }

    pub fn load(path: &str) -> Option<(Self, SimSettings)> {
        let mut f = File::open(path).ok()?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic).ok()?;
        if &magic != b"ECO3" {
            return None;
        }

        let w = read_i32(&mut f).ok()?;
        let h = read_i32(&mut f).ok()?;
        if w != W || h != H {
            return None;
        }

        let set = read_settings(&mut f).ok()?;
        let next_id = read_u32(&mut f).ok()?;

        let n = read_u32(&mut f).ok()? as usize;
        if n != (W * H) as usize {
            return None;
        }

        let mut plants = vec![0u8; n];
        f.read_exact(&mut plants).ok()?;

        let hn = read_u32(&mut f).ok()? as usize;
        let mut herbs = Vec::with_capacity(hn);
        for _ in 0..hn {
            herbs.push(read_agent(&mut f).ok()?);
        }

        let pn = read_u32(&mut f).ok()? as usize;
        let mut preds = Vec::with_capacity(pn);
        for _ in 0..pn {
            preds.push(read_agent(&mut f).ok()?);
        }

        Some((Self { plants, herbs, preds, next_id }, set))
    }

    fn plants_step(&mut self, rng: &mut impl Rng, set: SimSettings) {
        let n = self.plants.len();
        let grow = set.plant_grow as i32;

        for i in 0..n {
            let v = self.plants[i] as i32;
            let jitter = (2.0 * rng.random::<f32>()) as i32;
            self.plants[i] = (v + grow + jitter).min(255) as u8;
        }

        let tries = (W * H / 6) as usize;
        for _ in 0..tries {
            let x = (rng.random::<f32>() * W as f32) as i32;
            let y = (rng.random::<f32>() * H as f32) as i32;
            let i = (y * W + x) as usize;

            if self.plants[i] < 110 {
                continue;
            }

            let dx = (rng.random::<f32>() * 3.0) as i32 - 1;
            let dy = (rng.random::<f32>() * 3.0) as i32 - 1;
            if dx == 0 && dy == 0 {
                continue;
            }

            let xx = wrap_i(x + dx, W);
            let yy = wrap_i(y + dy, H);
            let j = (yy * W + xx) as usize;

            if self.plants[j] < 60 && rng.random::<f32>() < set.plant_spread {
                self.plants[j] = (self.plants[j] + 45).min(255);
            }
        }
    }

    fn herbs_step(&mut self, rng: &mut impl Rng, set: SimSettings) {
        let preds_pos: Vec<Vec2> = self.preds.iter().map(|p| p.p).collect();
        let speed = set.herb_speed;
        let plants = &mut self.plants;

        for h in &mut self.herbs {
            h.pp = h.p;
            h.e -= set.herb_met;
            h.age = h.age.saturating_add(1);

            let dir = herb_dir(h.p, &*plants, &preds_pos, rng);
            h.p.x = wrap_f(h.p.x + dir.x * speed, W as f32);
            h.p.y = wrap_f(h.p.y + dir.y * speed, H as f32);

            let cx = wrap_i(h.p.x.floor() as i32, W);
            let cy = wrap_i(h.p.y.floor() as i32, H);
            let i = (cy * W + cx) as usize;

            let bite = 16u8;
            let avail = plants[i];
            let take = avail.min(bite);
            plants[i] = avail - take;

            h.e += (take as f32) * 0.0022;
        }
    }

    fn preds_step(&mut self, rng: &mut impl Rng, set: SimSettings) -> u32 {
        let mut herb_pos: Vec<Vec2> = self.herbs.iter().map(|h| h.p).collect();
        let speed = set.pred_speed;
        let eat_r = set.eat_radius;

        let mut eaten = 0u32;

        let mut pi = 0usize;
        while pi < self.preds.len() {
            let p = &mut self.preds[pi];
            p.pp = p.p;
            p.e -= set.pred_met;
            p.age = p.age.saturating_add(1);

            let dir = pred_dir(p.p, &herb_pos, rng);
            p.p.x = wrap_f(p.p.x + dir.x * speed, W as f32);
            p.p.y = wrap_f(p.p.y + dir.y * speed, H as f32);

            if let Some(hi) = nearest_within(p.p, &herb_pos, eat_r) {
                self.herbs.swap_remove(hi);
                herb_pos.swap_remove(hi);
                p.e += 0.85;
                eaten += 1;
            }

            pi += 1;
        }

        eaten
    }

    fn cleanup_repro(&mut self, rng: &mut impl Rng, _set: SimSettings, eaten: u32) -> Deltas {
        let herb_before = self.herbs.len() as u32;
        let pred_before = self.preds.len() as u32;

        self.herbs.retain(|h| h.e > 0.0);
        self.preds.retain(|p| p.e > 0.0);

        let herb_after = self.herbs.len() as u32;
        let pred_after = self.preds.len() as u32;

        let mut herb_birth = 0u32;
        let mut pred_birth = 0u32;

        let mut new_herbs = Vec::new();
        for h in &mut self.herbs {
            if h.e > 2.2 && rng.random::<f32>() < 0.10 {
                h.e *= 0.62;
                let jitter = vec2(rng.random::<f32>() - 0.5, rng.random::<f32>() - 0.5) * 0.9;
                let np = vec2(wrap_f(h.p.x + jitter.x, W as f32), wrap_f(h.p.y + jitter.y, H as f32));
                let id = self.next_id;
                self.next_id = self.next_id.saturating_add(1);
                new_herbs.push(Agent { id, p: np, pp: np, e: 1.0, age: 0 });
                herb_birth += 1;
            }
        }
        self.herbs.extend(new_herbs);

        let mut new_preds = Vec::new();
        for p in &mut self.preds {
            if p.e > 2.7 && rng.random::<f32>() < 0.08 {
                p.e *= 0.64;
                let jitter = vec2(rng.random::<f32>() - 0.5, rng.random::<f32>() - 0.5) * 0.8;
                let np = vec2(wrap_f(p.p.x + jitter.x, W as f32), wrap_f(p.p.y + jitter.y, H as f32));
                let id = self.next_id;
                self.next_id = self.next_id.saturating_add(1);
                new_preds.push(Agent { id, p: np, pp: np, e: 1.2, age: 0 });
                pred_birth += 1;
            }
        }
        self.preds.extend(new_preds);

        if self.herbs.len() < 20 && rng.random::<f32>() < 0.25 {
            for _ in 0..18 {
                let p = vec2(rng.random::<f32>() * W as f32, rng.random::<f32>() * H as f32);
                let id = self.next_id;
                self.next_id = self.next_id.saturating_add(1);
                self.herbs.push(Agent { id, p, pp: p, e: 1.4, age: 0 });
                herb_birth += 1;
            }
        }

        if self.preds.len() < 6 && rng.random::<f32>() < 0.20 {
            for _ in 0..5 {
                let p = vec2(rng.random::<f32>() * W as f32, rng.random::<f32>() * H as f32);
                let id = self.next_id;
                self.next_id = self.next_id.saturating_add(1);
                self.preds.push(Agent { id, p, pp: p, e: 1.8, age: 0 });
                pred_birth += 1;
            }
        }

        let herb_death = (herb_before - herb_after).saturating_add(eaten);
        let pred_death = pred_before - pred_after;

        Deltas { herb_birth, herb_death, pred_birth, pred_death }
    }
}

fn interp_agent(a: &Agent, alpha: f32) -> Vec2 {
    let mut dx = a.p.x - a.pp.x;
    let mut dy = a.p.y - a.pp.y;

    if dx > (W as f32) * 0.5 { dx -= W as f32; }
    if dx < -(W as f32) * 0.5 { dx += W as f32; }
    if dy > (H as f32) * 0.5 { dy -= H as f32; }
    if dy < -(H as f32) * 0.5 { dy += H as f32; }

    let x = wrap_f(a.pp.x + dx * alpha, W as f32);
    let y = wrap_f(a.pp.y + dy * alpha, H as f32);
    vec2(x, y)
}

fn herb_dir(p: Vec2, plants: &[u8], preds_pos: &[Vec2], rng: &mut impl Rng) -> Vec2 {
    let base = best_plant_dir(p, plants);
    let flee = flee_dir(p, preds_pos);
    let mut d = base + flee * 1.15;
    d += vec2(rng.random::<f32>() - 0.5, rng.random::<f32>() - 0.5) * 0.35;
    norm_or_rand(d, rng)
}

fn pred_dir(p: Vec2, herb_pos: &[Vec2], rng: &mut impl Rng) -> Vec2 {
    let chase = chase_dir(p, herb_pos);
    let j = vec2(rng.random::<f32>() - 0.5, rng.random::<f32>() - 0.5) * 0.22;
    norm_or_rand(chase + j, rng)
}

fn best_plant_dir(p: Vec2, plants: &[u8]) -> Vec2 {
    let cx = wrap_i(p.x.floor() as i32, W);
    let cy = wrap_i(p.y.floor() as i32, H);

    let mut best = -1i32;
    let mut bestv = vec2(0.0, 0.0);

    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx == 0 && dy == 0 { continue; }
            let xx = wrap_i(cx + dx, W);
            let yy = wrap_i(cy + dy, H);
            let i = (yy * W + xx) as usize;
            let v = plants[i] as i32 - (dx * dx + dy * dy) * 6;
            if v > best {
                best = v;
                bestv = vec2(dx as f32, dy as f32);
            }
        }
    }

    bestv
}

fn flee_dir(p: Vec2, preds_pos: &[Vec2]) -> Vec2 {
    if preds_pos.is_empty() { return vec2(0.0, 0.0); }

    let mut best = vec2(0.0, 0.0);
    let mut bestd = 9999.0;

    for &pp in preds_pos {
        let d = toroid_dist(p, pp);
        if d < bestd {
            bestd = d;
            best = toroid_vec(pp, p);
        }
    }

    if bestd < 8.0 { best } else { vec2(0.0, 0.0) }
}

fn chase_dir(p: Vec2, herb_pos: &[Vec2]) -> Vec2 {
    if herb_pos.is_empty() { return vec2(0.0, 0.0); }

    let mut best = vec2(0.0, 0.0);
    let mut bestd = 9999.0;

    for &hp in herb_pos {
        let d = toroid_dist(p, hp);
        if d < bestd {
            bestd = d;
            best = toroid_vec(p, hp);
        }
    }

    if bestd < 18.0 { best } else { vec2(0.0, 0.0) }
}

fn nearest_within(p: Vec2, pts: &[Vec2], r: f32) -> Option<usize> {
    let mut best = None;
    let mut bestd = r;

    for (i, &q) in pts.iter().enumerate() {
        let d = toroid_dist(p, q);
        if d < bestd {
            bestd = d;
            best = Some(i);
        }
    }

    best
}

fn toroid_vec(a: Vec2, b: Vec2) -> Vec2 {
    let mut dx = b.x - a.x;
    let mut dy = b.y - a.y;

    if dx > (W as f32) * 0.5 { dx -= W as f32; }
    if dx < -(W as f32) * 0.5 { dx += W as f32; }
    if dy > (H as f32) * 0.5 { dy -= H as f32; }
    if dy < -(H as f32) * 0.5 { dy += H as f32; }

    vec2(dx, dy)
}

fn toroid_dist(a: Vec2, b: Vec2) -> f32 {
    toroid_vec(a, b).length()
}

fn norm_or_rand(d: Vec2, rng: &mut impl Rng) -> Vec2 {
    let l = d.length();
    if l > 0.0001 { d / l } else {
        let a = rng.random::<f32>() * std::f32::consts::TAU;
        vec2(a.cos(), a.sin())
    }
}

fn write_i32(w: &mut File, v: i32) -> std::io::Result<()> { w.write_all(&v.to_le_bytes()) }
fn write_u32(w: &mut File, v: u32) -> std::io::Result<()> { w.write_all(&v.to_le_bytes()) }
fn write_u8(w: &mut File, v: u8) -> std::io::Result<()> { w.write_all(&[v]) }
fn write_f32(w: &mut File, v: f32) -> std::io::Result<()> { w.write_all(&v.to_le_bytes()) }

fn read_i32(r: &mut File) -> std::io::Result<i32> { let mut b=[0u8;4]; r.read_exact(&mut b)?; Ok(i32::from_le_bytes(b)) }
fn read_u32(r: &mut File) -> std::io::Result<u32> { let mut b=[0u8;4]; r.read_exact(&mut b)?; Ok(u32::from_le_bytes(b)) }
fn read_u8(r: &mut File) -> std::io::Result<u8> { let mut b=[0u8;1]; r.read_exact(&mut b)?; Ok(b[0]) }
fn read_f32(r: &mut File) -> std::io::Result<f32> { let mut b=[0u8;4]; r.read_exact(&mut b)?; Ok(f32::from_le_bytes(b)) }

fn write_agent(w: &mut File, a: &Agent) -> std::io::Result<()> {
    write_u32(w, a.id)?;
    write_f32(w, a.p.x)?;
    write_f32(w, a.p.y)?;
    write_f32(w, a.pp.x)?;
    write_f32(w, a.pp.y)?;
    write_f32(w, a.e)?;
    write_u32(w, a.age)?;
    Ok(())
}

fn read_agent(r: &mut File) -> std::io::Result<Agent> {
    let id = read_u32(r)?;
    let px = read_f32(r)?;
    let py = read_f32(r)?;
    let ppx = read_f32(r)?;
    let ppy = read_f32(r)?;
    let e = read_f32(r)?;
    let age = read_u32(r)?;
    Ok(Agent { id, p: vec2(px, py), pp: vec2(ppx, ppy), e, age })
}

fn write_settings(w: &mut File, s: SimSettings) -> std::io::Result<()> {
    write_u32(w, s.init_herbs)?;
    write_u32(w, s.init_preds)?;
    write_u8(w, s.plant_grow)?;
    write_f32(w, s.plant_spread)?;
    write_f32(w, s.herb_speed)?;
    write_f32(w, s.pred_speed)?;
    write_f32(w, s.herb_met)?;
    write_f32(w, s.pred_met)?;
    write_f32(w, s.eat_radius)?;
    Ok(())
}

fn read_settings(r: &mut File) -> std::io::Result<SimSettings> {
    Ok(SimSettings {
        init_herbs: read_u32(r)?,
        init_preds: read_u32(r)?,
        plant_grow: read_u8(r)?,
        plant_spread: read_f32(r)?,
        herb_speed: read_f32(r)?,
        pred_speed: read_f32(r)?,
        herb_met: read_f32(r)?,
        pred_met: read_f32(r)?,
        eat_radius: read_f32(r)?,
    })
}
