use std::collections::HashSet;
use std::convert::TryFrom;

use rayon::prelude::*;

pub mod image;

pub use image::{Image, Rgb};

pub fn generate_equally_spaced_rgb_colors(num_colors: usize) -> Vec<Rgb> {
    let mut colors = Vec::with_capacity(num_colors.pow(3));

    for ri in 0..num_colors {
        for gi in 0..num_colors {
            for bi in 0..num_colors {
                colors.push((
                    u8::try_from(ri * 255 / num_colors).unwrap(),
                    u8::try_from(gi * 255 / num_colors).unwrap(),
                    u8::try_from(bi * 255 / num_colors).unwrap(),
                ));
            }
        }
    }

    colors
}

pub fn generate(
    mut colors: Vec<Rgb>,
    (width, height): (usize, usize),
    seeds: HashSet<(usize, usize)>,
) -> Image<Rgb> {
    assert!(colors.len() == width * height);
    assert!(!seeds.is_empty());

    let mut img = Image::new((0, 0, 0), width, height);
    let mut seen = Image::new(false, width, height);
    let mut free = HashSet::new();

    for (sx, sy) in seeds {
        img.set(sx, sy, colors.pop().unwrap());
        seen.set(sx, sy, true);

        img.for_each_neighbor(sx, sy, |x, y| {
            free.insert((x, y));
        });
    }

    while let Some(rgb) = colors.pop() {
        let &(x, y) = free
            .par_iter()
            .min_by_key(|&&(x, y)| {
                let mut neighbors = 0;
                let mut total_color_dist = 0;

                img.for_each_neighbor(x, y, |xx, yy| {
                    if !seen.get(xx, yy) {
                        return;
                    }

                    neighbors += 1;
                    total_color_dist += color_dist(rgb, img.get(xx, yy));
                });

                debug_assert!(neighbors > 0);
                total_color_dist / neighbors
            })
            .unwrap();

        debug_assert!(!seen.get(x, y));
        seen.set(x, y, true);

        free.remove(&(x, y));
        img.for_each_neighbor(x, y, |nx, ny| {
            if !seen.get(nx, ny) {
                free.insert((nx, ny));
            }
        });

        img.set(x, y, rgb);
    }

    img
}

fn color_dist((r1, g1, b1): Rgb, (r2, g2, b2): Rgb) -> i32 {
    let dr = i32::from(r1) - i32::from(r2);
    let dg = i32::from(g1) - i32::from(g2);
    let db = i32::from(b1) - i32::from(b2);

    dr * dr + dg * dg + db * db
}
