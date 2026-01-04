use rand09::{rngs::StdRng, SeedableRng};

pub fn gen_seed(cpu_threads: usize) -> u64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let a = t.as_nanos() as u64;
    let b = (cpu_threads as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    a ^ b.rotate_left(17) ^ 0xD1B5_4A32_D192_ED03
}

pub fn rng_from_seed(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

pub fn wrap_i(x: i32, m: i32) -> i32 {
    let mut v = x % m;
    if v < 0 {
        v += m;
    }
    v
}

pub fn wrap_f(x: f32, m: f32) -> f32 {
    let mut v = x % m;
    if v < 0.0 {
        v += m;
    }
    v
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn fmt_compact(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}m", (n as f64) / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", (n as f64) / 1_000.0)
    } else {
        n.to_string()
    }
}
