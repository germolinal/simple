/*
MIT License
Copyright (c) 2021 Germán Molina
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use crate::Float;

use crate::simulation_state_element::SimulationStateElement;
use std::ops::Index;

/// The structure containing the data that changes during the
/// simulation
pub type SimulationState = Vec<Float>;

/// The SimulationState is a Vector of SimulationStateElement objects.
/// It is intended to be a quick-to-clone structure.
///
/// To make it quicker to operate, the following conventions
/// are enforced:
/// * Personal elements go first, then Operational go second, and Physical go third. (This is checked when pushing elements to the state)
/// * Elements cannot be repeated (this is not really checked.).
#[derive(Debug, Clone)]
pub struct SimulationStateHeader {
    /// The number of operational StateElements
    /// in the State
    n_operational: usize,

    /// The number of individual StateElements
    /// in the State
    n_individual: usize,

    /// A flag indicating whether any physical
    /// element has been assigned. If it has, then
    /// no more individual elements are allowed. This
    /// allows maintaining (this is a convention, made in favour
    /// of performance) all the individual elements at the
    /// beginning, then all the operational ones, and then
    /// all the physical ones
    passed_operational: bool,

    /// A flag indicating whether a non-individual
    /// element has been assigned. If it has, then
    /// no more individual elements are allowed. This
    /// allows maintaining (this is a convention, made in favour
    /// of performance) all the individual elements at the
    /// beginning, then all the operational ones, and then
    /// all the physical ones
    passed_personal: bool,

    /// The elements in the SimulationState
    pub elements: Vec<SimulationStateElement>,

    /// The default values, from which the simulation will start
    pub default_values: Option<SimulationState>,
}

impl std::default::Default for SimulationStateHeader {
    fn default() -> Self {
        Self {
            n_operational: 0,
            n_individual: 0,
            passed_operational: false,
            passed_personal: false,
            elements: Vec::new(),
            default_values: Some(Vec::new()),
        }
    }
}

impl SimulationStateHeader {
    /// Creates a new empty Building State
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes an SimulationStateElement into the elements
    /// vector.
    ///
    /// It ensures that operational elements are there first,
    /// and that physical are there afterwards.
    ///
    /// Returns the index of the new element.
    pub fn push(&mut self, e: SimulationStateElement, value: Float) -> Result<usize, String> {
        if e.is_personal() {
            // If individual, check if we are passed that... if not, count
            // and add
            if self.passed_personal {
                return Err(format!("Trying to add an individual SimulationStateElement '{:?}' after no more were allowed", e));
            }
            self.n_individual += 1;
        } else if e.is_operational() {
            // If operational, check if we are passed that... if not, count
            // and add... mark individuals as passed
            if self.passed_operational {
                return Err(format!("Trying to add an operational SimulationStateElement '{:?}' after no more were allowed", e));
            }
            self.passed_personal = true;
            self.n_operational += 1;
        } else {
            // Else, mark both individual and operational as pased.
            // Just add them
            self.passed_operational = true;
            self.passed_personal = true;
        }

        let ret = self.elements.len();
        self.elements.push(e);
        if let Some(values) = &mut self.default_values {
            values.push(value);
        } else {
            return Err(
                "Trying to push a value to a SimulationStateHeader with no values vector"
                    .to_string(),
            );
        }
        // return
        Ok(ret)
    }

    /// Returns the number of elements in the
    /// State element
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Checks if the
    /// State element is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the number of operational elements
    pub fn n_operational(&self) -> usize {
        self.n_operational
    }

    /// Returns the number of operational elements
    pub fn n_individual(&self) -> usize {
        self.n_individual
    }

    /// Copies the whole state into another state
    pub fn copy_from(&mut self, origin: &SimulationStateHeader) {
        debug_assert_eq!(origin.len(), self.len());

        self.elements.copy_from_slice(origin.elements.as_slice())
    }

    /// Copies the Physical SimulationStateElements objects from `origin` to `destination`.
    /// It is expected that all the Operational BuildingStateElements are bundled at
    /// the beginning of the element.
    pub fn copy_physical_state(
        &mut self,
        origin: &SimulationState,
        destination: &mut SimulationState,
    ) {
        debug_assert_eq!(origin.len(), self.len());
        debug_assert_eq!(origin.len(), destination.len());

        let ini = self.n_individual + self.n_operational;
        let fin = self.len();

        #[cfg(debug_assertions)]
        for i in ini..fin {
            // Check that it is physical indeed
            debug_assert!(self[i].is_physical());
        }

        let origin_slice = origin.as_slice();
        destination[ini..fin].copy_from_slice(&origin_slice[ini..fin]);
    }

    /// Copies the Operational SimulationStateElements objects from origin to destination.
    /// It is expected that all the Operational BuildingStateElements are bundled at
    /// the beginning of the element.    
    pub fn copy_operational_state(
        &mut self,
        origin: &SimulationState,
        destination: &mut SimulationState,
    ) {
        debug_assert_eq!(origin.len(), self.len());
        debug_assert_eq!(origin.len(), destination.len());

        let ini = self.n_individual;
        let fin = self.n_individual + self.n_operational;

        #[cfg(debug_assertions)]
        for i in ini..fin {
            // Check that it is physical indeed
            debug_assert!(self[i].is_operational());
        }

        let origin_slice = origin.as_slice();
        destination[ini..fin].copy_from_slice(&origin_slice[ini..fin]);
    }

    /// Copies the Individual SimulationStateElements objects from origin to destination.
    /// It is expected that all the Operational BuildingStateElements are bundled at
    /// the beginning of the element.
    pub fn copy_individual_state(
        &mut self,
        origin: &SimulationState,
        destination: &mut SimulationState,
    ) {
        debug_assert_eq!(origin.len(), self.len());
        debug_assert_eq!(origin.len(), destination.len());

        let ini = 0;
        let fin = self.n_individual;

        #[cfg(debug_assertions)]
        for i in ini..fin {
            // Check that it is physical indeed
            debug_assert!(self[i].is_personal());
        }
        let origin_slice = origin.as_slice();
        destination[ini..fin].copy_from_slice(&origin_slice[ini..fin]);
    }

    /// Takes the `default_values` from the [`SimulationStateHeader`]. Leaves
    /// `None` in its place.
    pub fn take_values(&mut self) -> Option<SimulationState> {
        self.default_values.take()
    }
}

impl Index<usize> for SimulationStateHeader {
    type Output = SimulationStateElement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_new() {
        let state = SimulationStateHeader::new();
        assert_eq!(state.n_individual, 0);
        assert_eq!(state.passed_personal, false);
        assert_eq!(state.n_operational, 0);
        assert_eq!(state.passed_operational, false);
        assert_eq!(state.elements.len(), 0);
    }

    #[test]
    fn test_push() -> Result<(), String> {
        let mut state = SimulationStateHeader::new();
        assert_eq!(0, state.len());

        // Add one operational
        assert_eq!(
            0,
            state.push(SimulationStateElement::LuminairePowerConsumption(0), 1.0)?
        );
        assert_eq!(1, state.len());
        assert_eq!(1, state.n_operational());
        assert!(state.passed_personal);
        assert!(!state.passed_operational);
        if let Some(values) = &state.default_values {
            assert_eq!(values[0], 1.0);
        } else {
            assert!(false)
        }

        assert_eq!(
            1,
            state.push(SimulationStateElement::LuminairePowerConsumption(0), 12.0)?
        );
        assert_eq!(2, state.len());
        assert_eq!(2, state.n_operational());
        assert!(!state.passed_operational);
        if let Some(values) = &state.default_values {
            assert_eq!(values[1], 12.0);
        } else {
            assert!(false)
        }

        // push a physical one
        assert_eq!(
            2,
            state.push(SimulationStateElement::SpaceDryBulbTemperature(2), 2.)?
        );
        assert_eq!(3, state.len());
        assert_eq!(2, state.n_operational());
        assert!(state.passed_operational);
        if let Some(values) = &state.default_values {
            assert_eq!(values[2], 2.0);
        } else {
            assert!(false)
        }
        Ok(())
    }

    #[test]
    fn push_panic() {
        let mut state = SimulationStateHeader::new();

        // Add one operational
        assert!(state
            .push(SimulationStateElement::LuminairePowerConsumption(0), 1.0)
            .is_ok());

        // push a physical one
        assert!(state
            .push(SimulationStateElement::SpaceDryBulbTemperature(2), 2.)
            .is_ok());

        // Add an operational... it should panic now
        assert!(state
            .push(SimulationStateElement::LuminairePowerConsumption(0), 1.0)
            .is_err());
    }

    #[test]
    fn test_copy_operational_state() -> Result<(), String> {
        /* CREATE STATE 1 */
        let mut headers = SimulationStateHeader::new();

        // Add individual ones
        headers.push(SimulationStateElement::Clothing, 0.0)?;
        headers.push(SimulationStateElement::Clothing, 1.0)?;

        // Add operational ones
        headers.push(SimulationStateElement::LuminairePowerConsumption(1), 2.0)?;
        headers.push(SimulationStateElement::LuminairePowerConsumption(1), 3.0)?;

        // push physical ones
        headers.push(SimulationStateElement::SpaceDryBulbTemperature(1), 4.)?;
        headers.push(SimulationStateElement::SpaceDryBulbTemperature(1), 5.)?;

        let state1: SimulationState = headers.take_values().ok_or("Could not take values")?;
        let mut state2: SimulationState = vec![-1.; state1.len()];

        headers.copy_operational_state(&state1, &mut state2);

        // CHECK

        // Check that the individual states were untouched
        assert_eq!(state2[0], -1.);
        assert_eq!(state2[1], -1.);

        // Check that the two operational states were transferred
        assert_eq!(state2[2], 2.);
        assert_eq!(state2[3], 3.);

        // Check that the physical states were untouched
        assert_eq!(state2[4], -1.);
        assert_eq!(state2[5], -1.);

        Ok(())
    }

    #[test]
    fn test_copy_individual_state() -> Result<(), String> {
        /* CREATE STATE 1 */
        let mut headers = SimulationStateHeader::new();

        // Add individual ones
        headers.push(SimulationStateElement::Clothing, 0.0)?;
        headers.push(SimulationStateElement::Clothing, 1.0)?;

        // Add operational ones
        headers.push(SimulationStateElement::LuminairePowerConsumption(1), 2.0)?;
        headers.push(SimulationStateElement::LuminairePowerConsumption(1), 3.0)?;

        // push physical ones
        headers.push(SimulationStateElement::SpaceDryBulbTemperature(1), 4.)?;
        headers.push(SimulationStateElement::SpaceDryBulbTemperature(1), 5.)?;

        let state1: SimulationState = headers.take_values().ok_or("Could not take values")?;
        let mut state2: SimulationState = vec![-1.; state1.len()];

        headers.copy_individual_state(&state1, &mut state2);

        // CHECK

        // Check that the individual states were copied
        assert_eq!(state2[0], 0.);
        assert_eq!(state2[1], 1.);

        // Check that the two operational states were untouched
        assert_eq!(state2[2], -1.);
        assert_eq!(state2[3], -1.);

        // Check that the physical states were untouched
        assert_eq!(state2[4], -1.);
        assert_eq!(state2[5], -1.);
        Ok(())
    }

    #[test]
    fn test_copy_physical_state() -> Result<(), String> {
        /* CREATE STATE 1 */
        let mut headers = SimulationStateHeader::new();

        // Add individual ones
        headers.push(SimulationStateElement::Clothing, 0.0)?;
        headers.push(SimulationStateElement::Clothing, 1.0)?;

        // Add operational ones
        headers.push(SimulationStateElement::LuminairePowerConsumption(1), 2.0)?;
        headers.push(SimulationStateElement::LuminairePowerConsumption(1), 3.0)?;

        // push physical ones
        headers.push(SimulationStateElement::SpaceDryBulbTemperature(1), 4.)?;
        headers.push(SimulationStateElement::SpaceDryBulbTemperature(1), 5.)?;

        let state1: SimulationState = headers.take_values().ok_or("Could not take values")?;
        let mut state2: SimulationState = vec![-1.; state1.len()];

        headers.copy_physical_state(&state1, &mut state2);

        // CHECK

        // Check that the individual states were untouched
        assert_eq!(state2[0], -1.);
        assert_eq!(state2[1], -1.);

        // Check that the two operational states were untouched
        assert_eq!(state2[2], -1.);
        assert_eq!(state2[3], -1.);

        // Check that the physical states were copied
        assert_eq!(state2[4], 4.);
        assert_eq!(state2[5], 5.);
        Ok(())
    }
}
