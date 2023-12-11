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

use crate::convection::ConvectionParams;
use crate::discretization::Discretization;
use crate::glazing::Glazing;
use crate::Float;
use geometry::Vector3D;
use matrix::Matrix;
use model::{
    Boundary, Construction, Fenestration, Model, SimulationStateHeader, Substance, Surface,
    SurfaceTrait, TerrainClass,
};
use model::{SimulationState, SiteDetails};
use std::sync::Arc;

/// Calculates whether a surface is facing the wind direction
/// **wind_direction in Radians**
pub fn is_windward(wind_direction: Float, cos_tilt: Float, normal: Vector3D) -> bool {
    if cos_tilt.abs() < 0.98 {
        // tilted
        let wind_direction = Vector3D::new(wind_direction.sin(), wind_direction.cos(), 0.0);
        normal * wind_direction > 0.0
    } else {
        // if it is horizontal
        true
    }
}

/// The memory needed to simulate the marching forward
/// of a massive chunk
#[derive(Debug, Clone)]
pub struct ChunkMemory {
    /// memory for a matrix
    pub temps: Matrix,
    /// memory for a matrix
    pub aux: Matrix,
    /// memory for a matrix
    pub k: Matrix,
    /// memory for a matrix
    pub c: Matrix,
    /// memory for a matrix
    pub q: Matrix,
    /// memory for a matrix
    pub k1: Matrix,

    /// memory for a matrix
    pub k2: Matrix,
    /// memory for a matrix
    pub k3: Matrix,
    /// memory for a matrix
    pub k4: Matrix,
}

impl ChunkMemory {
    /// Allocates memory for running a simulation of a chunk.
    ///
    /// This means allocating matrices
    pub fn new(ini: usize, fin: usize) -> Self {
        let n = fin - ini - 1;
        ChunkMemory {
            aux: Matrix::new(0.0, n + 1, 1),
            k: Matrix::new(0.0, n + 1, n + 1),
            c: Matrix::new(0.0, n + 1, n + 1),
            q: Matrix::new(0.0, n + 1, 1),
            temps: Matrix::new(0.0, n + 1, 1),
            k1: Matrix::new(0.0, n + 1, 1),
            k2: Matrix::new(0.0, n + 1, 1),
            k3: Matrix::new(0.0, n + 1, 1),
            k4: Matrix::new(0.0, n + 1, 1),
        }
    }
}

/// The memory needed to simulate the marching of
/// a surface
#[derive(Debug, Clone)]
pub struct SurfaceMemory {
    /// Memory for each massive chunk
    pub massive_chunks: Vec<ChunkMemory>,
    /// Memory for each no-mass chunk
    pub nomass_chunks: Vec<ChunkMemory>,
    /// The temperatures
    pub temperatures: Matrix,

    /// The solar absorption on each node
    pub q: Matrix,
}

fn rearrange_k(dt: Float, memory: &mut ChunkMemory) -> Result<(), String> {
    let (crows, ..) = memory.c.size();
    // Rearrenge into dT = (dt/C) * K + (dt/C)*q
    for nrow in 0..crows {
        let v = dt / memory.c.get(nrow, nrow)?;
        // transform k into k_prime (i.e., k * dt/C)
        let ini = if nrow == 0 { 0 } else { nrow - 1 };
        let fin = if nrow == crows - 1 {
            crows - 1
        } else {
            nrow + 1
        };
        // for ncol in 0..kcols{
        for ncol in ini..=fin {
            memory.k.scale_element(nrow, ncol, v)?;
        }
        memory.q.scale_element(nrow, 0, v)?;
    }
    Ok(())
}

/// Marches forward through time, solving the
/// Ordinary Differential Equation that governs the heat transfer in walls.
///
///
/// The equation to solve is the following:
///
/// ```math
/// \overline{C}  \dot{T} - \overline{K}  T = q
/// ```
///
/// Where $`\overline{C}`$ and $`\overline{K}`$ are matrices representing the
/// thermal mass of each node and the thermal network, respectively; and where $`T`$ and $`q`$ are
/// vectors representing the Temperature and the heat "flow into" each node, respectively.
/// $`\overline{C}`$ and $`\overline{K}`$ are build based on the finite difference method.
///
/// This model uses a 4th order [Runga-Kutte](https://en.wikipedia.org/wiki/Runge%E2%80%93Kutta_methods) (a.k.a., RK4)
/// to march through time. In order to do this, it is convenient to write the equation to solve
/// as follows:
///
/// ```math
/// \dot{T}  = f(t, T)
/// ```
///
/// Where
/// ```math
/// f(t,T) = \overline{C}^{-1} \overline{K}  T + \overline{C}^{-1} q
/// ```
///
/// Then, the 4th order Runge-Kutta method allows marching forward through time as follows:
/// ```math
///  T_{i+1} = T_i + \frac{k_1 + 2k_2 + 2k_3 + k_4}{6}
/// ```
/// Where $`k_1`$, $`k_2`$, $`k_3`$ and $`k_4`$ can be calculated based on the
/// timestep $`\Delta t`$ as follows:
///
/// * $`k_1 = \Delta t \times f(t,T)`$
/// * $`k_2 = \Delta t \times f(t+\frac{\Delta t}{2}, T+\frac{k_1}{2})`$
/// * $`k_3 = \Delta t \times f(t+\frac{\Delta t}{2}, T+\frac{k_2}{2})`$
/// * $`k_4 = \Delta t \times f(t+\delta t, T+k_3 )`$
pub fn rk4(memory: &mut ChunkMemory) -> Result<(), String> {
    #[cfg(debug_assertions)]
    {
        let (krows, kcols) = memory.k.size();
        assert_eq!(
            krows, kcols,
            "Expecting 'K' to be a squared matrix... nrows={}, ncols={}",
            krows, kcols
        );
        let (crows, ccols) = memory.c.size();
        assert_eq!(
            crows, ccols,
            "Expecting 'C' to be a squared matrix... nrows={}, ncols={}",
            crows, ccols
        );
        let (qrows, qcols) = memory.q.size();
        assert_eq!(
            qrows, krows,
            "Expecting 'q' to be to have {} rows because K has {} rows... found {}",
            krows, krows, qrows
        );
        assert_eq!(
            qcols, 1,
            "expecting 'q' to have 1 column... found {}",
            qcols
        );
    }

    // I am not sure why I need to clean... I thought this was not necessary.
    memory.k1 *= 0.0;
    memory.k2 *= 0.0;
    memory.k3 *= 0.0;
    memory.k4 *= 0.0;
    memory.aux *= 0.0;

    // get k1
    memory.k.prod_tri_diag_into(&memory.temps, &mut memory.k1)?;
    memory.k1 += &memory.q;

    // returning "temperatures + k1" is Euler... continuing is
    // Runge–Kutta 4th order
    /*
    memory.temps += &memory.k1;
    return Ok(());
    */

    memory.k1.scale_into(0.5, &mut memory.aux)?;
    memory.aux += &memory.temps;

    // k2
    memory.k.prod_tri_diag_into(&memory.aux, &mut memory.k2)?;
    memory.k2 += &memory.q;

    // k3
    memory.k2.scale_into(0.5, &mut memory.aux)?;
    memory.aux += &memory.temps;
    memory.k.prod_tri_diag_into(&memory.aux, &mut memory.k3)?;
    memory.k3 += &memory.q;

    // k4
    memory.aux.copy_from(&memory.k3);
    memory.aux += &memory.temps;
    memory.k.prod_tri_diag_into(&memory.aux, &mut memory.k4)?;
    memory.k4 += &memory.q;

    // Scale them and add them all up
    memory.k1 /= 6.;
    memory.k2 /= 3.;
    memory.k3 /= 3.;
    memory.k4 /= 6.;

    // Let's add it all and return
    memory.temps += &memory.k1;
    memory.temps += &memory.k2;
    memory.temps += &memory.k3;
    memory.temps += &memory.k4;

    Ok(())
}

/// This is a Surface from the point of view of our thermal solver.
/// Since this module only calculate heat transfer (and not short-wave solar
/// radiation, e.g., light), both model::Fenestration and model::Surface
/// are treated in the same way.
#[derive(Clone, Debug)]
pub struct ThermalSurfaceData<T: SurfaceTrait + Send + Sync> {
    /// A clone of the element in the [`Model`] which this struct represents
    pub parent: Arc<T>,

    /// The [`Discretization`] that represents this `ThermalSurfaceData`
    pub discretization: Discretization,

    /// The back boundary
    pub front_boundary: Boundary,

    /// The index of the space in front of this, if any
    pub front_space_index: Option<usize>,

    /// The back boundary
    pub back_boundary: Boundary,

    /// The index of the space at the back of this, if any
    pub back_space_index: Option<usize>,

    /// The thermal absorbtance on the front side (from 0 to 1)
    pub front_emissivity: Float,

    /// The thermal absorbtance on the back side (from 0 to 1)
    pub back_emissivity: Float,

    /// The area of the Surface
    pub area: Float,

    /// The perimeter of the surface
    pub perimeter: Float,

    /// The normal of the surface
    pub normal: Vector3D,

    /// The wind velocity changes with altitude. This field
    /// stores the factor with which the wind velocity of the weather
    /// file needs to be multiplied in order to estimate the wind speed
    /// at the exterior of the surface.
    pub wind_speed_modifier: Float,

    /// The cosine of the tilt angle (normal * Vector3D(0., 0., 1.))
    pub cos_tilt: Float,

    /// The chunks of nodes that have mass
    pub massive_chunks: Vec<(usize, usize)>,

    /// The chunks of nodes that have nomass
    pub nomass_chunks: Vec<(usize, usize)>,

    /// The absorbtances of each node in the system, proportional
    /// to the front incident radiation (i.e., they do not add up to 1.0)
    pub front_alphas: Matrix,

    /// The absorbtances of each node in the system, proportional
    /// to the back incident radiation (i.e., they do not add up to 1.0)
    pub back_alphas: Matrix,

    /// this allows setting a fixed convection
    /// coefficient
    pub front_hs: Option<Float>,

    /// this allows setting a fixed convection
    /// coefficient
    pub back_hs: Option<Float>,
}

impl<T: SurfaceTrait + Send + Sync> ThermalSurfaceData<T> {
    /// Allocates memory for the simulation
    pub fn allocate_memory(&self) -> SurfaceMemory {
        let massive_chunks = self
            .massive_chunks
            .iter()
            .map(|(ini, fin)| ChunkMemory::new(*ini, *fin))
            .collect();

        let nomass_chunks = self
            .nomass_chunks
            .iter()
            .map(|(ini, fin)| ChunkMemory::new(*ini, *fin))
            .collect();

        let ini = self.parent.first_node_temperature_index();
        let fin = self.parent.last_node_temperature_index() + 1;
        let n_nodes = fin - ini;
        let q = Matrix::new(0.0, n_nodes, 1);
        let temperatures = Matrix::new(0.0, n_nodes, 1);

        SurfaceMemory {
            massive_chunks,
            nomass_chunks,
            temperatures,
            q,
        }
    }

    /// Creates a new [`ThermalSurfaceData`]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: &mut SimulationStateHeader,
        model: &Model,
        site_details: &Option<SiteDetails>,
        ref_surface_index: usize,
        parent: &Arc<T>,
        area: Float,
        perimeter: Float,
        height: Float,
        normal: Vector3D,
        construction: &Arc<Construction>,
        discretization: Discretization,
    ) -> Result<ThermalSurfaceData<T>, String> {
        // Set Front and Back state
        parent.add_front_convection_state(state, ref_surface_index)?;
        parent.add_back_convection_state(state, ref_surface_index)?;

        parent.add_front_convective_heatflow_state(state, ref_surface_index)?;
        parent.add_back_convective_heatflow_state(state, ref_surface_index)?;

        parent.add_front_solar_irradiance_state(state, ref_surface_index)?;
        parent.add_back_solar_irradiance_state(state, ref_surface_index)?;

        parent.add_front_ir_irradiance_state(state, ref_surface_index)?;
        parent.add_back_ir_irradiance_state(state, ref_surface_index)?;

        // Add node data.
        let n_nodes = discretization.segments.len();
        parent.add_node_temperature_states(state, ref_surface_index, n_nodes)?;

        let front_substance = model.get_material_substance(&construction.materials[0])?;

        let back_substance = model
            .get_material_substance(construction.materials.last().ok_or("No last material")?)?;

        const DEFAULT_EM: Float = 0.84;
        let front_emissivity = match &front_substance {
            Substance::Normal(s) => {
                s.front_thermal_absorbtance_or(crate::heat_model::MODULE_NAME, DEFAULT_EM)
            }
            _ => panic!("Front Emissivity not available for this particular kind of Substance"),
        };
        let back_emissivity = match &&back_substance {
            Substance::Normal(s) => {
                s.back_thermal_absorbtance_or(crate::heat_model::MODULE_NAME, DEFAULT_EM)
            }
            _ => panic!("Front Emissivity not available for this particular kind of Substance"),
        };

        let (massive_chunks, nomass_chunks) = discretization.get_chunks();

        // Calculate solar absoption
        let front_glazing = Glazing::get_front_glazing_system(construction, model)?;
        let back_glazing = Glazing::get_back_glazing_system(construction, model)?;
        // These two are the absorbtion of each glazing layer. We need the absorption of each node
        let front_alphas_prev = Glazing::alphas(&front_glazing);

        if front_alphas_prev.len() != 1 && front_alphas_prev.len() != construction.materials.len() {
            eprintln!("Construction '{}' might to have a mixture of transparent and opaque layers. This is not currently supported.", construction.name());
        }
        let n_nodes = discretization.segments.len();
        let n_layers = construction.materials.len();

        let mut front_alphas = Matrix::new(0.0, n_nodes, 1);
        let mut global_i = 0;
        for (alpha_i, alpha) in front_alphas_prev.iter().enumerate() {
            let layer_index = 2 * alpha_i; // We need to skip cavities
            let n = if discretization.n_elements[layer_index] == 0 {
                1
            } else {
                discretization.n_elements[layer_index]
            };
            let substance = model.get_material_substance(&construction.materials[layer_index])?;
            if let Substance::Normal(sub) = &substance {
                let tau = *sub.solar_transmittance().unwrap_or(&0.0);
                if tau > 0.0 {
                    // Distribute across all the nodes
                    for local_i in 0..=n {
                        front_alphas.add_to_element(
                            global_i + local_i,
                            0,
                            *alpha / (n + 1) as Float,
                        )?;
                    }
                } else {
                    // Add only to the first node
                    front_alphas.add_to_element(global_i, 0, *alpha)?;
                }
            } else {
                unreachable!()
            }
            global_i += n + 1;
        }

        let back_alphas_prev = Glazing::alphas(&back_glazing);
        if back_alphas_prev.len() != 1 && back_alphas_prev.len() != construction.materials.len() {
            eprintln!("Construction '{}' might to have a mixture of transparent and opaque layers. This is not currently supported.", construction.name());
        }
        let mut back_alphas = Matrix::new(0.0, n_nodes, 1);
        let mut global_i = n_nodes;
        for (alpha_i, alpha) in back_alphas_prev.iter().enumerate() {
            let layer_index = n_layers - 2 * alpha_i - 1; // We need to skip cavities
            let n = if discretization.n_elements[layer_index] == 0 {
                1
            } else {
                discretization.n_elements[layer_index]
            };
            let substance = model.get_material_substance(&construction.materials[layer_index])?;
            if let Substance::Normal(sub) = substance {
                let tau = *sub.solar_transmittance().unwrap_or(&0.0);
                if tau > 0.0 {
                    // Distribute across all the nodes
                    for local_i in 0..=n {
                        let row = global_i - local_i - 1;
                        back_alphas.add_to_element(row, 0, *alpha / (n + 1) as Float)?;
                    }
                } else {
                    // Add only to the last node
                    back_alphas.add_to_element(global_i - 1, 0, *alpha)?;
                }
            } else {
                unreachable!()
            }
            global_i -= n + 1;
        }

        let cos_tilt = normal * Vector3D::new(0., 0., 1.);
        let wind_speed_modifier = match site_details {
            Some(d) => d.wind_speed_modifier(height),
            None => TerrainClass::default().wind_speed_modifier(height),
        };

        let parent = parent.clone();

        let front_hs = parent.fixed_front_hs();
        let back_hs = parent.fixed_back_hs();

        // Build resulting
        Ok(ThermalSurfaceData {
            parent,
            area,
            perimeter,
            normal,
            cos_tilt,
            discretization,
            front_boundary: Boundary::default(),
            back_boundary: Boundary::default(),
            front_space_index: None,
            back_space_index: None,
            front_emissivity,
            back_emissivity,
            wind_speed_modifier,
            front_alphas,
            back_alphas,
            massive_chunks,
            nomass_chunks,
            front_hs,
            back_hs,
        })
    }

    /// Sets the front boundary
    pub fn set_front_boundary(&mut self, b: Boundary, model: &Model) {
        self.front_boundary = b;
        if let Boundary::Space { space } = &self.front_boundary {
            for (i, s) in model.spaces.iter().enumerate() {
                if s.name() == space {
                    self.front_space_index = Some(i);
                    break;
                }
            }
        }
    }

    /// Sets the back boundary
    pub fn set_back_boundary(&mut self, b: Boundary, model: &Model) {
        self.back_boundary = b;
        if let Boundary::Space { space } = &self.back_boundary {
            for (i, s) in model.spaces.iter().enumerate() {
                if s.name() == space {
                    self.back_space_index = Some(i);
                    break;
                }
            }
        }
    }

    /// Calculates the border conditions
    pub fn calc_border_conditions(
        &self,
        state: &SimulationState,
        t_front: Float,
        t_back: Float,
        wind_direction: Float,
        wind_speed: Float,
    ) -> (ConvectionParams, ConvectionParams, Float, Float) {
        // Calculate and set Front and Back IR Irradiance
        let ir_front = self.parent.front_infrared_irradiance(state);
        let ir_back = self.parent.back_infrared_irradiance(state);

        let windward = is_windward(wind_direction, self.cos_tilt, self.normal);

        // TODO: There is something to do here if we are talking about windows
        let (front_env, mut front_hs) = match &self.front_boundary {
            Boundary::Adiabatic => {
                let front_env = ConvectionParams {
                    air_temperature: t_back,
                    air_speed: 0.0,
                    rad_temperature: t_back,
                    surface_temperature: self.parent.back_temperature(state),
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };

                let front_hs = match self.front_hs {
                    Some(v) => v,
                    None => front_env.get_tarp_natural_convection_coefficient(),
                };

                (front_env, front_hs)
            }
            Boundary::Space { .. } => {
                let front_env = ConvectionParams {
                    air_temperature: t_front,
                    air_speed: 0.0,
                    rad_temperature: t_front,
                    surface_temperature: self.parent.front_temperature(state),
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };

                let front_hs = match self.front_hs {
                    Some(v) => v,
                    None => front_env.get_tarp_natural_convection_coefficient(),
                };

                (front_env, front_hs)
            }
            Boundary::AmbientTemperature { temperature } => {
                let front_env = ConvectionParams {
                    air_temperature: *temperature,
                    air_speed: 0.0,
                    rad_temperature: *temperature,
                    surface_temperature: self.parent.front_temperature(state),
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };

                let front_hs = match self.front_hs {
                    Some(v) => v,
                    None => front_env.get_tarp_natural_convection_coefficient(),
                };

                (front_env, front_hs)
            }
            Boundary::Ground => unreachable!(),
            Boundary::Outdoor => {
                let mut front_env = ConvectionParams {
                    air_temperature: t_front,
                    air_speed: wind_speed * self.wind_speed_modifier,
                    rad_temperature: (ir_front / crate::SIGMA).powf(0.25) - 273.15,
                    surface_temperature: self.parent.front_temperature(state),
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };
                front_env.cos_surface_tilt = -self.cos_tilt;

                let front_hs = match self.front_hs {
                    Some(v) => v,
                    None => front_env.get_tarp_convection_coefficient(
                        self.area,
                        self.perimeter,
                        windward,
                    ),
                };

                (front_env, front_hs)
            }
        };

        let (back_env, mut back_hs) = match &self.back_boundary {
            Boundary::Adiabatic => {
                // Apply same boundary conditions
                (front_env, front_hs)
            }
            Boundary::Space { .. } => {
                let back_env = ConvectionParams {
                    air_temperature: t_back,
                    air_speed: 0.0,
                    rad_temperature: t_back, //self.parent.back_temperature(state),//(ir_back/crate::SIGMA).powf(0.25) - 273.15,
                    surface_temperature: self.parent.back_temperature(state),
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };
                let back_hs = match self.back_hs {
                    Some(v) => v,
                    None => back_env.get_tarp_natural_convection_coefficient(),
                };

                (back_env, back_hs)
            }
            Boundary::AmbientTemperature { temperature } => {
                let back_env = ConvectionParams {
                    air_temperature: *temperature,
                    air_speed: 0.0,
                    rad_temperature: *temperature,
                    surface_temperature: self.parent.front_temperature(state),
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };

                let back_hs = match self.back_hs {
                    Some(v) => v,
                    None => back_env.get_tarp_natural_convection_coefficient(),
                };

                (back_env, back_hs)
            }
            Boundary::Ground => unreachable!(),
            Boundary::Outdoor => {
                let surface_temperature = self.parent.back_temperature(state);
                let back_env = ConvectionParams {
                    air_temperature: t_back,
                    air_speed: wind_speed * self.wind_speed_modifier,
                    rad_temperature: (ir_back / crate::SIGMA).powf(0.25) - 273.15,
                    surface_temperature,
                    roughness_index: 1,
                    cos_surface_tilt: self.cos_tilt,
                };
                let back_hs = match self.back_hs {
                    Some(v) => v,
                    None => back_env.get_tarp_convection_coefficient(
                        self.area,
                        self.perimeter,
                        windward,
                    ),
                };

                (back_env, back_hs)
            }
        };

        // assert!(
        //     !front_hs.is_nan() && !back_hs.is_nan(),
        //     "Found NaN convection coefficients: Front={front_hs} | back={back_hs}"
        // );

        if front_hs.is_nan() && back_hs.is_nan() {
            front_hs = 2.0;
            back_hs = 2.0;
        } else if front_hs.is_nan() && !back_hs.is_nan() {
            front_hs = back_hs;
        } else if !front_hs.is_nan() && back_hs.is_nan() {
            back_hs = front_hs;
        }

        (front_env, back_env, front_hs, back_hs)
    }

    #[allow(clippy::too_many_arguments)]
    fn march_mass(
        &self,
        global_temperatures: &mut Matrix,
        solar_radiation: &Matrix,
        dt: Float,
        t_front: Float,
        t_back: Float,
        front_rad_hs: Float,
        back_rad_hs: Float,
        wind_direction: Float,
        wind_speed: Float,
        ini: usize,
        fin: usize,
        memory: &mut ChunkMemory,
        state: &SimulationState,
    ) -> Result<(), String> {
        let (front_env, back_env, front_hs, back_hs) =
            self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);

        self.discretization.get_k_q(
            ini,
            fin,
            global_temperatures,
            &front_env,
            front_hs,
            front_rad_hs,
            &back_env,
            back_hs,
            back_rad_hs,
            memory,
        )?;

        // Build Mass matrix
        memory.c *= 0.0;
        for (i, (mass, ..)) in self
            .discretization
            .segments
            .iter()
            .skip(ini)
            .take(fin - ini)
            .enumerate()
        {
            memory.c.set(i, i, *mass)?;
        }

        // ... here we add solar gains
        for (local_i, global_i) in (ini..fin).enumerate() {
            let v = solar_radiation.get(global_i, 0)?;
            memory.q.add_to_element(local_i, 0, v)?;
        }

        rearrange_k(dt, memory)?;

        // Use RT4 for updating temperatures of massive nodes.
        // let mut local_temps = Matrix::new(0.0, fin - ini, 1);
        for (local_i, global_i) in (ini..fin).enumerate() {
            let v = global_temperatures.get(global_i, 0)?;
            #[cfg(debug_assertions)]
            if v.is_nan() {
                dbg!(v);
            }
            memory.temps.set(local_i, 0, v)?;
        }

        rk4(memory)?;

        for (local_i, global_i) in (ini..fin).enumerate() {
            let v = memory.temps.get(local_i, 0)?;
            #[cfg(debug_assertions)]
            if v.is_nan() {
                dbg!(v);
            }
            global_temperatures.set(global_i, 0, v)?;
        }
        Ok(())
    }

    /*
    /// This was meant an experimental alternative to the
    /// `march_nomass()` method in this same file...but it wasn't better than
    /// that.
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    fn march_nomass_newton_ralphson(
        &self,
        global_temperatures: &mut Matrix,
        solar_radiation: &Matrix,
        t_front: Float,
        t_back: Float,
        _front_rad_hs: Float,
        _back_rad_hs: Float,
        wind_direction: Float,
        wind_speed: Float,
        ini: usize,
        fin: usize,
        memory: &mut ChunkMemory,
        state: &SimulationState,
    ) -> Result<(), String> {
        let mut min_error = Float::MAX;
        let mut old_err = Float::MAX;

        let mut count = 0;
        let mut residuals = memory.q.clone();
        let (nrows, _) = residuals.size();
        let mut jacobian = memory.k.clone();

        for j in 0..nrows {
            let v = global_temperatures.get(j + ini, 0)?;
            memory.temps.set(j, 0, v)?;
        }
        // // initial guess
        // let (front_env, back_env, front_hs, back_hs) =
        //             self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);

        // self.discretization.get_k_q(
        //     ini,
        //     fin,
        //     global_temperatures,
        //     &front_env,
        //     front_hs,
        //     front_rad_hs,
        //     &back_env,
        //     back_hs,
        //     back_rad_hs,
        //     memory,
        // )?;

        // // add solar gains
        // for (local_i, i) in (ini..fin).enumerate() {
        //     let v = solar_radiation.get(i, 0)?;
        //     memory.q.add_to_element(local_i, 0, v)?;
        // }

        // memory.k.mut_n_diag_gaussian(&mut memory.q, 3)?;
        // memory.temps.copy_from(&memory.q);

        //     for (local_i, i) in (ini..fin).enumerate() {
        //         let local_temp = memory.temps.get(local_i, 0)?;
        //         global_temperatures.set(i, 0, local_temp)?;
        //         // global_temperatures.add_to_element(i, 0, local_temp)?;
        //         // global_temperatures.scale_element(i, 0, 0.5)?;
        //     }

        let calc_residual = move |memory: &mut ChunkMemory,
                                  residuals: &mut Matrix,
                                  global_temperatures: &Matrix|
              -> Result<Float, String> {
            // Update convection coefficients
            let (front_env, back_env, front_hs, back_hs) =
                self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);

            let front_rad_hs = 4.
                * self.front_emissivity
                * crate::SIGMA
                * (273.15 + (front_env.rad_temperature + front_env.surface_temperature) / 2.)
                    .powi(3);
            let back_rad_hs = 4.
                * self.back_emissivity
                * crate::SIGMA
                * (273.15 + (back_env.rad_temperature + back_env.surface_temperature) / 2.).powi(3);

            self.discretization.get_k_q(
                ini,
                fin,
                global_temperatures,
                &front_env,
                front_hs,
                front_rad_hs,
                &back_env,
                back_hs,
                back_rad_hs,
                memory,
            )?;

            // add solar gains
            for (local_i, i) in (ini..fin).enumerate() {
                let v = solar_radiation.get(i, 0)?;
                memory.q.add_to_element(local_i, 0, v)?;
            }
            memory.q *= -1.;

            memory.k.prod_into(&memory.temps, residuals).unwrap();
            *residuals -= &memory.q;

            let mut err = 0.0;
            for j in 0..nrows {
                let aux = residuals.get(j, 0).unwrap();
                err += aux;
            }
            err /= nrows as Float;

            // Ok(err.abs()/nrows as Floats)
            Ok(err)
        };

        loop {
            let err = calc_residual(memory, &mut residuals, global_temperatures)?;
            let prev_temps = memory.temps.clone();

            // if err > old_err {
            //     #[cfg(debug_assertions)]
            //     if count > 100 {
            //         eprintln!("Breaking after {} iterations... because BAD!", count);
            //     }
            //     break;
            // }

            if err.is_nan() {
                assert!(
                    !err.is_nan(),
                    // "Error is NaN... \nfront_env = {:?}| back_env = {:?} \nfront_hc = {} | back_hs = {}. \nError = {}\ntemps={}\nq={}\nsolar_front={}, solar_back={}\nfront_alphas={}\nback_alphas={}\n",
                    // front_env,
                    // back_env,
                    // front_hs,
                    // back_hs,
                    // err / ((fin - ini) as Float),
                    // temps,
                    // q,
                    // solar_front,
                    // solar_back,
                    // self.front_alphas,
                    // self.back_alphas,
                );
            }

            let prev_residuals = residuals.clone();
            // Calculate derivatives
            for j in 0..nrows {
                const DELTA: Float = 0.01;
                // increase
                memory.temps.add_to_element(j, 0, DELTA)?;
                // calculate change
                calc_residual(memory, &mut residuals, global_temperatures)?;

                for k in 0..nrows {
                    let delta_t = residuals.get(k, 0)? - prev_residuals.get(k, 0)?;
                    // jacobian.set(j, k, delta_t/DELTA)?;
                    jacobian.set(k, j, delta_t / DELTA)?;
                }
                // go back
                memory.temps.add_to_element(j, 0, -DELTA)?;
                calc_residual(memory, &mut residuals, global_temperatures)?;
            }
            // update guess.
            // let mut delta = memory.temps.clone();
            // jacobian.gauss_seidel(&prev_residuals, &mut delta, 9999, 0.001)?;
            let mut delta = prev_residuals;
            jacobian.mut_n_diag_gaussian(&mut delta, 3)?;
            // temps_clone *= 0.5;
            // delta *= 0.1;
            memory.temps -= &delta;

            let mut err = 0.0;
            for j in 0..nrows {
                let aux = memory.temps.get(j, 0).unwrap() - prev_temps.get(j, 0).unwrap();
                err += aux.abs();
            }
            err /= nrows as Float;

            let threshold = if count < 100 { 0.02 } else { 0.2 };
            if err.abs() < threshold {
                break;
            }

            if err.abs() < min_error {
                min_error = err.abs();
            }
            old_err = err;

            count += 1;
            if count > 600 {
                eprintln!("{}", &memory.temps);
                panic!(
                    "Too many iterations... did not converge... min_error is {}",
                    min_error
                );
            }
        }

        for (local_i, i) in (ini..fin).enumerate() {
            let local_temp = memory.temps.get(local_i, 0)?;
            global_temperatures.set(i, 0, local_temp)?;
            // global_temperatures.add_to_element(i, 0, local_temp)?;
            // global_temperatures.scale_element(i, 0, 0.5)?;
        }

        Ok(())
    }
    */

    #[allow(clippy::too_many_arguments)]
    fn march_nomass(
        &self,
        global_temperatures: &mut Matrix,
        solar_radiation: &Matrix,
        t_front: Float,
        t_back: Float,
        front_rad_hs: Float,
        back_rad_hs: Float,
        wind_direction: Float,
        wind_speed: Float,
        ini: usize,
        fin: usize,
        memory: &mut ChunkMemory,
        state: &SimulationState,
    ) -> Result<(), String> {
        let mut old_err = 99999.;
        let mut count = 0;

        let mut temp_k = memory.k.clone();
        let mut temps = memory.q.clone();

        loop {
            // Update convection coefficients
            let (front_env, back_env, front_hs, back_hs) =
                self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);

            // Calculate q based on heat transfer (convection, IR radiation)
            self.discretization.get_k_q(
                ini,
                fin,
                global_temperatures,
                &front_env,
                front_hs,
                front_rad_hs,
                &back_env,
                back_hs,
                back_rad_hs,
                memory,
            )?;

            // add solar gains
            for (local_i, i) in (ini..fin).enumerate() {
                let v = solar_radiation.get(i, 0)?;
                memory.q.add_to_element(local_i, 0, v)?;
            }
            memory.q *= -1.;

            temp_k.copy_from(&memory.k);
            temps.copy_from(&memory.q);

            temp_k.mut_n_diag_gaussian(&mut temps, 3)?; // and just like that, temps is the new temperatures

            let mut err = 0.0;
            for (local_i, i) in (ini..fin).enumerate() {
                let local_temp = temps.get(local_i, 0)?;
                let global_temp = global_temperatures.get(i, 0)?;
                err += (local_temp - global_temp).abs();
            }
            err /= (fin - ini) as Float;
            if err > old_err {
                // #[cfg(debug_assertions)]
                if count > 100 {
                    eprintln!("Breaking after {} iterations... because BAD!", count);
                }
                break;
            }

            if err.is_nan() {
                assert!(
                    !err.is_nan(),
                    // "Error is NaN... \nfront_env = {:?}| back_env = {:?} \nfront_hc = {} | back_hs = {}. \nError = {}\ntemps={}\nq={}\nsolar_front={}, solar_back={}\nfront_alphas={}\nback_alphas={}\n",
                    // front_env,
                    // back_env,
                    // front_hs,
                    // back_hs,
                    // err / ((fin - ini) as Float),
                    // temps,
                    // q,
                    // solar_front,
                    // solar_back,
                    // self.front_alphas,
                    // self.back_alphas,
                );
            }

            
            // assert!(
            //     count < 900,
            //     "Excessive number of iterations... \n====\t\tfront_env = {:?}\n\tback_env = {:?}\n\tfront_hc = {}\n\tback_hs = {}.\n\tError = {}\n====\n",
            //     front_env,
            //     back_env,
            //     front_hs,
            //     back_hs,
            //     err / ((fin - ini) as Float),
            // );
            for (local_i, i) in (ini..fin).enumerate() {
                let local_temp = temps.get(local_i, 0)?;
                // temperatures.set(i, 0, local_temp)?;
                global_temperatures.add_to_element(i, 0, local_temp)?;
                global_temperatures.scale_element(i, 0, 0.5)?;
            }

            let max_allowed_error = if count < 100 { 0.01 } else /*if count < 1000*/ { 0.5 }; // else { 1. };

            if err  < max_allowed_error {
                // #[cfg(debug_assertions)]
                // eprintln!(
                //     "Breaking after {} iterations... because err = {}",
                //     count,
                //     err / ((fin - ini) as Float)
                // );
                break;
            }
            if count > 19000 {
                break // Sometimes this converges too slowly.
            }
            old_err = err;
            count += 1;
        }
        Ok(())
    }

    /*
    /// This was meant an experimental alternative to the
    /// `march_nomass()` method in this same file...but it wasn't better than
    /// that.
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    fn march_nomass_virtual_mass(
        &self,
        global_temperatures: &mut Matrix,
        solar_radiation: &Matrix,
        t_front: Float,
        t_back: Float,
        front_rad_hs: Float,
        back_rad_hs: Float,
        wind_direction: Float,
        wind_speed: Float,
        ini: usize,
        fin: usize,
        memory: &mut ChunkMemory,
        state: &SimulationState,
    ) -> Result<(), String> {
        // Build Mass matrix
        let (n_nodes, _) = memory.temps.size();
        memory.c = Matrix::eye(n_nodes);
        // memory.c *= 300.;

        // Use RT4 for updating temperatures of massive nodes.
        // let mut local_temps = Matrix::new(0.0, fin - ini, 1);
        for (local_i, global_i) in (ini..fin).enumerate() {
            let v = global_temperatures.get(global_i, 0)?;
            memory.temps.set(local_i, 0, v)?;
        }

        let mut prev_temps = memory.temps.clone();
        const DT: Float = 0.0005;
        let mut count = 0;

        let (front_env, back_env, front_hs, back_hs) =
            self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);

        self.discretization.get_k_q(
            ini,
            fin,
            global_temperatures,
            &front_env,
            front_hs,
            front_rad_hs,
            &back_env,
            back_hs,
            back_rad_hs,
            memory,
        )?;
        // ... here we add solar gains
        for (local_i, global_i) in (ini..fin).enumerate() {
            let v = solar_radiation.get(global_i, 0)?;
            memory.q.add_to_element(local_i, 0, v)?;
        }

        rearrange_k(DT, memory)?;
        loop {
            // Updated temperatures
            rk4(memory)?;

            let mut delta = Float::MAX;
            for i in 0..n_nodes {
                let aux = (prev_temps.get(i, 0)? - memory.temps.get(i, 0)?).abs();
                if aux.is_nan() {
                    return Err(format!("Error is NaN after {} iterations", count));
                }
                if aux < delta {
                    delta = aux;
                }
            }
            prev_temps = memory.temps.clone();
            for (local_i, global_i) in (ini..fin).enumerate() {
                let v = memory.temps.get(local_i, 0)?;
                global_temperatures.set(global_i, 0, v)?;
            }
            if delta < 0.00001 {
                break;
            }
            count += 1;
            const MAX_ITER: usize = 20000;
            if count > MAX_ITER {
                return Err(format!("Error is {} after {} iterations", delta, MAX_ITER));
            }
        }

        // for (local_i, global_i) in (ini..fin).enumerate() {
        //     let v = memory.temps.get(local_i, 0)?;
        //     global_temperatures.set(global_i, 0, v)?;
        // }
        Ok(())
    }
    */

    /// Marches one timestep. Returns front and back heat flow    
    #[allow(clippy::too_many_arguments)]
    pub fn march(
        &self,
        state: &SimulationState,
        t_front: Float,
        t_back: Float,
        wind_direction: Float,
        wind_speed: Float,
        dt: Float,
        memory: &mut SurfaceMemory,
    ) -> Result<(), String> {
        // Calculate and set Front and Back Solar Irradiance
        let mut solar_front = self.parent.front_solar_irradiance(state);
        if solar_front.is_nan() || solar_front < 0.0 {
            solar_front = 0.0;
        }
        let mut solar_back = self.parent.back_solar_irradiance(state);
        if solar_back.is_nan() || solar_front < 0.0 {
            solar_back = 0.0;
        }

        /////////////////////
        // 1st: Calculate the solar radiation absorbed by each node
        /////////////////////
        let mut solar_radiation = &self.front_alphas * solar_front;
        solar_radiation += &(&self.back_alphas * solar_back);

        /////////////////////
        // 2nd: Calculate the temperature in all no-mass nodes.
        // Also, the heat flow into
        /////////////////////

        let (front_env, back_env, _front_hs, _back_hs) =
            self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);
        let front_rad_hs = 4.
            * self.front_emissivity
            * crate::SIGMA
            * (273.15 + (front_env.rad_temperature + front_env.surface_temperature) / 2.).powi(3);
        let back_rad_hs = 4.
            * self.back_emissivity
            * crate::SIGMA
            * (273.15 + (back_env.rad_temperature + back_env.surface_temperature) / 2.).powi(3);

        for (chunk_i, (ini, fin)) in self.nomass_chunks.iter().enumerate() {
            self.march_nomass(
                &mut memory.temperatures,
                &solar_radiation, // &memory.q,
                t_front,
                t_back,
                front_rad_hs,
                back_rad_hs,
                wind_direction,
                wind_speed,
                *ini,
                *fin,
                &mut memory.nomass_chunks[chunk_i],
                state,
            )?;
        }

        // Calculate final conditions.

        let (front_env, back_env, _front_hs, _back_hs) =
            self.calc_border_conditions(state, t_front, t_back, wind_direction, wind_speed);
        let front_rad_hs = 4.
            * self.front_emissivity
            * crate::SIGMA
            * (273.15 + (front_env.rad_temperature + front_env.surface_temperature) / 2.).powi(3);
        let back_rad_hs = 4.
            * self.back_emissivity
            * crate::SIGMA
            * (273.15 + (back_env.rad_temperature + back_env.surface_temperature) / 2.).powi(3);

        /////////////////////
        // 3rd: Calculate K and C matrices for the massive walls, and march
        /////////////////////

        for (chunk_i, (ini, fin)) in self.massive_chunks.iter().enumerate() {
            self.march_mass(
                &mut memory.temperatures,
                &solar_radiation, // &memory.q,
                dt,
                t_front,
                t_back,
                front_rad_hs,
                back_rad_hs,
                wind_direction,
                wind_speed,
                *ini,
                *fin,
                &mut memory.massive_chunks[chunk_i],
                state,
            )?;
        }
        Ok(())
    }
}

/// A [`ThermalSurfaceData`] whose parent is a [`Surface`]
pub type ThermalSurface = ThermalSurfaceData<Surface>;

/// A [`ThermalSurfaceData`] whose parent is a [`Fenestration`]
pub type ThermalFenestration = ThermalSurfaceData<Fenestration>;

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {

    use super::*;
    use geometry::{Loop3D, Point3D, Polygon3D};

    use model::{
        substance::Normal as NormalSubstance, Construction, Material, Model, Substance, Surface,
    };

    fn add_polyurethane(model: &mut Model) -> Result<Substance, String> {
        let mut poly = NormalSubstance::new("polyurethane".to_string());
        poly.set_density(17.5) // kg/m3... reverse engineered from paper
            .set_specific_heat_capacity(2400.) // J/kg.K
            .set_front_thermal_absorbtance(0.)
            .set_back_thermal_absorbtance(0.)
            .set_thermal_conductivity(0.0252); // W/m.K

        assert_eq!(poly.thermal_diffusivity()?, 0.6E-6);
        let ret = model.add_substance(poly.wrap());
        Ok(ret)
    }

    fn add_brickwork(model: &mut Model) -> Result<Substance, String> {
        let mut brickwork = NormalSubstance::new("brickwork".to_string());

        brickwork
            .set_density(1700.) // kg/m3... reverse engineered from paper
            .set_specific_heat_capacity(800.) // J/kg.K
            .set_front_thermal_absorbtance(0.)
            .set_back_thermal_absorbtance(0.)
            .set_thermal_conductivity(0.816); // W/m.K

        assert!((brickwork.thermal_diffusivity()? - 0.6E-6).abs() < 0.00000001);
        let ret = model.add_substance(brickwork.wrap());

        Ok(ret)
    }

    fn add_material(model: &mut Model, substance: Substance, thickness: Float) -> Arc<Material> {
        let mat = Material::new("123123".to_string(), substance.name().clone(), thickness);

        model.add_material(mat)
    }

    #[test]
    fn test_march_massive_1() -> Result<(), String> {
        let mut model = Model::default();

        /* SUBSTANCES */
        let brickwork = add_brickwork(&mut model)?;

        /* MATERIALS */
        let m1 = add_material(&mut model, brickwork, 20. / 1000.);

        /* CONSTRUCTION */
        let mut c = Construction::new("construction".to_string());
        c.materials.push(m1.name().clone());
        let c = model.add_construction(c);

        /* GEOMETRY */
        let mut the_loop = Loop3D::new();
        let l = 1. as Float;
        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.close()?;
        let p = Polygon3D::new(the_loop)?;

        /* SURFACE */
        let mut s = Surface::new(
            "Surface 1",
            p,
            c.name(),
            Boundary::Outdoor,
            Boundary::Outdoor,
        );
        s.set_precalculated_back_convection_coef(10.0);
        s.set_precalculated_front_convection_coef(10.0);

        let surface = model.add_surface(s);

        // FIRST TEST -- 10 degrees on each side
        let main_dt = 300.0;
        let max_dx = m1.thickness / 2.0;
        let min_dt = 1.0;
        let d = Discretization::new(&c, &model, main_dt, max_dx, min_dt, 1., 0.)?;
        let dt = main_dt / d.tstep_subdivision as Float;
        let normal = geometry::Vector3D::new(0., 0., 1.);
        let perimeter = 8. * l;
        let mut state_header = SimulationStateHeader::new();
        let ts = ThermalSurface::new(
            &mut state_header,
            &model,
            &None,
            0,
            &surface,
            surface.area(),
            perimeter,
            10.,
            normal,
            &c,
            d,
        )?;

        let mut state = state_header.take_values().ok_or("Could not take values")?;

        // TEST

        // Try marching until q_in and q_out are zero.
        let mut q: Float = 9999000009.0;
        let mut counter: usize = 0;
        let t_environment = 10.;
        let v = crate::SIGMA * (t_environment + 273.15 as Float).powi(4);

        let mut memory = ts.allocate_memory();
        ts.parent
            .get_node_temperatures(&state, &mut memory.temperatures)?;
        let surfaces = vec![ts];
        let mut alloc = vec![memory];

        while q.abs() > 0.00015 {
            surfaces[0].parent.set_front_ir_irradiance(&mut state, v)?;
            surfaces[0].parent.set_back_ir_irradiance(&mut state, v)?;

            crate::heat_model::iterate_surfaces(
                &surfaces,
                &mut alloc,
                0.0,
                0.0,
                t_environment,
                dt,
                &model,
                &mut state,
            )?;

            let q_in = surface
                .back_convective_heat_flow(&state)
                .ok_or("back_convective_heat_flow")?;
            let q_out = surface
                .front_convective_heat_flow(&state)
                .ok_or("front_convective_heat_flow")?;
            // the same amount of heat needs to leave in each direction
            // println!("q_in = {}, q_out = {} | diff = {}", q_in, q_out, (q_in - q_out).abs());
            assert!(
                (q_in - q_out).abs() < 0.5, //1E-5,
                "diff is {} (count is {counter})",
                (q_in - q_out).abs()
            );

            // q_front is positive
            assert!(q_in >= 0., "q_in = {} | c = {}", q_in, counter);
            assert!(q_out >= 0., "q_out = {} | c = {}", q_out, counter);

            q = q_in;

            counter += 1;
            if counter > 9999999 {
                panic!("Exceded number of iterations... q.abs() = {}", q.abs())
            }
        }

        // all nodes should be at 10.0 now.
        let ini = surfaces[0]
            .parent
            .first_node_temperature_index()
            .ok_or("first_node_temperature_index")?;
        let fin = surfaces[0]
            .parent
            .last_node_temperature_index()
            .ok_or("last_node_temperature_index")?
            + 1;
        let n_nodes = fin - ini;
        let mut temperatures = Matrix::new(0.0, n_nodes, 1);
        surfaces[0]
            .parent
            .get_node_temperatures(&state, &mut temperatures)?;
        for i in 0..n_nodes {
            let t = temperatures.get(i, 0)?;
            assert!(
                (t - 10.0).abs() < 0.002,
                "Error found is {}",
                (t - 10.0).abs()
            );
        }

        Ok(())
    }

    #[test]
    fn test_march_massive_2() -> Result<(), String> {
        let mut model = Model::default();

        /* SUBSTANCES */
        let brickwork = add_brickwork(&mut model)?;

        /* MATERIALS */
        let m1 = add_material(&mut model, brickwork, 20. / 1000.);

        /* CONSTRUCTION */
        let mut c = Construction::new("construction".to_string());
        c.materials.push(m1.name().clone());
        let c = model.add_construction(c);

        /* GEOMETRY */
        let mut the_loop = Loop3D::new();
        let l = 1. as Float;
        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.close()?;
        let p = Polygon3D::new(the_loop)?;

        /* SURFACE */
        let mut s = Surface::new(
            "Surface 1",
            p,
            c.name(),
            Boundary::AmbientTemperature { temperature: 30.0 },
            Boundary::Outdoor,
        );
        s.set_precalculated_back_convection_coef(10.0);
        s.set_precalculated_front_convection_coef(10.0);

        let surface = model.add_surface(s);

        // FIRST TEST -- 10 degrees on each side
        let main_dt = 300.0;
        let max_dx = m1.thickness / 2.0;
        let min_dt = 1.0;
        let d = Discretization::new(&c, &model, main_dt, max_dx, min_dt, 1., 0.)?;
        let dt = main_dt / d.tstep_subdivision as Float;
        let normal = geometry::Vector3D::new(0., 0., 1.);
        let perimeter = 8. * l;
        let mut state_header = SimulationStateHeader::new();
        let ts = ThermalSurface::new(
            &mut state_header,
            &model,
            &None,
            0,
            &surface,
            surface.area(),
            perimeter,
            10.,
            normal,
            &c,
            d,
        )?;

        let mut state = state_header.take_values().ok_or("Could not take values")?;

        let memory = ts.allocate_memory();
        let surfaces = vec![ts];
        let mut alloc = vec![memory];

        //  TEST -- 10 degrees in and 30 out.
        // We expect the final q to be (30-10)/R from
        // outside to inside. I.E. q_in = (30-10)/R,
        // q_out = -(30-10)/R

        // March until q converges
        let mut change: Float = 99.0;
        let mut counter: usize = 0;
        let mut previous_q: Float = -125.0;
        let mut final_qfront: Float = -12312.;
        let mut final_qback: Float = 123123123.;
        while change.abs() > 1E-10 {
            crate::heat_model::iterate_surfaces(
                &surfaces, &mut alloc, 0.0, 0.0, 10.0, dt, &model, &mut state,
            )?;

            let q_front = surface
                .front_convective_heat_flow(&state)
                .ok_or("front_convective_heat_flow")?;
            let q_back = surface
                .back_convective_heat_flow(&state)
                .ok_or("back_convective_heat_flow")?;

            surfaces[0].parent.set_front_ir_irradiance(
                &mut state,
                crate::SIGMA * (10. + 273.15 as Float).powi(4),
            )?;
            surfaces[0].parent.set_back_ir_irradiance(
                &mut state,
                crate::SIGMA * (30. + 273.15 as Float).powi(4),
            )?;

            final_qfront = q_front;
            final_qback = q_back;

            change = (q_front - previous_q).abs();
            previous_q = q_front;

            counter += 1;
            if counter > 99999 {
                panic!("Exceded number of iterations")
            }
        }
        // Expecting
        #[cfg(feature = "float")]
        const SMOL: Float = 1e-3;
        #[cfg(not(feature = "float"))]
        const SMOL: Float = 1e-5;
        assert!(final_qfront > -SMOL, "final_qfront = {}", final_qfront);
        assert!(final_qback < SMOL, "final_qback = {}", final_qback);

        Ok(())
    }

    #[test]
    fn test_march_nomass() -> Result<(), String> {
        let mut model = Model::default();

        /* SUBSTANCE */
        let polyurethane = add_polyurethane(&mut model)?;

        /* MATERIAL */
        let m1 = add_material(&mut model, polyurethane, 3. / 1000.);

        /* CONSTRUCTION */
        let mut c = Construction::new("Construction");
        c.materials.push(m1.name().clone());
        c.materials.push(m1.name().clone());
        let c = model.add_construction(c);

        /* GEOMETRY */
        let mut the_loop = Loop3D::new();
        let l = 1. as Float;
        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.close()?;
        let p = Polygon3D::new(the_loop)?;

        /* SURFACE */
        let mut s = Surface::new("WALL", p, c.name(), Boundary::Outdoor, Boundary::Outdoor);
        s.set_precalculated_back_convection_coef(10.0);
        s.set_precalculated_front_convection_coef(10.0);
        let surface = model.add_surface(s);
        let mut state_header = SimulationStateHeader::new();

        /* TEST */

        let main_dt = 3.0;
        let max_dx = m1.thickness / 7.0;
        let min_dt = 10.0;
        let d = Discretization::new(&c, &model, main_dt, max_dx, min_dt, 1., 0.)?;
        let dt = main_dt / d.tstep_subdivision as Float;

        let normal = geometry::Vector3D::new(0., 0., 1.);
        let perimeter = 8. * l;
        let ts = ThermalSurface::new(
            &mut state_header,
            &model,
            &None,
            0,
            &surface,
            surface.area(),
            perimeter,
            10.,
            normal,
            &c,
            d,
        )?;

        let mut state = state_header.take_values().ok_or("Could not take values")?;

        // FIRST TEST -- 10 degrees on each side

        // Try marching until q_in and q_out are zero.

        let memory = ts.allocate_memory();
        let surfaces = vec![ts];
        let mut alloc = vec![memory];

        crate::heat_model::iterate_surfaces(
            &surfaces, &mut alloc, 0.0, 0.0, 10.0, dt, &model, &mut state,
        )?;

        let q_in = surface
            .front_convective_heat_flow(&state)
            .ok_or("Could not get front convective flow")?;
        let q_out = surface
            .back_convective_heat_flow(&state)
            .ok_or("Could not get back convective flow")?;

        // this should show instantaneous update. So,
        let ini = surfaces[0]
            .parent
            .first_node_temperature_index()
            .ok_or("Could not get first node index")?;
        let fin = surfaces[0]
            .parent
            .last_node_temperature_index()
            .ok_or("Could not get last node index")?
            + 1;
        let n_nodes = fin - ini;
        let mut temperatures = Matrix::new(0.0, n_nodes, 1);
        surfaces[0]
            .parent
            .get_node_temperatures(&state, &mut temperatures)?;

        println!(" T == {}", &temperatures);
        assert!(
            (temperatures.get(0, 0)? - 10.0).abs() < 0.2,
            "T = {}",
            temperatures.get(0, 0)?
        );
        assert!(
            (temperatures.get(n_nodes - 1, 0)? - 10.0).abs() < 0.2,
            "T = {}",
            temperatures.get(n_nodes - 1, 0)?
        );
        assert!(q_in.abs() < 0.07, "q_in is {}", q_in);
        assert!(q_out.abs() < 0.07);

        Ok(())
    }

    #[test]
    fn test_march_nomass_2() -> Result<(), String> {
        let mut model = Model::default();

        /* SUBSTANCE */
        let polyurethane = add_polyurethane(&mut model)?;

        /* MATERIAL */
        let m1 = add_material(&mut model, polyurethane, 3. / 1000.);

        /* CONSTRUCTION */
        let mut c = Construction::new("Construction".to_string());
        c.materials.push(m1.name().clone());
        c.materials.push(m1.name().clone());
        let c = model.add_construction(c);

        /* GEOMETRY */
        let mut the_loop = Loop3D::new();
        let l = 1. as Float;
        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.close()?;
        let p = Polygon3D::new(the_loop)?;

        /* SURFACE */
        let s = Surface::new(
            "WALL".to_string(),
            p,
            c.name().clone(),
            Boundary::Outdoor,
            Boundary::AmbientTemperature { temperature: 30. },
        );

        let surface = model.add_surface(s);
        let mut state_header = SimulationStateHeader::new();

        /* TEST */

        let main_dt = 3.0;
        let max_dx = m1.thickness / 7.0;
        let min_dt = 10.0;
        let d = Discretization::new(&c, &model, main_dt, max_dx, min_dt, 1., 0.)?;
        let dt = main_dt / d.tstep_subdivision as Float;

        let normal = geometry::Vector3D::new(0., 0., 1.);
        let perimeter = 8. * l;
        let mut ts = ThermalSurface::new(
            &mut state_header,
            &model,
            &None,
            0,
            &surface,
            surface.area(),
            perimeter,
            10.,
            normal,
            &c,
            d,
        )?;

        ts.front_hs = Some(10.);
        ts.back_hs = Some(10.);

        // assert!(!d.is_massive);

        let mut state = state_header.take_values().ok_or("Could not take values")?;

        let mut memory = ts.allocate_memory();
        ts.parent
            .get_node_temperatures(&state, &mut memory.temperatures)?;
        let surfaces = vec![ts];
        let mut alloc = vec![memory];

        // SECOND TEST -- 10 degrees in and 30 out.
        // We expect the final q to be (30-10)/R from
        // outside to inside. I.E. q_in = (30-10)/R,
        // q_out = -(30-10)/R

        crate::heat_model::iterate_surfaces(
            &surfaces, &mut alloc, 0.0, 0.0, 10.0, dt, &model, &mut state,
        )?;

        let q_front = surface
            .front_convective_heat_flow(&state)
            .ok_or("Could not get front convective flow")?;
        let q_back = surface
            .back_convective_heat_flow(&state)
            .ok_or("Could not get back convective flow")?;

        // Expecting
        let ini = surfaces[0]
            .parent
            .first_node_temperature_index()
            .ok_or("Could not get first node index")?;
        let fin = surfaces[0]
            .parent
            .last_node_temperature_index()
            .ok_or("Could not get last node index")?
            + 1;
        let n_nodes = fin - ini;
        let mut temperatures = Matrix::new(0.0, n_nodes, 1);
        surfaces[0]
            .parent
            .get_node_temperatures(&state, &mut temperatures)?;

        println!(" T == {}", &temperatures);

        assert!(q_front > -3e-2, "q_in = {}", q_front);
        assert!(q_back < 3e-2, "q_out = {}", q_back);

        let full_qfront = q_front;
        let full_qback = q_back;

        assert!(
            (full_qfront + full_qback).abs() < 0.08,
            "q_front = {} | q_back = {} | delta = {}",
            full_qfront,
            full_qback,
            (full_qfront + full_qback).abs()
        );

        Ok(())
    }

    #[test]
    fn test_rk4() -> Result<(), String> {
        // Setup
        let c = Matrix::from_data(2, 2, vec![1., 0., 0., 1.]);
        let k = Matrix::from_data(2, 2, vec![1., -3., 4., -6.]);
        let q = Matrix::from_data(2, 1, vec![0., 0.]);

        // This system has a transient solution equals to c1*[0.75 1]'* e^(-3t) + c2*[1 1]' * e^(-2t)...

        // Find initial conditions so that C1 and C2 are 1.

        let temp_a_fn = |time: Float| 0.75 * (-3. * time).exp() + (-2. * time).exp();
        let temp_b_fn = |time: Float| (-3. * time).exp() + (-2. * time).exp();
        let mut memory = ChunkMemory {
            c,
            k,
            q,
            temps: Matrix::from_data(2, 1, vec![0.75 + 1., 2.]),
            // These are just to put intermediate data
            aux: Matrix::new(0.0, 2, 1),
            k1: Matrix::new(0.0, 2, 1),
            k2: Matrix::new(0.0, 2, 1),
            k3: Matrix::new(0.0, 2, 1),
            k4: Matrix::new(0.0, 2, 1),
        };
        let dt = 0.01;
        rearrange_k(dt, /*&c,*/ &mut memory)?;

        let mut time = 0.0;
        loop {
            let temp_a = memory.temps.get(0, 0)?;
            let exp_temp_a = temp_a_fn(time);
            let diff_a = (temp_a - exp_temp_a).abs();

            let temp_b = memory.temps.get(1, 0)?;
            let exp_temp_b = temp_b_fn(time);
            let diff_b = (temp_b - exp_temp_b).abs();
            #[cfg(feature = "float")]
            const SMOL: Float = 1e-2;
            #[cfg(not(feature = "float"))]
            const SMOL: Float = 1e-2;
            assert!(
                diff_a < SMOL,
                "temp_a = {} | exp_temp_a = {}, diff = {}",
                temp_a,
                exp_temp_a,
                diff_a
            );
            assert!(
                diff_b < SMOL,
                "temp_b = {} | exp_temp_b = {}, diff = {}",
                temp_b,
                exp_temp_b,
                diff_b
            );

            rk4(&mut memory)?;

            time += dt;

            if time > 100. {
                break;
            }
        }

        Ok(())
    }
}
