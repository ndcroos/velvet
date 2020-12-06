//! Physical properties of the simulated system.

use nalgebra::Vector3;

use crate::potential::{Potentials, Restriction};
use crate::system::System;

/// Calculates a system-wide property.
pub trait Property {
    /// The property's return type.
    type Output;
    /// Returns a physical property of the system.
    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Output;
}

/// Force acting on each atom in the system.
#[derive(Clone, Copy, Debug)]
pub struct Forces;

impl Property for Forces {
    type Output = Vec<Vector3<f32>>;

    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Output {
        let sys_size = system.size();
        let mut forces: Vec<Vector3<f32>> = vec![Vector3::new(0.0, 0.0, 0.0); sys_size];

        // iterate over all pairs of atoms
        for i in 0..sys_size {
            // skip duplicate or identical pairs
            for j in (i + 1)..sys_size {
                // calculate distance between the pair
                let pos1 = &system.positions[i];
                let pos2 = &system.positions[j];
                let r = system.cell.distance(pos1, pos2);

                // iterate over the pair potentials
                for (potential, meta) in potentials.pairs() {
                    // check cutoff radius
                    if meta.cutoff < r {
                        continue;
                    }

                    // check element pair
                    let elem1 = &system.elements[i];
                    let elem2 = &system.elements[j];
                    if (*elem1, *elem2) != meta.elements {
                        continue;
                    }

                    // check restricton
                    let ok = match meta.restriction {
                        Restriction::None => true,
                        Restriction::Intermolecular => &system.molecules[i] != &system.molecules[j],
                        Restriction::Intramolecular => &system.molecules[i] == &system.molecules[j],
                    };
                    if ok {
                        let dir = &system.cell.direction(pos1, pos2);
                        let force = potential.force(r) * dir;
                        forces[i] += force;
                        forces[j] -= force;
                    }
                }
            }
        }
        forces
    }
}

/// Potential energy of the whole system.
#[derive(Clone, Copy, Debug)]
pub struct PotentialEnergy;

impl Property for PotentialEnergy {
    type Output = f32;

    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Output {
        let sys_size = system.size();
        let mut potential_energy: f32 = 0.0 as f32;

        // iterate over all pairs of atoms
        for i in 0..sys_size {
            // skip duplicate or identical pairs
            for j in (i + 1)..sys_size {
                // calculate distance between the pair
                let pos1 = &system.positions[i];
                let pos2 = &system.positions[j];
                let r = system.cell.distance(pos1, pos2);

                // iterate over the pair potentials
                for (potential, meta) in potentials.pairs() {
                    // check cutoff radius
                    if meta.cutoff < r {
                        continue;
                    }

                    // check element pair
                    let elem1 = &system.elements[i];
                    let elem2 = &system.elements[j];
                    if (*elem1, *elem2) != meta.elements {
                        continue;
                    }

                    // check restricton
                    let ok = match meta.restriction {
                        Restriction::None => true,
                        Restriction::Intermolecular => &system.molecules[i] != &system.molecules[j],
                        Restriction::Intramolecular => &system.molecules[i] == &system.molecules[j],
                    };
                    if ok {
                        potential_energy += potential.energy(r);
                    }
                }
            }
        }
        potential_energy
    }
}

/// Kinetic energy of the whole system
#[derive(Clone, Copy, Debug)]
pub struct KineticEnergy;

impl Property for KineticEnergy {
    type Output = f32;

    fn calculate(&self, system: &System, _: &Potentials) -> Self::Output {
        let sys_size = system.size();
        let mut kinetic_energy = 0.0 as f32;

        for i in 0..sys_size {
            kinetic_energy += 0.5 * system.masses[i] * system.velocities[i].norm_squared();
        }
        kinetic_energy
    }
}

/// Total energy of the system.
#[derive(Clone, Copy, Debug)]
pub struct TotalEnergy;

impl Property for TotalEnergy {
    type Output = f32;

    fn calculate(&self, system: &System, potentials: &Potentials) -> Self::Output {
        let kinetic = KineticEnergy.calculate(system, potentials);
        let potential = PotentialEnergy.calculate(system, potentials);
        kinetic + potential
    }
}

#[cfg(test)]
mod tests {
    use crate::potential::pair::{Harmonic, PairPotentialMeta};
    use crate::potential::{Potentials, Restriction};
    use crate::property::{Forces, KineticEnergy, PotentialEnergy, Property, TotalEnergy};
    use crate::system::{cell::Cell, element::Element, System};
    use approx::*;
    use nalgebra::Vector3;

    fn get_pair_system() -> System {
        let size = 2 as usize;
        let fluorine = Element::F;
        let mut sys = System::new(size);
        sys.cell = Cell::new(10.0, 10.0, 10.0, 90.0, 90.0, 90.0);
        sys.elements = vec![fluorine, fluorine];
        sys.molecules = vec![0 as usize, 0 as usize];
        sys.positions = vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.3, 0.0, 0.0)];
        sys.velocities = vec![
            Vector3::new(
                -0.007225222699367925,
                -0.002405756495275919,
                0.0026065109398392215,
            ),
            Vector3::new(
                0.001179633958023287,
                0.003525262341736351,
                -0.0004132774783154952,
            ),
        ];
        sys.masses = vec![fluorine.mass(), fluorine.mass()];
        sys.charges = vec![0.0, 0.0];
        sys
    }

    fn get_pair_potentials() -> Potentials {
        let mut pots = Potentials::new();
        let potential = Box::new(Harmonic::new(300.0, 1.2));
        let meta = PairPotentialMeta::new((Element::F, Element::F), 5.0, Restriction::None);
        pots.add_pair(potential, meta);
        pots
    }

    #[test]
    fn forces() {
        // define the system
        let sys = get_pair_system();

        // define the potentials
        let pots = get_pair_potentials();

        // calculate the forces
        let forces = Forces.calculate(&sys, &pots);
        let total_force = forces[0] + forces[1];
        assert_relative_eq!(total_force.norm(), 0.0);

        let target_force = 30.0 as f32;
        assert_relative_eq!(forces[0][0], -target_force, epsilon = 1e-4);
        assert_relative_eq!(forces[0][1], 0.0);
        assert_relative_eq!(forces[0][2], 0.0);

        assert_relative_eq!(forces[1][0], target_force, epsilon = 1e-4);
        assert_relative_eq!(forces[1][1], 0.0);
        assert_relative_eq!(forces[1][2], 0.0);
    }

    #[test]
    fn energy() {
        // define the system
        let sys = get_pair_system();

        // define the potentials
        let pots = get_pair_potentials();

        // calculate the energies
        let kinetic = KineticEnergy.calculate(&sys, &pots);
        let potential = PotentialEnergy.calculate(&sys, &pots);
        let total = TotalEnergy.calculate(&sys, &pots);

        assert_eq!(kinetic + potential, total);
        assert_relative_eq!(kinetic, 0.0007483);
    }
}
