#![allow(dead_code)]

use java_random::{Random, LCG};
use std::collections::HashMap;
use intmap::IntMap;
use crate::noise::Noise;
use crate::math::lfloor;

pub const SKIP_262: LCG = LCG::combine_java(262);

#[derive(Clone)]
pub struct PerlinNoise {
    lacunarity: f64,
    persistence: f64,
    noise_octaves: Vec<Option<Noise>>,
    cache2d: IntMap<f64>,
    cache3d: HashMap<u128, f64>,
}


#[cfg(test)]
mod perlin_test {
    use super::*;
    use crate::create_range;

    #[test]
    fn test_coordinates() {
        let perlin=PerlinNoise::new(Random::with_seed(1),create_range(1,2));
        let value=perlin.sample_default(0f64,0f64,0f64);
        assert_eq!(value,-0.20402661037924066f64)
    }
}

impl PerlinNoise {
    pub fn new(mut random: Random, octaves: Vec<i32>) -> PerlinNoise {
        if octaves.is_empty() {
            panic!("No octaves defined")
        }
        let start = -*octaves.first().expect("Missing an element");
        let end = *octaves.last().expect("Missing an element");
        let length = start + end + 1;
        if length < 1 {
            panic!("You need at least one octave")
        }
        let noise: Noise = Noise::init(random);
        let mut noise_octaves: Vec<Option<Noise>> = vec![None; 5];

        if end >= 0 && end < length && octaves.contains(&0) {
            noise_octaves[end as usize] = Option::from(noise);
        }
        for i in end + 1..length {
            if i >= 0 && octaves.contains(&(end - i)) {
                noise_octaves[i as usize] = Option::from(Noise::init(random));
                continue;
            }
            random.advance(SKIP_262);
        }
        if end > 0 {
            let noise_seed: i64 = (noise.get_noise_value(0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64) * 9.223372036854776E18) as i64;
            random = Random::with_seed(noise_seed as u64);
            for i in end - 1..=0 {
                if i < length && octaves.contains(&(end - i)) {
                    noise_octaves[i as usize] = Option::from(Noise::init(random));
                    continue;
                }
                random.advance(SKIP_262);
            }
        }
        let persistence: f64 = 2f64.powi(end);
        let lacunarity: f64 = 1.0f64 / (2f64.powi(length) - 1.0);
        PerlinNoise {
            lacunarity,
            persistence,
            noise_octaves,
            cache2d: IntMap::with_capacity(1024),
            cache3d: Default::default(),
        }
    }

    pub fn sample_default(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sample(x, y, z, 0.0f64, 0.0f64, false)
    }

    pub fn sample(&self, x: f64, y: f64, z: f64, y_amplification: f64, y_min: f64, use_default_y: bool) -> f64 {
        let mut noise_value: f64 = 0.0f64;
        // contribution of each octaves to the final noise, diminished by a factor of 2 (or increased by factor of 0.5)
        let mut persistence: f64 = self.persistence;
        // distance between octaves, increased for each by a factor of 2
        let mut lacunarity: f64 = self.lacunarity;

        for sampler in &self.noise_octaves {
            if let Some(noise)=sampler {

                noise_value += noise.get_noise_value(
                    Self::wrap(x * persistence),
                    if use_default_y { -noise.y0 } else { Self::wrap(y * persistence) },
                    Self::wrap(z * persistence),
                    y_amplification * persistence,
                    y_min * persistence) * lacunarity;
            }

            persistence /= 2.0f64;
            lacunarity *= 2.0f64;
        }

        return noise_value;
    }

    pub fn sample_surface(&self, x: f64, z: f64,y_amplification: f64, y_min: f64)->f64{
        self.sample(x,0.0f64,z,y_amplification,y_min,false)
    }

    fn wrap(x: f64) -> f64 {
        return x - (lfloor(x / 3.3554432E7 + 0.5) as f64) * 3.3554432E7;
    }
}
