use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::internal::Float;
use crate::system::elements::Element;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Specie {
    id: u128,
    mass: Float,
    charge: Float,
}

impl Specie {
    pub fn new(mass: Float, charge: Float) -> Specie {
        Specie { id: Uuid::new_v4().as_u128(), mass, charge }
    }

    pub fn from_element(element: Element) -> Specie {
        Specie {
            id: element.number() as u128,
            mass: element.mass(),
            charge: element.charge(),
        }
    }

    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn mass(&self) -> Float {
        self.mass
    }

    pub fn charge(&self) -> Float {
        self.charge
    }
}

impl PartialEq for Specie {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::Specie;
    use crate::system::elements::Element;

    #[test]
    fn from_element() {
        let element = Element::H;
        let specie = Specie::from_element(element);
        assert_eq!(specie.mass(), element.mass());
        assert_eq!(specie.charge(), element.charge());
        assert_eq!(specie.id(), element.number() as u128);
    }

    #[test]
    fn compare_equivalent() {
        let hydrogen1 = Specie::from_element(Element::H);
        let hydrogen2 = Specie::from_element(Element::H);
        assert_eq!(hydrogen1, hydrogen2);
    }

    #[test]
    fn compare_nonequivalent() {
        let hydrogen = Specie::from_element(Element::H);
        let helium = Specie::from_element(Element::He);
        assert_ne!(hydrogen, helium);
        let specie = Specie::new(hydrogen.mass(), hydrogen.charge());
        assert_ne!(specie, hydrogen);
    }
}
