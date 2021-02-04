#![deny(clippy::perf, clippy::correctness, clippy::complexity, clippy::style, missing_debug_implementations)]
#![warn(clippy::pedantic)]

use crate::vec3::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub struct Ray {
  origin: Vec3,
  direction: Vec3,
}

#[allow(dead_code)]
impl Ray {
  pub fn new(origin: &Vec3, direction: &Vec3) -> Self {
    Self { origin: *origin, direction: *direction }
  }
  pub fn origin(&self) -> &Vec3 {
    &self.origin
  }
  pub fn direction(&self) -> &Vec3 {
    &self.direction
  }
  pub fn point_at_parameter(&self, scalar_length: f32) -> Vec3 {
    self.origin + scalar_length * self.direction
  }
}
