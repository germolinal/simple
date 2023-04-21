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

use model::{Model, SimulationState, HVAC};

use crate::control_trait::SimpleControl;
use crate::MultiphysicsModel;
use std::borrow::Borrow;

/// A relatively simple control algorithm aiming to represent a quite
/// rational building control.
///
/// For now it:
///
/// * Turns the heating/cooling systems in each zone depending on the tempreature of the space where its thermostat is located.
pub struct OccupantBehaviour {}

impl OccupantBehaviour {
    
    /// Creates a 
    pub fn new(_model: &Model)->Result<Self, String>{
        Ok(Self{})
    }
}

impl SimpleControl for OccupantBehaviour {

    
    fn control<M: Borrow<Model>>(&self, model: M, _physics_model: &MultiphysicsModel, state: &mut SimulationState)->Result<(), String>{
        // Switch HVACs on/off
        for hvac in model.borrow().hvacs.iter() {
            match hvac {
                HVAC::ElectricHeater(hvac)=> {
                    let target_space = hvac.target_space();
                    let heating_setpoint = hvac.heating_setpoint();
                    let max_consumption = hvac.max_heating_power();
                    if let (Ok(space_name), Ok(setpoint), Ok(power)) = (target_space, heating_setpoint, max_consumption){
                        let space : std::sync::Arc<model::Space> = model.borrow().get_space(space_name)?;                        
                        let space_temp = space.dry_bulb_temperature(state).ok_or("Could not get ElectricHeater's target_space temperature ")?;
                        if space_temp < *setpoint{
                            hvac.set_heating_cooling_consumption(state, *power)?;
                        }else{
                            hvac.set_heating_cooling_consumption(state, 0.0)?;
                        }
                    }                    
                },
                HVAC::IdealHeaterCooler(_hvac) => {
                    todo!()
                },
            }
        }
        Ok(())        
    }
}
