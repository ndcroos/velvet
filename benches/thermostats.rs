use std::fs::File;
use std::io::BufReader;

use criterion::{criterion_group, criterion_main, Criterion};

use velvet_core::thermostats::{Berendsen, NoseHoover, Thermostat};
use velvet_core::velocity_distributions::{Boltzmann, VelocityDistribution};
use velvet_external_data::poscar::load_poscar;
use velvet_test_utils as test_utils;

pub fn thermostats_group_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("thermostats");

    let file = File::open(test_utils::resources_path("Ar.poscar")).unwrap();
    let reader = BufReader::new(file);
    let mut system = load_poscar(reader);

    let target = 100.0;
    let boltz = Boltzmann::new(target);
    boltz.apply(&mut system);

    let mut berendsen = Berendsen::new(target, 2.0);
    berendsen.setup(&system);

    let mut nose = NoseHoover::new(target, 1.5, 1.0);
    nose.setup(&system);

    group.bench_function("berendsen", |b| {
        b.iter(|| {
            berendsen.pre_integrate(&mut system);
            berendsen.post_integrate(&mut system);
        })
    });

    group.bench_function("nose_hoover", |b| {
        b.iter(|| {
            nose.pre_integrate(&mut system);
            nose.post_integrate(&mut system);
        })
    });

    group.finish()
}

criterion_group!(thermostats, thermostats_group_benchmark);
criterion_main!(thermostats);
