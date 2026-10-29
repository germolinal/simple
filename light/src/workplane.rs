/*
MIT License
Copyright (c)  Germán Molina
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

//! Daylight illuminance on workplanes: horizontal grids of sensors placed at
//! a certain height over the floor of a space.
//!
//! Contrary to the solar calculations (which use a single bounce, for
//! comparability with EnergyPlus), the daylight coefficients of the sensors
//! account for interreflections, which dominate the illuminance indoors.

use crate::Float;
use crate::dc_matrix::DCMatrix;
use crate::solar_model::solar_from_meta_options;
use calendar::Date;
use communication::MetaOptions;
use geometry::{Point3D, Polygon3D, Ray3D, Vector3D};
use model::{Boundary, Model, Workplane};
use rendering::colour_matrix::colour_matrix_to_radiance;
use rendering::{DCFactory, Scene, Wavelengths};
use weather::{PerezSky, ReinhartSky, SkyUnits, Solar, WeatherTrait};

const MODULE_NAME: &str = "Workplane";

/// The luminous efficacy (lm/W) used to convert visible irradiance into
/// illuminance (as in Radiance)
const WHITE_EFFICACY: Float = 179.;

/// The ground reflectance used when building the sky (as in the solar model)
const ALBEDO: Float = 0.2;

/// Surfaces whose normal is within ~25 degrees of the vertical are
/// considered horizontal (i.e., can be floors)
const MIN_FLOOR_COS: Float = 0.9;

/// The default height of workplanes over the floor, in m
pub const DEFAULT_HEIGHT: Float = 0.7;

/// The default distance between the sensors of a workplane, in m
pub const DEFAULT_SPACING: Float = 0.5;

/// The sensors of a [`Workplane`]: a grid of
/// upward-facing illuminance sensors, at a certain height over the floor of a
/// space.
///
/// The sensors are placed on a regular grid (aligned with the X and Y axes)
/// rather than on a mesh, as usual in daylight simulation: every sensor
/// represents the same area, and the placement is robust to any floor
/// shape.
pub struct WorkplaneSensors {
    /// The name of the workplane
    pub name: String,

    /// The name of the space
    pub space: String,

    /// The position of each sensor
    pub sensors: Vec<Point3D>,

    /// The floor area represented by each sensor (in m2)
    pub areas: Vec<Float>,

    /// The daylight coefficients of the sensors (one row per sensor)
    dc: DCMatrix,
}

impl WorkplaneSensors {
    /// The horizontal surfaces whose upward-facing side faces `space`
    /// (i.e., the same criteria as [`Model::get_space_sizes`], but only for
    /// horizontal surfaces)
    fn floors<'a>(model: &'a Model, space: &str) -> Vec<&'a Polygon3D> {
        model
            .surfaces
            .iter()
            .filter_map(|s| {
                let normal = s.vertices.normal();
                // the boundary at the side facing up
                let up = if normal.z > 0.0 {
                    &s.front_boundary
                } else {
                    &s.back_boundary
                };
                match up {
                    Boundary::Space { space: name }
                        if name == space && normal.z.abs() >= MIN_FLOOR_COS =>
                    {
                        Some(&s.vertices)
                    }
                    _ => None,
                }
            })
            .collect()
    }

    /// Places the sensors over the floors of a space, returning their
    /// positions and the floor area each of them represents
    pub fn place_sensors(
        model: &Model,
        space: &str,
        height: Float,
        spacing: Float,
    ) -> Result<(Vec<Point3D>, Vec<Float>), String> {
        model.get_space(space)?;
        if spacing <= 0.0 {
            return Err(format!(
                "The spacing between sensors should be positive... found {spacing}"
            ));
        }
        let floors = Self::floors(model, space);
        if floors.is_empty() {
            return Err(format!("Space '{space}' has no floor"));
        }

        let lift = Vector3D::new(0., 0., height);
        let mut sensors = Vec::new();
        let mut areas = Vec::new();
        for floor in floors {
            // The plane of the floor, for placing the points on it (floors
            // are not necessarily perfectly horizontal)
            let normal = floor.normal();
            let origin = floor.outer().vertices()[0];
            let z_at = |x: Float, y: Float| -> Float {
                origin.z - (normal.x * (x - origin.x) + normal.y * (y - origin.y)) / normal.z
            };
            let bbox = floor.outer().bbox()?;
            let nx = ((bbox.max.x - bbox.min.x) / spacing).ceil() as usize;
            let ny = ((bbox.max.y - bbox.min.y) / spacing).ceil() as usize;
            // Center the grid on the bounding box
            let x0 = (bbox.min.x + bbox.max.x - (nx as Float - 1.) * spacing) / 2.;
            let y0 = (bbox.min.y + bbox.max.y - (ny as Float - 1.) * spacing) / 2.;
            let first = sensors.len();
            for i in 0..nx {
                for j in 0..ny {
                    let x = x0 + i as Float * spacing;
                    let y = y0 + j as Float * spacing;
                    let p = Point3D::new(x, y, z_at(x, y));
                    if floor.test_point(p)? {
                        sensors.push(p + lift);
                    }
                }
            }
            // The floor is split evenly among its sensors (cells at the
            // edges are partly outside of it, so it is not spacing^2)
            let n = sensors.len() - first;
            areas.extend(std::iter::repeat_n(floor.area() / n as Float, n));
        }
        if sensors.is_empty() {
            return Err(format!(
                "Space '{space}' is too small for placing sensors {spacing}m apart"
            ));
        }
        Ok((sensors, areas))
    }

    /// Places the sensors of a [`Workplane`], calculating
    /// their daylight coefficients. The `scene` should be built with
    /// [`Wavelengths::Visible`], and `reinhart` should be the sky with which
    /// the workplane will be used.
    pub fn new(
        model: &Model,
        scene: &Scene,
        reinhart: ReinhartSky,
        workplane: &Workplane,
    ) -> Result<Self, String> {
        let height = workplane.height_or(MODULE_NAME, DEFAULT_HEIGHT);
        let spacing = workplane.spacing_or(MODULE_NAME, DEFAULT_SPACING);
        let n_ambient_samples =
            workplane.n_ambient_samples_or(MODULE_NAME, DCFactory::default().n_ambient_samples);
        let (sensors, areas) = Self::place_sensors(model, &workplane.space, height, spacing)
            .map_err(|e| format!("Workplane '{}': {e}", workplane.name))?;

        let up = Vector3D::new(0., 0., 1.);
        let rays: Vec<Ray3D> = sensors
            .iter()
            .map(|origin| Ray3D {
                origin: *origin,
                direction: up,
            })
            .collect();

        let factory = DCFactory {
            reinhart,
            n_ambient_samples,
            ..DCFactory::default()
        };
        let dc = factory.calc_dc(&rays, scene, None);
        let dc = DCMatrix::from(&colour_matrix_to_radiance(&dc));

        Ok(Self {
            name: workplane.name.clone(),
            space: workplane.space.clone(),
            sensors,
            areas,
            dc,
        })
    }

    /// Calculates the illuminance (in lux) at each sensor, given a sky vector
    /// in [`SkyUnits::Visible`]
    pub fn illuminance_into(&self, sky_vec: &[Float], lux: &mut [Float]) -> Result<(), String> {
        self.dc.mul_vec_into(sky_vec, lux)?;
        for v in lux.iter_mut() {
            *v = (*v * WHITE_EFFICACY).max(0.0);
        }
        Ok(())
    }
}

/// Calculates the daylight illuminance over the [`Workplane`]s of a model in
/// each timestep
pub struct WorkplaneModel {
    solar: Solar,
    sky: ReinhartSky,
    workplanes: Vec<WorkplaneSensors>,
}

/// The memory used by a [`WorkplaneModel`]: the sky vector, and the latest
/// illuminance at each [`Workplane`] (in the same order as in the
/// model)
pub struct WorkplaneModelMemory {
    sky_vec: Vec<Float>,
    illuminance: Vec<Vec<Float>>,
}

impl WorkplaneModelMemory {
    /// The latest illuminance (in lux) at each sensor of a [`Workplane`]
    pub fn illuminance(&self, workplane: usize) -> Option<&[Float]> {
        self.illuminance.get(workplane).map(|v| v.as_slice())
    }
}

impl WorkplaneModel {
    /// The default sky discretization (i.e., the 145 Tregenza patches, as
    /// the default of the solar model)
    pub const DEFAULT_MF: usize = 1;

    /// Places the sensors of every [`Workplane`] in the model, and calculates
    /// their daylight coefficients, discretizing the sky with Reinhart's
    /// subdivision `mf` (`1` means the 145 Tregenza patches)
    pub fn new(meta_options: &MetaOptions, model: &Model, mf: usize) -> Result<Self, String> {
        let workplanes = if model.workplanes.is_empty() {
            Vec::new()
        } else {
            let mut scene = Scene::from_simple_model(model, Wavelengths::Visible)?;
            scene.build_accelerator();
            model
                .workplanes
                .iter()
                .map(|w| WorkplaneSensors::new(model, &scene, ReinhartSky::new(mf), w))
                .collect::<Result<Vec<_>, String>>()?
        };
        Ok(Self {
            solar: solar_from_meta_options(meta_options),
            sky: ReinhartSky::new(mf),
            workplanes,
        })
    }

    /// The sensors of each workplane (in the same order as in the model)
    pub fn workplanes(&self) -> &[WorkplaneSensors] {
        &self.workplanes
    }

    /// Allocates the memory needed for calculating the illuminance
    pub fn allocate_memory(&self) -> WorkplaneModelMemory {
        WorkplaneModelMemory {
            sky_vec: vec![0.0; self.sky.n_bins],
            illuminance: self
                .workplanes
                .iter()
                .map(|w| vec![0.0; w.sensors.len()])
                .collect(),
        }
    }

    /// Calculates the illuminance at every workplane on `date`
    pub fn march<W: WeatherTrait>(
        &self,
        date: Date,
        weather: &W,
        memory: &mut WorkplaneModelMemory,
    ) -> Result<(), String> {
        if memory.illuminance.len() != self.workplanes.len() {
            return Err("The memory was allocated for a different number of workplanes".into());
        }
        let weather_data = weather.get_weather_data(date);
        let is_day = weather_data.direct_normal_radiation
            + weather_data.diffuse_horizontal_radiation
            >= 1e-4;
        if !is_day {
            for lux in memory.illuminance.iter_mut() {
                lux.iter_mut().for_each(|v| *v = 0.0);
            }
            return Ok(());
        }

        PerezSky::update_sky_vec_for(
            &self.sky,
            &mut memory.sky_vec,
            &self.solar,
            date,
            weather_data,
            SkyUnits::Visible,
            ALBEDO,
            true, // add sky
            true, // add sun
        )?;
        for (workplane, lux) in self.workplanes.iter().zip(memory.illuminance.iter_mut()) {
            workplane.illuminance_into(&memory.sky_vec, lux)?;
        }
        Ok(())
    }
}

/// The illuminance at the sensors of a [`Workplane`] over a simulation, for
/// calculating how many hours each sensor is within a certain range of
/// illuminance (e.g., over 300 lux).
///
/// Occupied hours can be defined as a range of hours of the day (e.g.,
/// `Some((8., 18.))`, from 8:00 to 18:00), which can also wrap around
/// midnight (e.g., `Some((22., 6.))`). `None` means every hour.
#[derive(Debug, Clone)]
pub struct IlluminanceRecord {
    n_sensors: usize,
    /// The duration of each timestep, in hours
    dt_hours: Float,
    /// The hour of the day of each timestep
    hours: Vec<Float>,
    /// The illuminance at each sensor, one timestep after the other
    lux: Vec<Float>,
}

impl IlluminanceRecord {
    /// Creates an empty record for `n_sensors` sensors, with timesteps of
    /// `dt_hours` hours
    pub fn new(n_sensors: usize, dt_hours: Float) -> Self {
        Self {
            n_sensors,
            dt_hours,
            hours: Vec::new(),
            lux: Vec::new(),
        }
    }

    /// Records the illuminance at each sensor on `date`
    pub fn push(&mut self, date: Date, lux: &[Float]) -> Result<(), String> {
        if lux.len() != self.n_sensors {
            return Err(format!(
                "Expecting the illuminance of {} sensors, found {}",
                self.n_sensors,
                lux.len()
            ));
        }
        self.hours.push(date.hour);
        self.lux.extend_from_slice(lux);
        Ok(())
    }

    /// The number of timesteps recorded
    pub fn len(&self) -> usize {
        self.hours.len()
    }

    /// Checks whether nothing has been recorded
    pub fn is_empty(&self) -> bool {
        self.hours.is_empty()
    }

    /// The illuminance at each sensor (one timestep after the other)
    pub fn values(&self) -> &[Float] {
        &self.lux
    }

    /// The number of hours each sensor received at least `min` lux
    pub fn hours_above(&self, min: Float, occupied: Option<(Float, Float)>) -> Vec<Float> {
        self.hours_where(occupied, |lux| lux >= min)
    }

    /// The number of hours each sensor received between `min` and `max` lux
    /// (both included)
    pub fn hours_within(
        &self,
        min: Float,
        max: Float,
        occupied: Option<(Float, Float)>,
    ) -> Vec<Float> {
        self.hours_where(occupied, |lux| lux >= min && lux <= max)
    }

    /// The number of occupied hours recorded
    pub fn occupied_hours(&self, occupied: Option<(Float, Float)>) -> Float {
        self.hours
            .iter()
            .filter(|h| is_occupied(**h, occupied))
            .count() as Float
            * self.dt_hours
    }

    fn hours_where<F: Fn(Float) -> bool>(
        &self,
        occupied: Option<(Float, Float)>,
        counts: F,
    ) -> Vec<Float> {
        let mut ret = vec![0.0; self.n_sensors];
        if self.n_sensors == 0 {
            return ret;
        }
        for (hour, lux) in self.hours.iter().zip(self.lux.chunks_exact(self.n_sensors)) {
            if !is_occupied(*hour, occupied) {
                continue;
            }
            for (total, lux) in ret.iter_mut().zip(lux) {
                if counts(*lux) {
                    *total += self.dt_hours;
                }
            }
        }
        ret
    }
}

fn is_occupied(hour: Float, occupied: Option<(Float, Float)>) -> bool {
    match occupied {
        None => true,
        Some((from, to)) if from <= to => hour >= from && hour < to,
        Some((from, to)) => hour >= from || hour < to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(hour: Float) -> Date {
        Date {
            month: 1,
            day: 1,
            hour,
        }
    }

    #[test]
    fn test_record_hours() -> Result<(), String> {
        let mut record = IlluminanceRecord::new(2, 0.5);
        record.push(date(7.), &[500., 50.])?;
        record.push(date(9.), &[500., 150.])?;
        record.push(date(12.), &[2500., 300.])?;
        record.push(date(23.), &[0., 0.])?;
        assert!(record.push(date(23.), &[0.]).is_err());

        assert_eq!(record.hours_above(300., None), vec![1.5, 0.5]);
        assert_eq!(record.hours_above(300., Some((8., 18.))), vec![1.0, 0.5]);
        assert_eq!(record.hours_within(100., 2000., None), vec![1.0, 1.0]);
        assert_eq!(
            record.hours_within(0., 100., Some((22., 8.))),
            vec![0.5, 1.0]
        );
        assert_eq!(record.occupied_hours(Some((8., 18.))), 1.0);
        assert_eq!(record.occupied_hours(None), 2.0);
        Ok(())
    }

    #[test]
    fn test_place_sensors() -> Result<(), String> {
        // The box's floor (back side facing the space, normal pointing down),
        // 15.24m x 15.24m
        let (model, _) = Model::from_file("../tests/box/box.spl")?;
        let (sensors, areas) = WorkplaneSensors::place_sensors(&model, "ZONE ONE", 0.7, 0.5)?;
        assert_eq!(sensors.len(), 31 * 31);
        assert_eq!(areas.len(), sensors.len());
        assert!((areas.iter().sum::<Float>() - 15.24 * 15.24).abs() < 1e-9);
        assert!(sensors.iter().all(|p| (p.z - 0.7).abs() < 1e-9));
        assert!(
            sensors
                .iter()
                .all(|p| (0.0..=15.24).contains(&p.x) && (0.0..=15.24).contains(&p.y))
        );
        // centered: as far from the walls on every side
        let min_x = sensors.iter().map(|p| p.x).fold(Float::MAX, Float::min);
        let max_x = sensors.iter().map(|p| p.x).fold(Float::MIN, Float::max);
        assert!((min_x - (15.24 - max_x)).abs() < 1e-9);

        // Front side facing the space, normal pointing up: 2.98m x 2.39m
        let (model, _) = Model::from_file("../tests/cold_apartment/cold.spl")?;
        let (sensors, _) = WorkplaneSensors::place_sensors(&model, "Kids Bedroom", 0.8, 0.5)?;
        assert_eq!(sensors.len(), 6 * 5);
        assert!(sensors.iter().all(|p| (p.z - 0.8).abs() < 1e-9));

        assert!(WorkplaneSensors::place_sensors(&model, "Nowhere", 0.7, 0.5).is_err());
        assert!(WorkplaneSensors::place_sensors(&model, "Kids Bedroom", 0.7, 0.0).is_err());
        assert!(WorkplaneSensors::place_sensors(&model, "Kids Bedroom", 0.7, 50.).is_ok()); // one, centered
        Ok(())
    }

    #[test]
    fn test_place_sensors_l_shape() -> Result<(), String> {
        // An L-shaped floor, at some height: sensors only within the L
        let model = Model::from_bytes(
            br#"
            Space { name: "L" }
            Substance { type: "Normal", name: "s", thermal_conductivity: 1, density: 1, specific_heat_capacity: 1 }
            Material { name: "m", substance: "s", thickness: 0.1 }
            Construction { name: "c", materials: ["m"] }
            Surface {
                name: "floor",
                construction: "c",
                front_boundary: { type: "Space", space: "L" },
                vertices: [0,0,3,  8,0,3,  8,3,3,  3,3,3,  3,6,3,  0,6,3]
            }
            "#,
        )?
        .0;
        let (sensors, areas) = WorkplaneSensors::place_sensors(&model, "L", 0.7, 1.0)?;
        // 8x3 + 3x3 cells
        assert_eq!(sensors.len(), 24 + 9);
        assert!((areas.iter().sum::<Float>() - 33.).abs() < 1e-9);
        assert!(sensors.iter().all(|p| (p.z - 3.7).abs() < 1e-9));
        assert!(!sensors.iter().any(|p| p.x > 3. && p.y > 3.));
        Ok(())
    }
}
