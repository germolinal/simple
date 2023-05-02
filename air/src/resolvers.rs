use crate::air_model::Resolver;
use crate::Float;
use std::sync::Arc;

use model::{Building, ShelterClass, Model, SimulationState, Space};

use crate::eplus::*;
use weather::CurrentWeather;

pub fn constant_resolver(space: &Arc<Space>, v: Float) -> Result<Resolver, String> {
    let space_clone = Arc::clone(space);
    Ok(Box::new(
        move |current_weather: &CurrentWeather, state: &mut SimulationState| {
            // Set temperature
            let outdoor_temperature = current_weather
                .dry_bulb_temperature;
            space_clone
                .set_infiltration_temperature(state, outdoor_temperature)
                .unwrap();

            // Set volume
            space_clone.set_infiltration_volume(state, v).unwrap();
        },
    ))
}

pub fn blast_resolver(space: &Arc<Space>, v: Float) -> Result<Resolver, String> {
    let space_clone = Arc::clone(space);
    Ok(Box::new(
        move |current_weather: &CurrentWeather, state: &mut SimulationState| {
            // Set temperature
            let outdoor_temperature = current_weather
                .dry_bulb_temperature;
            space_clone
                .set_infiltration_temperature(state, outdoor_temperature)
                .unwrap();

            // Set volume
            let volume = blast_design_flow_rate(current_weather, &space_clone, state, v);
            space_clone.set_infiltration_volume(state, volume).unwrap();
        },
    ))
}

pub fn doe2_resolver(space: &Arc<Space>, v: Float) -> Result<Resolver, String> {
    let space_clone = Arc::clone(space);
    Ok(Box::new(
        move |current_weather: &CurrentWeather, state: &mut SimulationState| {
            // Set temperature
            let outdoor_temperature = current_weather
                .dry_bulb_temperature;
            space_clone
                .set_infiltration_temperature(state, outdoor_temperature)
                .unwrap();

            // Set volume
            let volume = doe2_design_flow_rate(current_weather, &space_clone, state, v);
            space_clone.set_infiltration_volume(state, volume).unwrap();
        },
    ))
}

pub fn design_flow_rate_resolver(
    space: &Arc<Space>,
    a: Float,
    b: Float,
    c: Float,
    d: Float,
    v: Float,
) -> Result<Resolver, String> {
    let space_clone = Arc::clone(space);
    Ok(Box::new(
        move |current_weather: &CurrentWeather, state: &mut SimulationState| {
            // Set temperature
            let outdoor_temperature = current_weather
                .dry_bulb_temperature;
            space_clone
                .set_infiltration_temperature(state, outdoor_temperature)
                .unwrap();

            // Set volume
            let volume = design_flow_rate(current_weather, &space_clone, state, a, b, c, d, v);
            space_clone.set_infiltration_volume(state, volume).unwrap();
        },
    ))
}

fn resolve_stack_coefficient(space: &Arc<Space>, building: &Arc<Building>) -> Result<Float, String> {
    let cs = match building.stack_coefficient() {
        Ok(v)=>*v,
        Err(_)=>{
            match building.n_storeys(){
                Ok(storeys)=>{
                    let n_storeys = *storeys;
                    if n_storeys == 0{
                        return Err(format!("Building '{}' has 0 storeys", building.name));
                    }else if n_storeys == 1 {
                        0.000145
                    }else if n_storeys == 2 {
                        0.000290
                    }else if n_storeys == 3 {
                        0.000435
                    }else {
                        eprintln!("The Infiltration::EffectiveAirLeakageArea object (used in Space '{}') is appropriate for Building up to about 3 storeys... Building is {} storeys", space.name, storeys);
                        0.000435
                    }
                },
                Err(_)=>return Err(format!("Space '{}' has been assigned an Infiltration::EffectiveAirLeakageArea but its associated building has not enough data... Please assign values to the Building's stack_coefficient or n_storey fields", space.name))
            }
        }
    };
    Ok(cs)
}

fn resolve_wind_coefficient(space: &Arc<Space>, building: &Arc<Building>) -> Result<Float, String> {
    let cw = match building.wind_coefficient() {
        Ok(v) => *v,
        Err(_) => {
            let n_storeys = match building.n_storeys(){
                Ok(storeys)=>*storeys,
                Err(_)=>{return Err(format!("Building '{}', associated with Space '{}' has not been assigned an n_storeys field... Cannot resolve Wind Coefficient for EffectiveAirLeakageArea infiltration", building.name, space.name))}
            };
            match building.shelter_class(){
                Ok(shelter)=>{
                    match shelter{
                        ShelterClass::NoObstructions=>{
                            if n_storeys == 1 {
                                0.000319
                            }else if n_storeys == 2 {
                                0.000420
                            }else{
                                0.000494
                            }
                        },
                        ShelterClass::IsolatedRural=>{
                            if n_storeys == 1 {
                                0.000246
                            }else if n_storeys == 2 {
                                0.000325
                            }else{
                                0.000382
                            }
                        },
                        ShelterClass::Urban=>{
                            if n_storeys == 1 {
                                0.000172
                            }else if n_storeys == 2 {
                                0.000231
                            }else{
                                0.000271
                            }
                        },
                        ShelterClass::LargeLotUrban=>{
                            if n_storeys == 1 {
                                0.000104
                            }else if n_storeys == 2 {
                                0.000137
                            }else{
                                0.000161
                            }
                        },
                        ShelterClass::SmallLotUrban=>{
                            if n_storeys == 1 {
                                0.000032
                            }else if n_storeys == 2 {
                                0.000042
                            }else{
                                0.000049
                            }
                        },
                    }
                },
                Err(_)=>return Err(format!("Space '{}' has been assigned an Infiltration::EffectiveAirLeakageArea but its associated building has not enough data... Please assign values to the Building's wind_coefficient or shelter_class and n_storeys fields", space.name))
            }
        }
    };
    Ok(cw)
}

pub fn effective_air_leakage_resolver(
    space: &Arc<Space>,
    model: &Model,
    al: Float,
) -> Result<Resolver, String> {
    // We need data from the building.
    if let Ok(b_name) = space.building() {
        let building = model.get_building(b_name)?;
        let cs = resolve_stack_coefficient(space, &building)?;
        let cw = resolve_wind_coefficient(space, &building)?;

        let space_clone = Arc::clone(space);
        Ok(Box::new(
            move |current_weather: &CurrentWeather, state: &mut SimulationState| {
                // Set temperature
                let outdoor_temperature = current_weather
                    .dry_bulb_temperature;
                space_clone
                    .set_infiltration_temperature(state, outdoor_temperature)
                    .unwrap();

                // Set volume
                let volume =
                    effective_leakage_area(current_weather, &space_clone, state, al, cw, cs);
                space_clone.set_infiltration_volume(state, volume).unwrap();
            },
        ))
    } else {
        Err(format!("Space '{}' has been assigned an Infiltration::EffectiveAirLeakageArea but no building... Assign a Building to it.", space.name))
    }
}
