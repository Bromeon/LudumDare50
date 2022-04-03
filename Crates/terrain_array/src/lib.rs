use ndarray::{s, Array2};

#[derive(Debug)]
pub struct TerrainArray {
    data: Array2<u8>,
}

pub const BLIGHT: u8 = u8::MAX;
pub const CLEAN: u8 = 0u8;

pub enum Shape {
    Circle { center: [usize; 2], radius: usize },
}

impl TerrainArray {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 256;

    pub fn new() -> Self {
        Self {
            data: Array2::from_elem((Self::WIDTH, Self::HEIGHT), CLEAN),
        }
    }

    pub fn fill_shape(&mut self, shape: Shape, fill: u8) {
        match shape {
            Shape::Circle { center, radius } => {
                let window_size = radius * 2 + 1;
                let wi = center[0].saturating_sub(radius);
                let wj = center[1].saturating_sub(radius);
                self.data
                    .slice_mut(s![wi..wi + window_size, wj..wj + window_size])
                    .indexed_iter_mut()
                    .for_each(|((i, j), value)| {
                        let di = radius as isize - i as isize;
                        let dj = radius as isize - j as isize;
                        let dist_sq = (di * di + dj * dj) as usize;
                        if dist_sq <= radius * radius {
                            *value = fill
                        }
                    });
            }
        }
    }

    pub fn query_shape_avg(&self, shape: Shape) -> u8 {
        match shape {
            Shape::Circle { center, radius } => {
                let mut sum: usize = 0;
                let mut count: usize = 0;
                let window_size = radius * 2 + 1;
                let wi = center[0].saturating_sub(radius);
                let wj = center[1].saturating_sub(radius);
                self.data
                    .slice(s![wi..wi + window_size, wj..wj + window_size])
                    .indexed_iter()
                    .for_each(|((i, j), value)| {
                        let di = radius as isize - i as isize;
                        let dj = radius as isize - j as isize;
                        let dist_sq = (di * di + dj * dj) as usize;
                        if dist_sq <= radius * radius {
                            sum += *value as usize;
                            count += 1;
                        }
                    });
                (sum / count) as u8
            }
        }
    }

    pub fn dilate(&mut self) {
        let mut new_data = Array2::zeros(self.data.raw_dim());
        ndarray::Zip::from(new_data.slice_mut(s![2..Self::WIDTH - 2, 2..Self::HEIGHT - 2]))
            .and(self.data.windows((5, 5)))
            .for_each(|v, window| {
                let kernel = ndarray::array![
                    [0, 1, 1, 1, 0],
                    [1, 1, 1, 1, 1],
                    [1, 1, 1, 1, 1],
                    [1, 1, 1, 1, 1],
                    [0, 1, 1, 1, 0],
                ];
                *v = ndarray::Zip::from(window)
                    .and(&kernel)
                    .fold(CLEAN, |acc, val, k| acc.max(*val * k));
            });
        self.data = new_data;
    }

    pub fn data(&self) -> &Array2<u8> {
        &self.data
    }
}

impl Default for TerrainArray {
    fn default() -> Self {
        Self::new()
    }
}
