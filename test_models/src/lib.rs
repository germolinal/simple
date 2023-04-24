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

#![deny(missing_docs)]

//! Functions for creating small SIMPLE models

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

#[cfg(not(feature = "float"))]
type Float = f64;

use geometry::{Loop3D, Point3D, Polygon3D};

use model::{
    hvac::ElectricHeater,
    substance::{gas::GasSpecification, Gas, Normal as NormalSubstance},
    Boundary, Construction, Fenestration, Infiltration, Luminaire, Material, Model,
    SimulationStateHeader, Space, Surface,
};

/// The test material
pub enum TestMat {
    /// A Concrete with a certain `Float` thickness   
    ///
    /// # Properties
    /// * density: 1700.
    /// * Specific heat: 800.
    /// * Thermal Cond.: 0.816
    /// * emissivity: From `options.emissivity`
    Concrete(Float),

    /// A Polyurethane with a certain `Float` thickness
    ///
    /// # Properties
    /// * density: 17.5
    /// * Specific heat: 2400.
    /// * Thermal Cond.: 0.0252
    /// * emissivity: From `options.emissivity
    Polyurethane(Float),

    /// A Glass material defined based on the thikness and solar_transmittance
    ///
    /// # Properties
    /// * density: 2.5
    /// * Specific heat: 840.
    /// * Thermal Cond.: 1.0
    /// * emissivity: From `options.emissivity
    Glass(Float, Float),

    /// Air Cavity
    Air(Float),
}

/// Characteristics of the Zone of the single-zone model
pub struct SingleZoneTestBuildingOptions {
    /// Volume, in m3
    pub zone_volume: Float,

    /// The construction, built out of [`TestMat`]
    pub construction: Vec<TestMat>, // Explicitly mentioned

    /// The surface width
    pub surface_width: Float,

    /// The surface height
    pub surface_height: Float,

    /// The window area, will be subtracted from the surface area
    pub window_width: Float,

    /// The window height
    pub window_height: Float,

    /// The power of the heating in the zone, in W
    pub heating_power: Float,

    /// The power of the lighting in the wall, in m3.
    pub lighting_power: Float,

    /// The infiltration rate, in m3/s
    pub infiltration_rate: Float,

    /// The emissivity of the substances (assigned to all)
    pub emissivity: Float,

    /// The solar absorbtance of the substances, assigned to all
    pub solar_absorbtance: Float,

    /// In degrees. When 0, the exterior points South
    pub orientation: Float,
}

impl Default for SingleZoneTestBuildingOptions {
    fn default() -> SingleZoneTestBuildingOptions {
        SingleZoneTestBuildingOptions {
            zone_volume: -1., // Will be checked... negative numbers panic
            construction: Vec::with_capacity(0),
            surface_width: -1.,  // Will be checked... negative numbers panic
            surface_height: -1., // Will be checked... negative numbers panic
            window_width: 0.,
            window_height: 0.,
            heating_power: 0.,
            lighting_power: 0.,
            infiltration_rate: 0.,
            emissivity: 0.84,
            solar_absorbtance: 0.7,
            orientation: 0.0,
        }
    }
}


/// Adds a luminare to the model
pub fn add_luminaire(model: &mut Model, options: &SingleZoneTestBuildingOptions) {
    let power = options.lighting_power;
    assert!(power > 0.);
    let mut luminaire = Luminaire::new("the luminaire");
    luminaire.set_max_power(power as Float);
    luminaire.set_target_space(model.spaces[0].name());
    model.add_luminaire(luminaire).unwrap();
}

/// Adds a heater to the model
pub fn add_heater(model: &mut Model, options: &SingleZoneTestBuildingOptions) {
    let power = options.heating_power;
    assert!(power > 0.);
    let mut hvac = ElectricHeater::new("some hvac");
    hvac.set_target_space(model.spaces[0].name());
    model.add_hvac(hvac.wrap()).unwrap();
}

/// A single space model with a single surface (optionally) one operable window that has the same construction
/// as the rest of the walls. Thw front of the surface faces South.
///
/// The surface_area includes the window; the window_area is cut down from it.
pub fn get_single_zone_test_building(
    options: &SingleZoneTestBuildingOptions,
) -> (Model, SimulationStateHeader) {
    let mut model = Model::default();

    /*************** */
    /* ADD THE SPACE */
    /*************** */
    let zone_volume = options.zone_volume;
    assert!(
        zone_volume > 0.0,
        "A positive zone_volume parameter is required (Float)"
    );

    let mut space = Space::new("Some space".to_string());
    space.set_volume(zone_volume);

    /*********************** */
    /* ADD INFILTRATION, IF NEEDED */
    /*********************** */
    if options.infiltration_rate > 0.0 {
        let infiltration_rate = options.infiltration_rate;
        assert!(infiltration_rate > 0.);
        let infiltration = Infiltration::Constant {
            flow: infiltration_rate,
        };
        space.set_infiltration(infiltration);
    }

    // .set_importance(Box::new(ScheduleConstant::new(1.0)));
    let space = model.add_space(space);

    /******************* */
    /* ADD THE SUBSTANCE */
    /******************* */

    // Add both substances
    let mut concrete = NormalSubstance::new("concrete".to_string());
    concrete
        .set_density(1700.)
        .set_specific_heat_capacity(800.)
        .set_thermal_conductivity(0.816)
        .set_front_thermal_absorbtance(options.emissivity)
        .set_back_thermal_absorbtance(options.emissivity)
        .set_front_solar_absorbtance(options.solar_absorbtance)
        .set_back_solar_absorbtance(options.solar_absorbtance);
    let concrete = model.add_substance(concrete.wrap());

    let mut polyurethane = NormalSubstance::new("polyurethane".to_string());
    polyurethane
        .set_density(17.5)
        .set_specific_heat_capacity(2400.)
        .set_thermal_conductivity(0.0252)
        .set_front_thermal_absorbtance(options.emissivity)
        .set_back_thermal_absorbtance(options.emissivity)
        .set_front_solar_absorbtance(options.solar_absorbtance)
        .set_back_solar_absorbtance(options.solar_absorbtance);

    let polyurethane = model.add_substance(polyurethane.wrap());

    let mut air = Gas::new("some_gas".to_string());
    air.set_gas(GasSpecification::Air);
    let air = model.add_substance(air.wrap());

    /*********************************** */
    /* ADD THE MATERIAL AND CONSTRUCTION */
    /*********************************** */
    let mut construction = Construction::new("the construction");
    for (i, c) in options.construction.iter().enumerate() {
        let material = match c {
            TestMat::Concrete(thickness) => Material::new(
                format!("Material {}", i),
                concrete.name().clone(),
                *thickness,
            ),
            TestMat::Polyurethane(thickness) => Material::new(
                format!("Material {}", i),
                polyurethane.name().clone(),
                *thickness,
            ),
            TestMat::Glass(thickness, solar_transmittance) => {
                let mut glass = NormalSubstance::new("polyurethane");
                glass
                    .set_density(2.5)
                    .set_specific_heat_capacity(840.)
                    .set_thermal_conductivity(1.)
                    .set_front_thermal_absorbtance(options.emissivity)
                    .set_back_thermal_absorbtance(options.emissivity)
                    .set_front_solar_absorbtance(options.solar_absorbtance)
                    .set_back_solar_absorbtance(options.solar_absorbtance)
                    .set_solar_transmittance(*solar_transmittance);
                let glass = model.add_substance(glass.wrap());
                Material::new(format!("Material {}", i), glass.name().clone(), *thickness)
            }
            TestMat::Air(thickness) => {
                Material::new(format!("Material {}", i), air.name().clone(), *thickness)
            }
        };
        let material = model.add_material(material);
        construction.materials.push(material.name().clone());
    }
    let construction = model.add_construction(construction);

    /****************** */
    /* SURFACE GEOMETRY */
    /****************** */
    // Wall
    assert!(
        options.surface_width > 0.0 && options.surface_height > 0.0,
        "A positive surface_area option is needed (Float)"
    );

    let l = options.surface_width / 2.;
    let mut the_loop = Loop3D::new();
    let angle = options.orientation.to_radians();

    the_loop
        .push(Point3D::new(-l * angle.cos(), -l * angle.sin(), 0.))
        .unwrap();
    the_loop
        .push(Point3D::new(l * angle.cos(), l * angle.sin(), 0.))
        .unwrap();
    the_loop
        .push(Point3D::new(
            l * angle.cos(),
            l * angle.sin(),
            options.surface_height,
        ))
        .unwrap();
    the_loop
        .push(Point3D::new(
            -l * angle.cos(),
            -l * angle.sin(),
            options.surface_height,
        ))
        .unwrap();
    the_loop.close().unwrap();

    let mut p = Polygon3D::new(the_loop).unwrap();

    // Window... if there is any
    let mut window_polygon: Option<Polygon3D> = None;
    if options.window_width > 0.0 && options.window_height > 0.0 {
        assert!(
            options.window_width * options.window_height
                < options.surface_width * options.surface_height,
            "Win_area >= Surface_area"
        );

        let l = options.window_width / 2.;
        let mut the_inner_loop = Loop3D::new();
        the_inner_loop
            .push(Point3D::new(
                -l * angle.cos(),
                -l * angle.sin(),
                options.surface_height / 2. - options.window_height / 2.,
            ))
            .unwrap();
        the_inner_loop
            .push(Point3D::new(
                l * angle.cos(),
                l * angle.sin(),
                options.surface_height / 2. - options.window_height / 2.,
            ))
            .unwrap();
        the_inner_loop
            .push(Point3D::new(
                l * angle.cos(),
                l * angle.sin(),
                options.surface_height / 2. + options.window_height / 2.,
            ))
            .unwrap();
        the_inner_loop
            .push(Point3D::new(
                -l * angle.cos(),
                -l * angle.sin(),
                options.surface_height / 2. + options.window_height / 2.,
            ))
            .unwrap();
        the_inner_loop.close().unwrap();
        p.cut_hole(the_inner_loop.clone()).unwrap();
        window_polygon = Some(Polygon3D::new(the_inner_loop).unwrap());
    }

    /***************** */
    /* ACTUAL SURFACES */
    /***************** */
    // Add surface
    let surface = Surface::new(
        "Surface".to_string(),
        p,
        construction.name().clone(),
        Boundary::Outdoor,
        Boundary::Space {
            space: space.name().clone(),
        },
    );

    model.add_surface(surface);

    // Add window.
    if let Some(window_polygon) = window_polygon {
        let fenestration = Fenestration::new(
            "window one".to_string(),
            window_polygon,
            construction.name().clone(),
            Boundary::Space {
                space: space.name().clone(),
            },
            Boundary::Outdoor,
        );

        model.add_fenestration(fenestration).unwrap();
    }

    /*********************** */
    /* ADD HEATER, IF NEEDED */
    /*********************** */
    if options.heating_power > 0.0 {
        add_heater(&mut model, options);
    }

    /*********************** */
    /* ADD LIGHTS, IF NEEDED */
    /*********************** */
    if options.lighting_power > 0.0 {
        add_luminaire(&mut model, options);
    }

    // Return
    let header = model.take_state().unwrap();
    (model, header)
}

#[cfg(test)]
mod testing {

    use super::*;

    #[test]
    fn test_with_window() {
        let surface_width = 2.;
        let surface_height = 2.;
        let window_width = 1.;
        let window_height = 1.;
        let zone_volume = 40.;

        let (model, _state_header) = get_single_zone_test_building(
            // &mut state,
            &SingleZoneTestBuildingOptions {
                zone_volume,
                surface_height,
                surface_width,
                window_height,
                window_width,
                construction: vec![TestMat::Concrete(0.2)],
                ..Default::default()
            },
        );

        let surf_area = model.surfaces[0].area();
        let exp_area = surface_width * surface_height - window_height * window_width;
        assert!(
            (surf_area - exp_area).abs() < 1e-3,
            "area = {}... expecting {}",
            surf_area,
            exp_area
        );
    }

    #[test]
    fn test_no_window() {
        let surface_width = 2.;
        let surface_height = 2.;
        let window_width = 0.;
        let window_height = 0.;
        let zone_volume = 40.;

        let (model, _state_header) = get_single_zone_test_building(
            // &mut state,
            &SingleZoneTestBuildingOptions {
                zone_volume,
                surface_height,
                surface_width,
                window_height,
                window_width,
                construction: vec![TestMat::Concrete(0.2)],
                ..Default::default()
            },
        );

        let surf_area = model.surfaces[0].area();
        let exp_area = surface_width * surface_height;
        assert!(
            (surf_area - exp_area).abs() < 1e-3,
            "area = {}... expecting {}",
            surf_area,
            exp_area
        );
    }
}
