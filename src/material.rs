#![deny(clippy::perf, clippy::correctness, clippy::complexity, clippy::style, missing_debug_implementations)]
#![warn(clippy::pedantic)]

use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::vec3::Vec3;
use dyn_clone::{clone_trait_object, DynClone};
use std::borrow::Cow;
use std::fmt::Debug;

fn random_in_unit_sphere() -> Vec3 {
  let point_in_unit_sphere;
  loop {
    let potential_point_in_unit_sphere = 2.0 * Vec3::new(fastrand::f32(), fastrand::f32(), fastrand::f32()) - Vec3::new(1.0, 1.0, 1.0);
    if potential_point_in_unit_sphere.squared_length() < 1.0 {
      point_in_unit_sphere = potential_point_in_unit_sphere;
      break;
    }
  }
  point_in_unit_sphere
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScatterResult<'a, 'b, 'c> {
  attenuation: Cow<'a, Vec3>,
  scattered: Ray<'b, 'c>,
}

impl<'a, 'b, 'c> ScatterResult<'a, 'b, 'c> {
  pub fn new(attenuation: Cow<'a, Vec3>, scattered: Ray<'b, 'c>) -> Self {
    Self { attenuation, scattered }
  }
}

impl<'a, 'b, 'c> ScatterResult<'a, 'b, 'c> {
  pub fn attenuation(&self) -> &Cow<'a, Vec3> {
    &self.attenuation
  }
  pub fn scattered(&self) -> &Ray<'b, 'c> {
    &self.scattered
  }
}

pub trait Material: Debug + DynClone {
  fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterResult>;
}

clone_trait_object!(Material);

#[derive(Clone, Debug, PartialEq)]
pub struct Lambertian<'a> {
  albedo: Cow<'a, Vec3>,
}

#[allow(dead_code)]
impl<'a> Lambertian<'a> {
  pub fn new(albedo: Cow<'a, Vec3>) -> Self {
    Self { albedo }
  }
  pub fn albedo(&self) -> &Cow<'a, Vec3> {
    &self.albedo
  }
}

impl<'a> Material for Lambertian<'a> {
  fn scatter(&self, _: &Ray, hit_record: &HitRecord) -> Option<ScatterResult<'a, 'a, 'a>> {
    let target = *(*hit_record.point()) + *(*hit_record.normal()) + random_in_unit_sphere();
    let scattered = Ray::new((*(*hit_record.point())).into(), (target - *(*hit_record.point())).into());
    Some(ScatterResult::new((*self.albedo).into(), scattered))
  }
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
  v - 2.0 * v.dot(n) * n
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metal<'a> {
  albedo: Cow<'a, Vec3>,
  fuzz: f32,
}

#[allow(dead_code)]
impl<'a> Metal<'a> {
  pub fn new(albedo: Cow<'a, Vec3>, fuzz: f32) -> Self {
    let fuzz = if fuzz < 0.0 { 0.0 } else { fuzz };
    Self { albedo, fuzz }
  }
  pub fn albedo(&self) -> &Cow<'a, Vec3> {
    &self.albedo
  }
  pub fn fuzz(&self) -> f32 {
    self.fuzz
  }
}

impl<'m> Material for Metal<'m> {
  fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterResult> {
    let reflected = reflect(&ray_in.direction().unit_vector(), hit_record.normal());
    let scattered = Ray::new((**hit_record.point()).into(), (reflected + self.fuzz() * random_in_unit_sphere()).into());
    if scattered.direction().dot(hit_record.normal()) > 0.0 {
      Some(ScatterResult::new((*self.albedo).into(), scattered))
    } else {
      None
    }
  }
}

fn refract(v: &Vec3, n: &Vec3, ni_over_nt: f32) -> Option<Vec3> {
  let unit_vector = v.unit_vector();
  let dt = unit_vector.dot(n);
  let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
  if discriminant > 0.0 {
    Some(ni_over_nt * (unit_vector - n * dt) - n * discriminant.sqrt())
  } else {
    None
  }
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
  let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
  let r0 = r0 * r0;
  r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dielectric {
  ref_idx: f32,
}

impl Dielectric {
  pub fn new(ref_idx: f32) -> Self {
    Self { ref_idx }
  }
}

impl Material for Dielectric {
  fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterResult> {
    let reflected = reflect(ray_in.direction(), hit_record.normal());
    let attenuation = Vec3::new(1.0, 1.0, 1.0);
    let (outward_normal, ni_over_nt, cosine) = if ray_in.direction().dot(hit_record.normal()) > 0.0 {
      let cosine = self.ref_idx * ray_in.direction().dot(hit_record.normal()) / ray_in.direction().length();
      (-**hit_record.normal(), self.ref_idx, cosine)
    } else {
      let cosine = -ray_in.direction().dot(hit_record.normal()) / ray_in.direction().length();
      (**hit_record.normal(), 1.0 / self.ref_idx, cosine)
    };
    let refraction_result = refract(ray_in.direction(), &outward_normal, ni_over_nt);
    let reflect_probability = if refraction_result.is_some() {
      schlick(cosine, self.ref_idx)
    } else {
      1.0
    };
    let scattered = if fastrand::f32() < reflect_probability {
      Ray::new((**hit_record.point()).into(), reflected.into())
    } else {
      refraction_result.map_or_else(
        || unreachable!("Refraction not possible"),
        |refracted| Ray::new((**hit_record.point()).into(), refracted.into()),
      )
    };
    Some(ScatterResult::new(attenuation.into(), scattered))
  }
}