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
                HVAC::IdealHeaterCooler(hvac) => {                       
                    if let Ok(space_name) = hvac.target_space(){
                        
                        let space : std::sync::Arc<model::Space> = model.borrow().get_space(space_name)?;                        
                        let space_temp = space.dry_bulb_temperature(state).ok_or("Could not get IdealHeaterCooler's target_space temperature ")?;

                        // Deal with heating
                        let heating_setpoint = hvac.heating_setpoint();
                        let max_heating = hvac.max_heating_power();
                        let mut has_heating = false;
                        if let (Ok(setpoint), Ok(power)) = ( heating_setpoint, max_heating){                            
                            has_heating = true;
                            if space_temp < *setpoint{
                                hvac.set_heating_cooling_consumption(state, *power)?;
                            }else{
                                hvac.set_heating_cooling_consumption(state, 0.0)?;
                            }
                        }  

                        // Deal with cooling
                        let cooling_setpoint = hvac.cooling_setpoint();
                        let max_cooling = hvac.max_cooling_power();
                        if let (Ok(setpoint), Ok(power)) = ( cooling_setpoint, max_cooling){                            
                            if space_temp > *setpoint{
                                hvac.set_heating_cooling_consumption(state, -*power)?;
                            }else if !has_heating {
                                hvac.set_heating_cooling_consumption(state, 0.0)?;
                            }
                            
                        }  

                    }                    
                },
            }
        }
        Ok(())        
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    

    use communication::{SimulationModel, MetaOptions};
    use model::Space;
    use model::hvac::{ElectricHeater, IdealHeaterCooler};
    use validate::assert_close;

    use super::*;

    use crate::model::Model;
    

    #[test]
    fn test_control_electric_heater() {

        let mut model = Model::default();

        // add a space
        let mut space = Space::new("The space");
        space.set_volume(212.0);
        let space = model.add_space(space);

        // Ad a heater
        let heating_setpoint = 19.0;
        let max_heating_power = 2400.0;
        let heater_name = "Heater".to_string();        
        let mut heater = ElectricHeater::new(heater_name.clone());
        heater.set_heating_setpoint(heating_setpoint)
            .set_target_space(space.name().clone())
            .set_max_heating_power(max_heating_power);
        let hvac = model.add_hvac(heater.wrap()).unwrap();

        // Get state
        let mut state_header = model.take_state().unwrap();        

        // Create a model... we don't use it, but we need it as an input.
        let meta_option = MetaOptions::default();        
        let physics_model = MultiphysicsModel::new(&meta_option, (), &model, &mut state_header, 1).unwrap();
        
        let mut state = state_header.take_values().unwrap();
        let controller = OccupantBehaviour::new(&model).unwrap();
        
        // Emulate previous data
        if let HVAC::ElectricHeater(heater) = &hvac {
            heater.set_heating_cooling_consumption(&mut state, 129081298.0).unwrap()
        }else{
            panic!("as")
        }

        // Test 1: temp is below setpoint
        space.set_dry_bulb_temperature(&mut state, heating_setpoint - 0.1).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::ElectricHeater(heater) = &hvac {
            assert_close!(max_heating_power, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        // Test 1: temp is now above
        space.set_dry_bulb_temperature(&mut state, heating_setpoint + 0.1).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::ElectricHeater(heater) = &hvac {
            assert_close!(0.0, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        
    }

    #[test]
    fn test_control_ideal_heating_cooling() {

        let mut model = Model::default();

        // add a space
        let mut space = Space::new("The space");
        space.set_volume(212.0);
        let space = model.add_space(space);

        // Ad a heater
        let heating_setpoint = 19.0;
        let cooling_setpoint = 25.0;
        let max_heating_power = 2400.0;
        let max_cooling_power = 1400.0;
        let heater_name = "Heater".to_string();        
        let mut heater = IdealHeaterCooler::new(heater_name.clone());
        heater.set_heating_setpoint(heating_setpoint)
            .set_cooling_setpoint(cooling_setpoint)
            .set_target_space(space.name().clone())
            .set_max_heating_power(max_heating_power)
            .set_max_cooling_power(max_cooling_power);

        
        let hvac = model.add_hvac(heater.wrap()).unwrap();

        // Get state
        let mut state_header = model.take_state().unwrap();        

        // Create a model... we don't use it, but we need it as an input.
        let meta_option = MetaOptions::default();        
        let physics_model = MultiphysicsModel::new(&meta_option, (), &model, &mut state_header, 1).unwrap();
        
        let mut state = state_header.take_values().unwrap();
        let controller = OccupantBehaviour::new(&model).unwrap();
        
        // Emulate previous data
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            heater.set_heating_cooling_consumption(&mut state, 129081298.0).unwrap()
        }else{
            panic!("as")
        }

        // Test 1: temp is below heating setpoint
        space.set_dry_bulb_temperature(&mut state, heating_setpoint - 0.1).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(max_heating_power, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        // Test 2: temp is between heating and cooling setpoints
        space.set_dry_bulb_temperature(&mut state, heating_setpoint/2.0 + cooling_setpoint/2.0).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(0.0, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        // Test 3: temp is above cooling setpoint
        space.set_dry_bulb_temperature(&mut state, cooling_setpoint + 2.0).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(-max_cooling_power, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        
    }

    #[test]
    fn test_control_ideal_heating_cooling_no_heating() {

        let mut model = Model::default();

        // add a space
        let mut space = Space::new("The space");
        space.set_volume(212.0);
        let space = model.add_space(space);

        // Ad a heater        
        let cooling_setpoint = 25.0;
        let max_cooling_power = 1400.0;
        let heater_name = "Heater".to_string();        
        let mut heater = IdealHeaterCooler::new(heater_name.clone());
        heater.set_cooling_setpoint(cooling_setpoint)
            .set_target_space(space.name().clone())            
            .set_max_cooling_power(max_cooling_power);
        let hvac = model.add_hvac(heater.wrap()).unwrap();

        // Get state
        let mut state_header = model.take_state().unwrap();        

        // Create a model... we don't use it, but we need it as an input.
        let meta_option = MetaOptions::default();        
        let physics_model = MultiphysicsModel::new(&meta_option, (), &model, &mut state_header, 1).unwrap();
        
        let mut state = state_header.take_values().unwrap();
        let controller = OccupantBehaviour::new(&model).unwrap();
        

        // Emulate previous data
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            heater.set_heating_cooling_consumption(&mut state, 129081298.0).unwrap()
        }else{
            panic!("as")
        }

        // Test 1: temp is above cooling setpoint
        space.set_dry_bulb_temperature(&mut state, cooling_setpoint + 0.1).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(-max_cooling_power, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        

        // Test 2: temp is below cooling setpoint
        space.set_dry_bulb_temperature(&mut state, cooling_setpoint - 2.0).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(0.0, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        
    }

    #[test]
    fn test_control_ideal_heating_cooling_no_cooling() {

        let mut model = Model::default();

        // add a space
        let mut space = Space::new("The space");
        space.set_volume(212.0);
        let space = model.add_space(space);

        // Ad a heater        
        let heating_setpoint = 25.0;
        let max_heating_power = 1400.0;
        let heater_name = "Heater".to_string();        
        let mut heater = IdealHeaterCooler::new(heater_name.clone());
        heater.set_heating_setpoint(heating_setpoint)
            .set_target_space(space.name().clone())            
            .set_max_heating_power(max_heating_power);
        let hvac = model.add_hvac(heater.wrap()).unwrap();

        // Get state
        let mut state_header = model.take_state().unwrap();        

        // Create a model... we don't use it, but we need it as an input.
        let meta_option = MetaOptions::default();        
        let physics_model = MultiphysicsModel::new(&meta_option, (), &model, &mut state_header, 1).unwrap();
        
        let mut state = state_header.take_values().unwrap();
        let controller = OccupantBehaviour::new(&model).unwrap();
        

        // Emulate previous data
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            heater.set_heating_cooling_consumption(&mut state, 129081298.0).unwrap()
        }else{
            panic!("as")
        }
        
        // Test 1: temp is below heating setpoint
        space.set_dry_bulb_temperature(&mut state, heating_setpoint - 0.1).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(max_heating_power, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        

        // Test 2: temp is above heating setpoint
        space.set_dry_bulb_temperature(&mut state, heating_setpoint + 2.0).unwrap();
        controller.control(&model, &physics_model, &mut state).unwrap();
        if let HVAC::IdealHeaterCooler(heater) = &hvac {
            assert_close!(0.0, heater.heating_cooling_consumption(&state).unwrap())
        }else{
            panic!("as")
        }

        
    }
}