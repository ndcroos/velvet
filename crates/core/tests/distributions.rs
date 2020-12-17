mod common;

use approx::*;

use std::fs::File;

use velvet_core::system::System;
use velvet_core::distributions::{Boltzmann, VelocityDistribution};
use velvet_core::properties::{IntrinsicProperty, Temperature};

#[test]
fn boltzmann() {
    // load system
    let path = common::test_resources_path("argon.sys.velvet");
    let file = File::open(&path).unwrap();
    let mut system: System = ron::de::from_reader(file).unwrap();

    let target = 1000 as f32;
    let boltz = Boltzmann::new(target);
    boltz.apply(&mut system);
    let res = Temperature.calculate_intrinsic(&system);
    assert_relative_eq!(res, target, epsilon = 1e-3);
}
