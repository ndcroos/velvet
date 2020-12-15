//! This example executes a simulation of 108 Ar atoms in the NVE ensemble.
//! After sucessful completion, a figure is generated at `nve.png` which plots the total energy at each timestep.

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use indicatif::{ProgressBar, ProgressStyle};
use plotters::prelude::*;

use std::fs::File;
use std::io::BufReader;

use velvet::convert::load_poscar;
use velvet::core::distributions::{Boltzmann, VelocityDistribution};
use velvet::core::integrators::{Integrator, VelocityVerlet};
use velvet::core::potentials::pair::{LennardJones, PairPotentialMeta};
use velvet::core::potentials::{Potentials, Restriction};
use velvet::core::properties::{Property, TotalEnergy};
use velvet::core::system::elements::Element;

static TIMESTEPS: u64 = 250000;
static PLOT_INTERVAL: u64 = 10;
static FILENAME: &'static str = "assets/nve.png";

fn main() {
    pretty_env_logger::init();
    info!("Starting a NVE simulation of Ar gas...");

    // Load the Ar gas system directly from a POSCAR formatted file.
    let file = File::open("resources/test/argon.poscar").unwrap();
    let reader = BufReader::new(file);
    let mut system = load_poscar(reader);

    // Setup an initial velocity distribution with a target temperature
    let boltz = Boltzmann::new(300 as f32);

    // Apply the initialized velocity distribution to the system.
    boltz.apply(&mut system);

    // Define a Lennard-Jones style pair potential.
    let lj = LennardJones::new(1.0, 3.4);

    // Define some metadata about the potential.
    // - The element pair which it applies to.
    // - The cutoff radius.
    // - Any additional restrictions (intermolecular/intramolecular...)
    let meta = PairPotentialMeta::new((Element::Ar, Element::Ar), 8.5, Restriction::None);

    // Initialize a collection of potentials and add the previously defined pair potential with metadata.
    let mut potentials = Potentials::new();
    potentials.add_pair(Box::new(lj), meta);

    // Define a velocity Verlet style integrator.
    let mut velocity_verlet = VelocityVerlet::new(1.0);
    velocity_verlet.setup(&system, &potentials);

    // Setup a progress bar to track the simulation.
    let progress = get_progress_bar(TIMESTEPS);

    let mut energy_results: Vec<(u64, f64)> =
        Vec::with_capacity((TIMESTEPS / PLOT_INTERVAL) as usize);

    // Integrate for N timesteps.
    for i in 0..TIMESTEPS {
        velocity_verlet.integrate(&mut system, &potentials);
        if i % PLOT_INTERVAL == 0 {
            energy_results.push((i, TotalEnergy.calculate(&system, &potentials) as f64));
        }
        progress.inc(1);
    }

    progress.finish();

    info!("Simulation completed successfully.");

    // Plot the energy results
    plot_results(energy_results);

    info!("Generated summary figure: `{}`", FILENAME);
}

fn get_progress_bar(len: u64) -> ProgressBar {
    let progress = ProgressBar::new(len);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green} {pos:>6}/{len} timesteps"),
    );
    progress
}

fn plot_results(data: Vec<(u64, f64)>) {
    let root_area = BitMapBackend::new(FILENAME, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .margin(10)
        .margin_right(30)
        .build_cartesian_2d(0..TIMESTEPS, -250.6..-250.5)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Timestep")
        .x_label_style(("sans-serif", 18))
        .y_desc("Total Energy (kJ/mol)")
        .y_label_style(("sans-serif", 18))
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(data.into_iter(), &BLUE))
        .unwrap();
}
