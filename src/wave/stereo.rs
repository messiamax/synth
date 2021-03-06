use std::{iter::zip, path::Path};

use hound::WavSpec;

use crate::{consts::SAMPLE_RATE, utils};

use super::Wave;

const STD_SPEC: WavSpec = WavSpec {
    channels: 2,
    sample_rate: SAMPLE_RATE as u32,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

#[derive(Debug, Clone)]
pub struct Stereo {
    right: Vec<f64>,
    left: Vec<f64>,
}

impl Wave for Stereo {
    fn new() -> Self {
        Self {
            right: Vec::new(),
            left: Vec::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            right: Vec::with_capacity(capacity),
            left: Vec::with_capacity(capacity),
        }
    }

    fn zeros(length: usize) -> Self {
        Self {
            right: vec![0.0; length],
            left: vec![0.0; length],
        }
    }

    fn ones(length: usize) -> Self {
        Self {
            right: vec![1.0; length],
            left: vec![1.0; length],
        }
    }

    fn from_vec(vec: Vec<f64>) -> Self {
        Self {
            right: vec.clone(),
            left: vec,
        }
    }

    fn resize(&mut self, new_len: usize, value: f64) {
        self.right.resize(new_len, value);
        self.left.resize(new_len, value)
    }

    fn clear(&mut self) {
        self.right.clear();
        self.left.clear()
    }

    fn add(&mut self, other: &Self, index: usize) {
        if index == 0 && self.len() == other.len() {
            for (e1, e2) in zip(&mut self.right, other.right.iter()) {
                *e1 += e2;
            }
            for (e1, e2) in zip(&mut self.left, other.left.iter()) {
                *e1 += e2;
            }
        } else {
            if self.len() < index + other.len() {
                self.resize(index + other.len(), 0.0)
            }
            for i in 0..other.len() {
                self.right[i + index] += other.right[i];
            }
            for i in 0..other.len() {
                self.left[i + index] += other.left[i];
            }
        }
    }

    fn add_consuming(&mut self, other: Self, index: usize) {
        if index == 0 && self.len() == other.len() {
            for (e1, e2) in zip(&mut self.right, other.right.iter()) {
                *e1 += e2;
            }
            for (e1, e2) in zip(&mut self.left, other.left.iter()) {
                *e1 += e2;
            }
        } else {
            if self.len() < index + other.len() {
                self.resize(index + other.len(), 0.0)
            }
            for i in 0..other.len() {
                self.right[i + index] += other.right[i];
            }
            for i in 0..other.len() {
                self.left[i + index] += other.left[i];
            }
        }
    }

    fn scale(&mut self, value: f64) {
        self.right = self.right.iter().map(|x| x * value).collect();
        self.left = self.left.iter().map(|x| x * value).collect()
    }

    fn scale_by_vec(&mut self, vec: Vec<f64>) {
        debug_assert_eq!(self.len(), vec.len(), "error in scale_by_vec");
        for (e1, e2) in zip(&mut self.right, vec.iter()) {
            *e1 *= e2;
        }
        for (e1, e2) in zip(&mut self.left, vec.into_iter()) {
            *e1 *= e2;
        }
    }

    fn len(&self) -> usize {
        self.right.len()
    }

    fn is_empty(&self) -> bool {
        self.right.is_empty()
    }

    fn normalize(&mut self) {
        todo!()
    }

    fn peak_normalize(&mut self) {
        let scale = 0.9
            / f64::max(
                utils::max_abs_f64(&self.right),
                utils::max_abs_f64(&self.left),
            );
        self.scale(scale)
    }

    fn save(&self, path: &Path) {
        let mut writer =
            hound::WavWriter::create(path, STD_SPEC).expect("Error while saving wave!");
        let mut writer_i16 = writer.get_i16_writer(self.len() as u32 * 2);
        let right = self.right.iter().map(|x| (x * i16::MAX as f64) as i16);
        let left = self.left.iter().map(|x| (x * i16::MAX as f64) as i16);
        for (r, l) in zip(right, left) {
            unsafe {
                writer_i16.write_sample_unchecked(r);
                writer_i16.write_sample_unchecked(l);
            }
        }
        writer_i16.flush().expect("Error while saving wave!");
        writer.finalize().expect("Error while saving wave!");
    }
}
