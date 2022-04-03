use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Mutex,
    }, time::Duration,
};

use ndarray::{s, Array2};
use rand::Rng;

#[derive(Debug)]
pub struct TerrainArray {
    data_read: Array2<u8>,
    shapes: HashMap<Shape, u8>,
    shapes_sender: Sender<HashMap<Shape, u8>>,
    outputs_receiver: Receiver<Array2<u8>>,
}

pub const BLIGHT: u8 = u8::MAX;
pub const CLEAN: u8 = 0u8;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Shape {
    Circle { center: [usize; 2], radius: usize },
}

impl TerrainArray {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 256;

    pub fn new() -> Self {
        let (shapes_sender, shapes_receiver): (_, Receiver<HashMap<Shape, u8>>) =
            std::sync::mpsc::channel();
        let (outputs_sender, outputs_receiver) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut array = Array2::from_elem((Self::WIDTH, Self::HEIGHT), CLEAN);
            outputs_sender.send(array.clone()).unwrap();
            while let Ok(input) = shapes_receiver.recv() {
                for (shape, fill) in input.into_iter() {
                    Self::do_fill_shape(&mut array, shape, fill);
                }
                Self::do_dilate(&mut array);
                outputs_sender.send(array.clone()).unwrap();

                std::thread::sleep(Duration::from_millis(500));
            }
        });

        Self {
            data_read: Array2::from_elem((Self::WIDTH, Self::HEIGHT), CLEAN),
            shapes: HashMap::new(),
            shapes_sender,
            outputs_receiver,
        }
    }

    fn do_fill_shape(data_write: &mut Array2<u8>, shape: Shape, fill: u8) {
        match shape {
            Shape::Circle { center, radius } => {
                let window_size = radius * 2 + 1;
                let wi = center[0].saturating_sub(radius);
                let wj = center[1].saturating_sub(radius);
                data_write
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

    pub fn fill_shape(&mut self, shape: Shape, fill: u8) {
        self.shapes.insert(shape, fill);
    }

    pub fn query_shape_avg(&self, shape: Shape) -> u8 {
        match shape {
            Shape::Circle { center, radius } => {
                let mut sum: usize = 0;
                let mut count: usize = 0;
                let window_size = radius * 2 + 1;
                let wi = center[0].saturating_sub(radius);
                let wj = center[1].saturating_sub(radius);
                self.data_read
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

    pub fn do_dilate(data_write: &mut Array2<u8>) {
        let mut new_data = Array2::zeros(data_write.raw_dim());

        let dist = rand::distributions::Uniform::<usize>::new(0, 4);
        let mut rng = rand::thread_rng();

        let kernels = [
            ndarray::array![
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            ndarray::array![
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 1, 1, 1],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            ndarray::array![
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
                [1, 1, 1, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ],
            ndarray::array![
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
            ],
        ];

        ndarray::Zip::from(new_data.slice_mut(s![2..Self::WIDTH - 2, 2..Self::HEIGHT - 2]))
            .and(data_write.windows((5, 5)))
            .for_each(|v: &mut u8, window| {
                if *v < BLIGHT {
                    *v = ndarray::Zip::from(window)
                        .and(&kernels[rng.sample(dist)])
                        .fold(CLEAN, |acc, val, k| acc.max(*val * k));
                }
            });
        *data_write = new_data;
    }

    pub fn data(&self) -> &Array2<u8> {
        &self.data_read
    }

    pub fn swap_if_ready(&mut self) {
        if let Ok(array) = self.outputs_receiver.try_recv() {
            self.data_read = array;
            self.shapes_sender
                .send(std::mem::take(&mut self.shapes))
                .unwrap();
        }
    }
}

impl Default for TerrainArray {
    fn default() -> Self {
        Self::new()
    }
}
