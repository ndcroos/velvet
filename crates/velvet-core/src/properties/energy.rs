//! Types of energy that can be evaluated.

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::internal::Float;
use crate::potentials::Potentials;
use crate::properties::{IntrinsicProperty, Property};
use crate::system::System;

/// Potential energy due to Coulombic potentials.
#[derive(Clone, Copy, Debug)]
pub struct CoulombicEnergy;

impl Property for CoulombicEnergy {
    type Res = Float;

    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Res {
        let coulomb_potentials = &potentials.coulomb_potentials.potentials;
        let selections = &potentials.coulomb_potentials.selections;
        let cutoffs = &potentials.coulomb_potentials.cutoffs;

        coulomb_potentials
            .iter()
            .zip(selections.iter())
            .zip(cutoffs.iter())
            .map(|((pot, select), &cut)| -> Float {
                select
                    .indices()
                    .map(|[i, j]| {
                        let pos_i = &system.positions[*i];
                        let qi = &system.particle_types[system.particle_type_map[*i]].charge();
                        let pos_j = &system.positions[*j];
                        let qj = &system.particle_types[system.particle_type_map[*j]].charge();
                        let r = system.cell.distance(&pos_i, &pos_j);
                        if r < cut {
                            pot.energy(*qi, *qj, r)
                        } else {
                            0.0 as Float
                        }
                    })
                    .sum()
            })
            .sum()
    }

    fn name(&self) -> String {
        "coulombic_energy".to_string()
    }
}


/// Potential energy due to pairwise potentials.
#[derive(Clone, Copy, Debug)]
pub struct PairEnergy;

impl Property for PairEnergy {
    type Res = Float;

    #[cfg(not(feature = "rayon"))]
    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Res {
        let pair_potentials = &potentials.pair_potentials.potentials;
        let selections = &potentials.pair_potentials.selections;
        let cutoffs = &potentials.pair_potentials.cutoffs;

        pair_potentials
            .iter()
            .zip(selections.iter())
            .zip(cutoffs.iter())
            .map(|((pot, select), &cut)| -> Float {
                select
                    .indices()
                    .map(|[i, j]| {
                        let pos_i = &system.positions[*i];
                        let pos_j = &system.positions[*j];
                        let r = system.cell.distance(&pos_i, &pos_j);
                        if r < cut {
                            pot.energy(r)
                        } else {
                            0.0 as Float
                        }
                    })
                    .sum()
            })
            .sum()
    }

    #[cfg(feature = "rayon")]
    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Res {
        let pair_potentials = &potentials.pair_potentials.potentials;
        let selections = &potentials.pair_potentials.selections;
        let cutoffs = &potentials.pair_potentials.cutoffs;

        pair_potentials
            .iter()
            .zip(selections.iter())
            .zip(cutoffs.iter())
            .map(|((pot, select), &cut)| -> Float {
                select
                    .par_indices()
                    .map(|[i, j]| {
                        let pos_i = &system.positions[*i];
                        let pos_j = &system.positions[*j];
                        let r = system.cell.distance(&pos_i, &pos_j);
                        if r < cut {
                            pot.energy(r)
                        } else {
                            0.0 as Float
                        }
                    })
                    .sum()
            })
            .sum()
    }

    fn name(&self) -> String {
        "pair_energy".to_string()
    }
}

/// Potential energy of the whole system.
#[derive(Clone, Copy, Debug)]
pub struct PotentialEnergy;

impl Property for PotentialEnergy {
    type Res = Float;

    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Res {
        let coulomb_energy = CoulombicEnergy.calculate(system, potentials);
        let pair_energy = PairEnergy.calculate(system, potentials);
        coulomb_energy + pair_energy
    }

    fn name(&self) -> String {
        "potential_energy".to_string()
    }
}

/// Kinetic energy of the whole system
#[derive(Clone, Copy, Debug)]
pub struct KineticEnergy;

impl IntrinsicProperty for KineticEnergy {
    type Res = Float;

    // NOTE: This function is faster without rayon.
    fn calculate_intrinsic(&self, system: &System) -> <Self as IntrinsicProperty>::Res {
        let kinetic_energy: Float = system
            .particle_type_map
            .iter()
            .zip(system.velocities.iter())
            .map(|(idx, vel)| {
                let pt = system.particle_types[*idx];
                0.5 * pt.mass() * vel.norm_squared()
            })
            .sum();
        kinetic_energy
    }

    fn name(&self) -> String {
        "kinetic_energy".to_string()
    }
}

/// Sum of potential and kinetic energy.
#[derive(Clone, Copy, Debug)]
pub struct TotalEnergy;

impl Property for TotalEnergy {
    type Res = Float;

    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Res {
        let kinetic = KineticEnergy.calculate(system, potentials);
        let potential = PotentialEnergy.calculate(system, potentials);
        kinetic + potential
    }

    fn name(&self) -> String {
        "total_energy".to_string()
    }
}
