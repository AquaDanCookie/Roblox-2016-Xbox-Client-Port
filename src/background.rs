use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions};

const WIDTH: usize = 960;
const HEIGHT: usize = 540;

pub fn texture(ctx: &Context) -> TextureHandle {
    let mut pixels = vec![Color32::BLACK; WIDTH * HEIGHT];

    for y in 0..HEIGHT {
        let yn = y as f32 / HEIGHT as f32;
        for x in 0..WIDTH {
            let xn = x as f32 / WIDTH as f32;

            let mut r = 9.0 + 15.0 * yn;
            let mut g = 15.0 + 23.0 * yn;
            let mut b = 19.0 + 28.0 * yn;

            for (cx, cy, radius, color, alpha) in [
                (0.18, 0.26, 0.38, (42.0, 68.0, 80.0), 0.30),
                (0.70, 0.20, 0.34, (50.0, 78.0, 90.0), 0.26),
                (0.88, 0.58, 0.30, (34.0, 58.0, 70.0), 0.24),
                (0.42, 0.80, 0.44, (28.0, 48.0, 58.0), 0.22),
            ] {
                let dx = xn - cx;
                let dy = (yn - cy) * 0.74;
                let distance = (dx * dx + dy * dy).sqrt();
                let falloff = (1.0 - distance / radius).max(0.0);
                let weight = falloff * falloff * alpha;
                r += color.0 * weight;
                g += color.1 * weight;
                b += color.2 * weight;
            }

            let beam_center = 0.54 + 0.14 * yn;
            let beam = (-((xn - beam_center) / 0.12).powi(2)).exp() * 0.20;
            r += 74.0 * beam;
            g += 108.0 * beam;
            b += 124.0 * beam;

            let dx = xn - 0.5;
            let dy = yn - 0.48;
            let vignette = (1.0 - (dx * dx + dy * dy).min(0.5) * 1.15).max(0.22);
            r *= vignette;
            g *= vignette;
            b *= vignette;

            let noise = hash_noise(x, y) * 4.5;
            pixels[y * WIDTH + x] = Color32::from_rgb(
                (r + noise).clamp(0.0, 255.0) as u8,
                (g + noise).clamp(0.0, 255.0) as u8,
                (b + noise).clamp(0.0, 255.0) as u8,
            );
        }
    }

    paint_ridges(&mut pixels);
    paint_building(&mut pixels);
    soften(&mut pixels);
    paint_bokeh(&mut pixels);

    let image = ColorImage::new([WIDTH, HEIGHT], pixels);
    ctx.load_texture("procedural-background", image, TextureOptions::LINEAR)
}

fn paint_ridges(pixels: &mut [Color32]) {
    let layers = [
        (0.43, 0.055, 4.2, 0.8, (27, 40, 47), 0.55),
        (0.53, 0.065, 6.4, 2.3, (19, 30, 36), 0.72),
        (0.68, 0.075, 8.8, 4.1, (12, 20, 25), 0.88),
    ];

    for (base, amplitude, frequency, phase, color, alpha) in layers {
        let layer = Color32::from_rgb(color.0, color.1, color.2);
        for x in 0..WIDTH {
            let xn = x as f32 / WIDTH as f32;
            let ridge = ridge_height(xn, base, amplitude, frequency, phase);
            let start = (ridge * HEIGHT as f32) as usize;
            for y in start.min(HEIGHT)..HEIGHT {
                let index = y * WIDTH + x;
                pixels[index] = blend(pixels[index], layer, alpha);
            }
        }
    }
}

fn paint_building(pixels: &mut [Color32]) {
    let blocks = [
        (0.69, 0.83, 0.19, 0.56, (10, 17, 21), 0.92),
        (0.60, 0.70, 0.36, 0.56, (13, 21, 26), 0.82),
        (0.79, 0.85, 0.30, 0.56, (8, 14, 18), 0.88),
    ];
    for (x0, x1, y0, y1, color, alpha) in blocks {
        let block = Color32::from_rgb(color.0, color.1, color.2);
        let left = (x0 * WIDTH as f32) as usize;
        let right = (x1 * WIDTH as f32) as usize;
        let top = (y0 * HEIGHT as f32) as usize;
        let bottom = (y1 * HEIGHT as f32) as usize;
        for y in top.min(HEIGHT)..bottom.min(HEIGHT) {
            for x in left.min(WIDTH)..right.min(WIDTH) {
                let index = y * WIDTH + x;
                pixels[index] = blend(pixels[index], block, alpha);
            }
        }
    }

    let antenna = Color32::from_rgb(7, 12, 16);
    let x = (0.765 * WIDTH as f32) as usize;
    for y in (0.10 * HEIGHT as f32) as usize..(0.19 * HEIGHT as f32) as usize {
        let index = y * WIDTH + x;
        if index < pixels.len() {
            pixels[index] = blend(pixels[index], antenna, 0.9);
        }
    }

    let window = Color32::from_rgb(52, 86, 99);
    for row in 0..5 {
        for column in 0..4 {
            if (row * 7 + column * 3) % 5 != 0 {
                continue;
            }
            let x = (0.705 + column as f32 * 0.030) * WIDTH as f32;
            let y = (0.255 + row as f32 * 0.052) * HEIGHT as f32;
            for dy in 0..3 {
                for dx in 0..4 {
                    let px = x as usize + dx;
                    let py = y as usize + dy;
                    if px < WIDTH && py < HEIGHT {
                        let index = py * WIDTH + px;
                        pixels[index] = blend(pixels[index], window, 0.55);
                    }
                }
            }
        }
    }
}

fn paint_bokeh(pixels: &mut [Color32]) {
    let mut state = 0x6d2b_79f5u32;
    for _ in 0..54 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let x = (state >> 8) as f32 / 16_777_216.0;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let y = (state >> 8) as f32 / 16_777_216.0;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let radius = 1.2 + ((state >> 16) as f32 / 65_535.0) * 5.0;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let alpha = 0.06 + ((state >> 16) as f32 / 65_535.0) * 0.18;

        let px = (x * WIDTH as f32) as isize;
        let py = (0.30 + y * 0.66) * HEIGHT as f32;
        let center_y = py as isize;
        let reach = radius.ceil() as isize + 1;
        for oy in -reach..=reach {
            for ox in -reach..=reach {
                let ix = px + ox;
                let iy = center_y + oy;
                if ix < 0 || iy < 0 || ix >= WIDTH as isize || iy >= HEIGHT as isize {
                    continue;
                }
                let distance = ((ox * ox + oy * oy) as f32).sqrt();
                if distance > radius {
                    continue;
                }
                let falloff = 1.0 - distance / radius;
                let weight = falloff * falloff * alpha;
                let index = iy as usize * WIDTH + ix as usize;
                let light = Color32::from_rgb(
                    (90.0 * weight) as u8,
                    (143.0 * weight) as u8,
                    (166.0 * weight) as u8,
                );
                pixels[index] = add(pixels[index], light);
            }
        }
    }
}

fn soften(pixels: &mut [Color32]) {
    let mut horizontal = vec![Color32::BLACK; WIDTH * HEIGHT];
    let radius = 2isize;
    let divisor = (radius * 2 + 1) as u32;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            for dx in -radius..=radius {
                let sx = (x as isize + dx).clamp(0, WIDTH as isize - 1) as usize;
                let pixel = pixels[y * WIDTH + sx];
                r += pixel.r() as u32;
                g += pixel.g() as u32;
                b += pixel.b() as u32;
            }
            horizontal[y * WIDTH + x] = Color32::from_rgb(
                (r / divisor) as u8,
                (g / divisor) as u8,
                (b / divisor) as u8,
            );
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            for dy in -radius..=radius {
                let sy = (y as isize + dy).clamp(0, HEIGHT as isize - 1) as usize;
                let pixel = horizontal[sy * WIDTH + x];
                r += pixel.r() as u32;
                g += pixel.g() as u32;
                b += pixel.b() as u32;
            }
            pixels[y * WIDTH + x] = Color32::from_rgb(
                (r / divisor) as u8,
                (g / divisor) as u8,
                (b / divisor) as u8,
            );
        }
    }
}

fn ridge_height(xn: f32, base: f32, amplitude: f32, frequency: f32, phase: f32) -> f32 {
    let a = (xn * frequency + phase).sin();
    let b = (xn * frequency * 2.3 + phase * 1.7).sin();
    let c = (xn * frequency * 5.1 + phase * 0.3).sin();
    base + amplitude * (0.55 * a + 0.30 * b + 0.15 * c)
}

fn blend(base: Color32, layer: Color32, alpha: f32) -> Color32 {
    let alpha = alpha.clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * alpha).round() as u8;
    Color32::from_rgb(
        mix(base.r(), layer.r()),
        mix(base.g(), layer.g()),
        mix(base.b(), layer.b()),
    )
}

fn add(base: Color32, light: Color32) -> Color32 {
    Color32::from_rgb(
        base.r().saturating_add(light.r()),
        base.g().saturating_add(light.g()),
        base.b().saturating_add(light.b()),
    )
}

fn hash_noise(x: usize, y: usize) -> f32 {
    let mut value = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    value = (value ^ (value >> 13)).wrapping_mul(1_274_126_177);
    value ^= value >> 16;
    (value & 0xffff) as f32 / 65535.0 - 0.5
}
