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

use derive::StateElements;
use std::sync::{Arc, Mutex};

/// The type used for storing the fields of structures that represent 
/// values stored in the `SimulationState`.
pub type StateElementField = Arc<Mutex<Option<usize>>>;

/// The idea is to have a cheap-to-clone (or copy?) structure
#[derive(Debug, Copy, Clone, PartialEq, Eq, StateElements)]
pub enum SimulationStateElement {
    /* PERSONAL ELEMENTS */
    /// The amount of clothing the person is using,
    /// in Clo value
    #[personal]
    Clothing,

    /* OPERATION AND OCCUPATION */
    /// Represents how open is a fenestration.
    /// Contains the Index of fenestration, and its open fraction
    #[operational]
    #[references("Fenestration")]
    FenestrationOpenFraction(usize),

    /// Represents the heating/cooling energy consumption of a Heating/Cooling system,
    /// in Watts
    ///
    /// Contains the index of the HVAC in the building's vector,
    /// and the power.        
    #[operational]
    #[references("HVAC")]
    HeatingCoolingPowerConsumption(usize),

    /// Represents the power being consumed by
    /// a Luminaire object, in Watts (luminaire index, power)
    #[operational]
    #[references("Luminaire")]
    LuminairePowerConsumption(usize),

    /* SOLAR */
    // Space
    //SpaceTotalSolarHeatGain(usize),
    //SpaceDirectSolarHeatGain(usize),
    //SpaceDiffuseSolarHeatGain(usize),
    // /// Represents the Brightness of a space.
    // ///
    // /// This perception is a placeholder. I need to
    // /// understand better what makes a space look "bright"
    // /// and how that relates to its attractiveness and
    // /// cleanliness and all.
    // ///
    // /// **This is written as a perception for now,
    // /// but it should be a physical quantity**
    // #[physical]
    // SpaceBrightness(usize),
    /// The convective heat transfer coefficient
    /// at the front of a surface
    #[physical]
    #[references("Surface")]
    SurfaceFrontConvectionCoefficient(usize),

    /// The convective heat transfer coefficient
    /// at the back of a surface
    #[physical]
    #[references("Surface")]
    SurfaceBackConvectionCoefficient(usize),

    /// The convective heat flow
    /// at the front of a surface
    #[physical]
    #[references("Surface")]
    SurfaceFrontConvectiveHeatFlow(usize),

    /// The convective heat flow
    /// at the back of a surface
    #[physical]
    #[references("Surface")]
    SurfaceBackConvectiveHeatFlow(usize),

    /// Incident solar irradiance at the front
    #[physical]
    #[references("Surface")]
    SurfaceFrontSolarIrradiance(usize),

    /// Incident solar irradiance at the back
    #[physical]
    #[references("Surface")]
    SurfaceBackSolarIrradiance(usize),

    /// Incident Infrared irradiance at the front
    #[physical]
    #[references("Surface")]
    SurfaceFrontIRIrradiance(usize),

    /// Incident Infrared irradiance at the back
    #[physical]
    #[references("Surface")]
    SurfaceBackIRIrradiance(usize),

    /// The convective heat transfer coefficient
    /// at the front of a surface
    #[physical]
    #[references("Fenestration")]
    FenestrationFrontConvectionCoefficient(usize),

    /// The convective heat transfer coefficient
    /// at the back of a surface
    #[physical]
    #[references("Fenestration")]
    FenestrationBackConvectionCoefficient(usize),

    /// The convective heat flow
    /// at the front of a surface
    #[physical]
    #[references("Fenestration")]
    FenestrationFrontConvectiveHeatFlow(usize),

    /// The convective heat flow
    /// at the back of a surface
    #[physical]
    #[references("Fenestration")]
    FenestrationBackConvectiveHeatFlow(usize),

    /// Incident solar irradiance at the front
    #[physical]
    #[references("Fenestration")]
    FenestrationFrontSolarIrradiance(usize),

    /// Incident solar irradiance at the back
    #[physical]
    #[references("Fenestration")]
    FenestrationBackSolarIrradiance(usize),

    /// Incident Infrared irradiance at the front
    #[physical]
    #[references("Fenestration")]
    FenestrationFrontIRIrradiance(usize),

    /// Incident Infrared irradiance at the back
    #[physical]
    #[references("Fenestration")]
    FenestrationBackIRIrradiance(usize),

    /// Space Air Temperature in C... The elements
    /// are the index of the Space in the Building mode
    /// and the temperature
    #[physical]
    #[references("Space")]
    SpaceDryBulbTemperature(usize),

    /// The volume of air that is entering the space in
    /// an uncontrolled way. In m3/s
    #[physical]
    #[references("Space")]
    SpaceInfiltrationVolume(usize),

    /// The temperature of air that is entering the space in
    /// an uncontrolled way. In C
    #[physical]
    #[references("Space")]
    SpaceInfiltrationTemperature(usize),

    /// The volume of air that is entering the space in
    /// a controlled way. In m3/s
    #[physical]
    #[references("Space")]
    SpaceVentilationVolume(usize),

    /// The temperature of air that is entering the space in
    /// a controlled way. In C
    #[physical]
    #[references("Space")]
    SpaceVentilationTemperature(usize),

    /// The volume of air that is moving from one space to another in
    /// a controlled way. In m3/s
    #[physical]
    #[references("Space, Space")]
    SpaceAirExchangeVolume(usize, usize),

    /// Temperature (Float) of Surface's (usize) node (usize)
    /// I.e. the order is (Surface Index, Node index, Temperature).    
    #[physical]
    #[references("Surface, Number")]
    SurfaceNodeTemperature(usize, usize),

    /// Temperature (Float) of Fenestration's (usize) node (usize)
    /// I.e. the order is (Surface Index, Node index, Temperature).    
    #[physical]
    #[references("Fenestration, Number")]
    FenestrationNodeTemperature(usize, usize),
    // Temperature (Float) of Fenestation's (usize) node usize
    // I.e. the order is (Surface Index, Node index, Temperature).
    //FenestrationNodeTemperature(usize,usize),

    // Fenestration

    // Shading

    //

    /* ACOUSTIC */
    // Space
    // /// Represents the loudness in a certain space
    // ///
    // /// **This is written as a perception for now,
    // /// but it should be a physical quantity**
    // #[physical]
    // SpaceLoudness(usize),
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {

    use super::*;
    // use crate::scanner::SimpleScanner;
    use crate::{Model, Space};

    #[test]
    fn test_output_dual_argument() {
        let mut model = Model::default();

        let space_a = Space::new("Space A");
        let _space_a = model.add_space(space_a);
        let space_b = Space::new("Space B");
        let _space_b = model.add_space(space_b);

        let element = SimulationStateElement::SpaceAirExchangeVolume(0, 1);
        assert_eq!(
            element.stringify(&model),
            "{\"SpaceAirExchangeVolume\":\"Space A-Space B\"}"
        )

        // assert!(false)
    }

    #[test]
    fn test_compare() {
        let i = 2;
        let a = SimulationStateElement::SpaceDryBulbTemperature(i);

        assert!(a == SimulationStateElement::SpaceDryBulbTemperature(i));
        assert!(a != SimulationStateElement::SpaceDryBulbTemperature(2 * i));
        assert!(a != SimulationStateElement::SurfaceNodeTemperature(i, 2));
    }

    #[test]
    fn test_classify() {
        // Physical
        let e = SimulationStateElement::SpaceDryBulbTemperature(2);
        assert!(e.is_physical());
        assert!(!e.is_operational());
        assert!(!e.is_personal());

        // Individual
        let e = SimulationStateElement::Clothing;
        assert!(!e.is_physical());
        assert!(!e.is_operational());
        assert!(e.is_personal());

        // Operational
        let e = SimulationStateElement::HeatingCoolingPowerConsumption(2);
        assert!(!e.is_physical());
        assert!(e.is_operational());
        assert!(!e.is_personal());
    }

    #[test]
    fn output_from_file() {
        let (model, ..) = Model::from_file("./tests/box.spl").unwrap();
        assert_eq!(model.outputs.len(), 1);
        assert_eq!(model.spaces.len(), 1);
        let space_name = model.spaces[0].name();

        let state_element = SimulationStateElement::SpaceDryBulbTemperature(0);
        let str1 = state_element.stringify(&model);
        let str2 = "{\"SpaceDryBulbTemperature\":\"Bedroom\"}";
        assert_eq!(str1, str2);
        assert!(str1.contains(space_name));
    }

    #[test]
    fn test_output_compare() {
        // are comparisons based on the pointer of the string?
        let s1 = Output::SpaceDryBulbTemperature("Some Space".into());
        let s2 = Output::SpaceDryBulbTemperature("Some Space".into());

        assert_eq!(s1, s2);

        // are comparisons based on the pointer of the string?
        let s1 = Output::SpaceDryBulbTemperature("Some other Space".into());
        let s2 = Output::SpaceDryBulbTemperature("Some Space".into());

        assert_ne!(s1, s2);
    }

    use crate::scanner::SimpleScanner;
    #[test]
    fn test_output() {
        let src = b"
            Output { SpaceDryBulbTemperature : \"The Space\" }
            Output { SpaceDryBulbTemperature : \"The Other Space\" }
            Output { SpaceDryBulbTemperature : \"Yet another Space\" }
        ";
        let mut scanner = SimpleScanner::new(src, 0);
        let (model, header) = scanner.parse_model().unwrap();
        assert_eq!(model.outputs.len(), 3);
        assert_eq!(header.len(), 0);
        dbg!(&model.outputs);

        dbg!(serde_json::to_string(&model.outputs[0]).unwrap());
    }
}
