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

pub(crate) const MAX_RS: Float = 0.05;
use crate::convection::ConvectionParams;
use crate::Float;
use crate::{cavity::Cavity, surface::ChunkMemory};
use matrix::matrix::Matrix;
use model::{Construction, Model, Substance};
use std::sync::Arc;

/// Represents a thermal connection in the thermal network.
/// It can be a Cavity, a Solid, or other.
#[derive(Debug, Clone)]
pub enum UValue {
    /// A normal (i.e., $`\lambda/\Delta x`$) U-value
    Solid(Float),

    /// A cavity, comprised of a gas
    Cavity(Box<Cavity>),

    /// The resistance is a surface coefficient.
    Back,

    /// Undefined yet
    None,
}

impl UValue {
    /// Gets the U-value of a `UValue` object
    pub fn u_value(&self, t_before: Float, t_after: Float) -> Float {
        match self {
            Self::Solid(u) => *u,
            Self::Cavity(c) => c.u_value(t_before, t_after),
            Self::Back => 0., // This should be calculated appart
            Self::None => panic!("Attempting to get the u-value of None"),
        }
    }
}

impl std::default::Default for UValue {
    fn default() -> Self {
        UValue::None
    }
}

/// Represents the discretization of a [`Construction`] for heat transfer
/// calculation purposes.
///
/// > **Note:** This object contains all the [`Cavity`] objects in it, which
/// contain information not only about their thickness but also their orientation.
/// This means that one `Discretization` should exist per `Surface`, not just by `Construction`.
#[derive(Clone, Debug)]
pub struct Discretization {
    /// Contains the node's mass and the `UValue` of each segment
    pub segments: Vec<(Float, UValue)>,

    /// Contains the minimum number of timesteps per model timestep that
    /// this discretization requires to ensure numerical stability and accuracy.
    ///
    /// While the caller model—e.g., a Multiphysics Simulation model—may attempt
    /// to simulate at a certain timestep, some of the constructions in it may
    /// require smaller timesteps to ensure numerical stability and accuracy.
    /// This means that—on each timestep in the caller model—the thermal model needs
    /// to perform `time_subdivision` sub-timesteps.
    pub tstep_subdivision: usize,

    /// The number of elements on each layer
    pub n_elements: Vec<usize>,
}

impl Discretization {
    /// Creates a new `Discretization`.
    ///
    /// It first calculates the `tstep_subdivision` and the number of elements
    /// on each layer of Construction by calling `discretize_construction()`; and then builds the
    /// `Discretization` by calling `build()`.
    pub fn new(
        construction: &Arc<Construction>,
        model: &Model,
        model_dt: Float,
        max_dx: Float,
        min_dt: Float,
        height: Float,
        angle: Float,
    ) -> Result<Self, String> {
        let (tstep_subdivision, n_elements) =
            Self::discretize_construction(construction, model, model_dt, max_dx, min_dt)?;
        Self::build(
            construction,
            model,
            tstep_subdivision,
            n_elements,
            height,
            angle,
        )
    }

    /// Auxiliary function for `get_chunks()` function
    fn chunk_segments(&self, indexes: &[usize]) -> Vec<(usize, usize)> {
        if indexes.is_empty() {
            return Vec::with_capacity(0);
        }
        let mut start = indexes[0];
        let mut prev = start;
        let mut ret = Vec::new();
        for i in indexes.iter().skip(1) {
            assert!(*i > start);
            assert!(*i > prev);
            if *i - prev == 1 {
                prev = *i;
            } else {
                ret.push((start, prev + 1));
                start = *i;
                prev = start;
            }
        }
        ret.push((start, prev + 1));
        ret
    }

    /// Gets a a the segments that correspond to Massive and non-massive chunks.
    ///
    /// The purpose of this is that—when marching—we can know which nodes are massive
    /// and which ones are not, and thus solving them through different methods.
    #[allow(clippy::type_complexity)]
    pub fn get_chunks(&self) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
        let mass_nodes: Vec<usize> = self
            .segments
            .iter()
            .enumerate()
            .filter_map(|(i, x)| if x.0 >= 1e-5 { Some(i) } else { None })
            .collect();
        let nomass_nodes: Vec<usize> = self
            .segments
            .iter()
            .enumerate()
            .filter_map(|(i, x)| if x.0 < 1e-5 { Some(i) } else { None })
            .collect();
        let mass = self.chunk_segments(&mass_nodes);
        let nomass = self.chunk_segments(&nomass_nodes);
        (mass, nomass)
    }

    /// Creates the `segments` of the `Discretization`.
    fn build(
        construction: &Arc<Construction>,
        model: &Model,
        tstep_subdivision: usize,
        n_elements: Vec<usize>,
        height: Float,
        angle: Float,
    ) -> Result<Self, String> {
        debug_assert_eq!(n_elements.len(), construction.materials.len());

        // Let's start with an empty set of segments
        let mut n_nodes: usize = n_elements.iter().sum();
        // Add those that are Zero
        n_nodes += n_elements.iter().filter(|x| **x == 0).count() + 1;
        // n_nodes = n_nodes.max(construction.materials.len() + 1); // At least one per layer... but Zero means  "no_mass wall"

        let mut segments: Vec<(Float, UValue)> = vec![(0.0, UValue::default()); n_nodes];

        let mut n_segment = 0;
        for (n_layer, n) in n_elements.iter().enumerate() {
            let mut n = *n;

            let mat_name = &construction.materials[n_layer];
            let material = model.get_material(mat_name)?;
            let substance = model.get_substance(&material.substance)?;

            // get the mass of each segment.
            let mass = if n == 0 {
                0.0
            } else {
                match &substance {
                    Substance::Normal(s) => {
                        let dx = material.thickness / n as Float;
                        let rho = s.density()?;
                        let cp = s.specific_heat_capacity()?;
                        rho * cp * dx
                    }
                    Substance::Gas(_s) => 0.0, // should be zero... so should have been captured earlier
                }
            };

            if n == 0 {
                n = 1;
            }
            // Now iterate all segments...
            for _ in 0..n {
                match &substance {
                    Substance::Normal(s) => {
                        // Add mass to this and next nodes (if it is NoMass, it is Zero)
                        segments[n_segment].0 += mass / 2.;
                        segments[n_segment + 1].0 += mass / 2.;

                        // Add resistance
                        let dx = material.thickness / n as Float;
                        let k = s.thermal_conductivity()?;
                        // Push U-value
                        segments[n_segment].1 = UValue::Solid(k / dx);
                    }
                    Substance::Gas(s) => {
                        let gas = match s.gas() {
                            Ok(model::substance::gas::GasSpecification::Air) => crate::gas::AIR,
                            Ok(model::substance::gas::GasSpecification::Argon) => crate::gas::ARGON,
                            Ok(model::substance::gas::GasSpecification::Xenon) => crate::gas::XENON,
                            Ok(model::substance::gas::GasSpecification::Krypton) => {
                                crate::gas::KRYPTON
                            }
                            _ => {
                                return Err(format!(
                                    "Substance '{}' does not have a standard gas.",
                                    substance.name()
                                ))
                            }
                        };
                        if n_layer == 0 {
                            dbg!("This should be checked earlier.");
                            return Err(format!(
                                "Construction '{}' has a Gas as its first layer",
                                construction.name
                            ));
                        }
                        let prev_mat_name = construction.materials.get(n_layer - 1).unwrap(); // we already checked this
                                                                                              // let prev_mat = model.get_material(prev_mat_name)?;

                        let next_mat_name = match construction.materials.get(n_layer + 1) {
                            Some(v) => v,
                            None => {
                                return Err(format!(
                                    "Construction '{}' has a Gas as its last layer",
                                    construction.name
                                ))
                            }
                        };
                        // let next_mat = model.get_material(next_mat_name)?;
                        let next_substance = model.get_material_substance(next_mat_name)?;
                        let prev_substance = model.get_material_substance(prev_mat_name)?;

                        const DEFAULT_EM: Float = 0.84;
                        let ein = match &next_substance{
                            Substance::Normal(s)=>s.front_thermal_absorbtance_or(crate::heat_model::MODULE_NAME,DEFAULT_EM),
                            Substance::Gas(_)=>return Err(format!("Construction '{}' has two gases without a solid layer between them", construction.name))
                        };

                        let eout = match &prev_substance {
                            Substance::Normal(s)=>s.back_thermal_absorbtance_or(crate::heat_model::MODULE_NAME,DEFAULT_EM),
                            Substance::Gas(_)=>return Err(format!("Construction '{}' has two gases without a solid layer between them", construction.name))
                        };

                        let c = Cavity {
                            gas,
                            thickness: material.thickness,
                            height,
                            angle,
                            eout,
                            ein,
                        };
                        segments[n_segment].1 = UValue::Cavity(Box::new(c));
                    }
                }
                n_segment += 1;
            }
            // Process last one
            segments[n_nodes - 1].1 = UValue::Back;
        }

        Ok(Self {
            segments,
            tstep_subdivision,
            n_elements,
        })
    }

    /// Calculates the R value of the whole system
    ///
    /// # Panics
    /// Panics if the calculated R value is Zero (i.e., if there are no
    /// layers or something like that)
    pub fn r_value(&self) -> Float {
        let mut r = 0.0;

        for (_, u_value) in &self.segments {
            r += match u_value {
                UValue::Cavity(_c) => todo!(), //c.u_value(t_front, t_back),
                UValue::Solid(v) => 1. / v,
                UValue::Back => 0.0,
                UValue::None => unreachable!(),
            }
        }

        assert!(r > 0.0, "Found Zero r-value");
        r
    }

    /// Given a Maximum element thickness ($`\Delta x_{max}`$) and a minimum timestep ($`\Delta t_{min}`$), this function
    /// will find an arguibly good (i.e., stable and accurate) combination of $`\Delta t`$ and number of elements in each
    /// layer of the construction.
    ///
    /// This function recursively increases the model's timestep subdivisions (`n`) in order to reduce $`\Delta t`$ to numbers
    /// that respect the restrictions of (1) stability, (2) $`\Delta x_{max}`$, and (3) $`\Delta t_{min}`$. In other words,
    /// it searches (by testing $`\Delta t_{model}/1`$, $`\Delta t_{model}/2`$, $`\Delta t_{model}/3`$, ... $`\Delta t_{model}/n`$)
    /// for the minimum `n` that respects this restrictions
    ///
    /// # The math behind it
    ///
    /// > *I am not sure how correct this is... seems to work, but there is room for improvements and optimizations, surely*
    ///
    /// The first thing to know is that the walls in this module march
    /// through time using a 4th order [Runga-Kutte](https://en.wikipedia.org/wiki/Runge%E2%80%93Kutta_methods)
    /// (a.k.a., RK4). The second thing to know is that the RK4 method is
    /// more stable than the [Euler method](https://en.wikipedia.org/wiki/Euler_method),
    /// and thus the restrictions of stability for the Euler method can be considered
    /// to be a conservative restriction for the RK4. Hence, this function uses the
    /// Euler method restrictions.
    ///
    /// We are solving the following equation:
    ///
    /// ```math
    /// \dot{T} = \overline{C}^{-1} \overline{K}  T + \overline{C}^{-1} q
    /// ```
    ///
    /// And thus the stability of the numerical method will depend on the matrix:
    ///
    /// ```math
    /// \overline{K}^{\star} =\Delta t \overline{C}^{-1} \overline{K}
    /// ```
    ///
    /// Specifically, we don't want any of its [eigenvalues](https://en.wikipedia.org/wiki/Eigenvalues_and_eigenvectors)
    /// $`\xi_1, \xi_2,\xi_3, ...`$ to be outside of the Euler method's stability region. Since this
    /// matrix has only Real eigenvalues, this is equivalent to saying:
    ///
    /// ```math
    /// -2 < \xi_i < 0 ; \forall i
    /// ```
    ///
    /// However, finding the eigenvalues for $`\overline{K}^{\star}`$ is far from trivial. So there is
    /// yet another heuristic I am using: I am treating the case of a wall with 1 layer that is subdivided
    /// into a single element as the limit case. I am not sure if this is correct, but most of the instabilities
    /// I identified through Trial and Error corresponded to this case.
    ///
    /// For this limit case:
    /// * $`R = \frac{\Delta x}{\lambda}`$
    /// * $`C = \rho  c_p  \Delta x`$
    ///
    /// thus the value of $`\overline{K}^{\star}`$ is:
    ///
    /// ```math
    /// \overline{K}^{\star}=\begin{bmatrix}
    /// -\frac{\Delta t}{C\times R} - \frac{\Delta t}{C\times R_s} & \frac{\Delta t}{C\times R} \\
    ///  \frac{\Delta t}{C\times R} & -\frac{\Delta t}{C\times R} - \frac{\Delta t}{C\times R_s}\\
    /// \end{bmatrix}   
    ///```
    /// Note that, in that equation, $`R_{si} = R_{so}`$. The reason for this is that, at the point of using this method,
    /// we do not not know waht the values of $`R_{si}`$ and $`R_{so}`$ will be. TO overcome this, I use a
    /// value low enough to cover most general cases (`0.05`).
    ///
    /// Then, it can be found that the eigenvalues of this case—which we are treating as the limit case—are:
    /// ```math
    /// \xi_1 = -\frac{\Delta t} { R_s \rho c_p \Delta x }
    /// ```
    /// ```math
    /// \xi_2 = \xi_1 - 2 \frac{\Delta t \lambda}{ \rho  c_p  {\Delta x}^2}
    /// ```
    /// Both these values are always negative, so we don't need to worry about
    /// $`\xi_i < 0 `$. Also, it can be noticed that $`\xi_2 < \xi_1`$, meaning that
    /// what we need to comply with for Euler's stability criteria is:
    /// ```math
    /// -\frac{\Delta t} { R_s \rho c_p \Delta x } - 2 \frac{\Delta t \lambda}{ \rho  c_p  {\Delta x}^2} > -1
    /// ```
    ///
    /// Which means that the chosen $`\Delta x`$ must be greater than the (I am pretty sure, only) positive
    /// solution to equation:
    ///
    /// ```math
    /// 0 = {\Delta x}^2 - \left( \frac{\Delta t}{\rho c_p R_s} \right) \Delta x - \frac{2 \Delta t \lambda}{\rho c_p}
    /// ```
    ///
    /// So, this method will identify a combination of $`\Delta t`$ and $`\Delta x`$ that
    /// allows complying with this
    ///
    /// All that said, the value for $`\Delta t`$ actually used by the final model is
    /// actually half of what these equations use. This is because what we are using
    /// is a heuristic and I want to be safe... ish
    fn discretize_construction(
        construction: &Arc<Construction>,
        model: &Model,
        model_dt: Float,
        max_dx: Float,
        min_dt: Float,
    ) -> Result<(usize, Vec<usize>), String> {
        // I could only think of how to make this recursively... so I did this.
        fn aux(
            construction: &Arc<Construction>,
            model: &Model,
            main_dt: Float,
            n: usize,
            max_dx: Float,
            min_dt: Float,
        ) -> Result<(usize, Vec<usize>), String> {
            let dt = main_dt / (n as Float);

            // So, for each layer
            let n_layers = construction.materials.len();
            let mut n_elements: Vec<usize> = Vec::with_capacity(n_layers);

            for n_layer in 0..n_layers {
                let mat_name = &construction.materials[n_layer];
                let material = model.get_material(mat_name)?;
                let sub_name = &material.substance;
                let substance = model.get_substance(sub_name)?;

                // Calculate the minimum_dx
                let thickness = material.thickness;
                let (k, rho, cp) = match substance {
                    Substance::Normal(s) => {
                        let k = s.thermal_conductivity().map_err(|_x| "Trying to discretize a construction that contains a Normal Substance without a 'thermal conductivity'")?;
                        let rho = s.density().map_err(|_x| "Trying to discretize a construction that contains a Normal Substance without a 'density'")?;
                        let cp = s.specific_heat_capacity().map_err(|_x| "Trying to discretize a construction that contains a Normal Substance without a 'specific heat capacity'")?;
                        (*k, *rho, *cp)
                    }
                    Substance::Gas(_) => {
                        n_elements.push(0);
                        continue;
                    }
                };

                let a_coef = 1.;
                let b_coef = -dt / (rho * cp * MAX_RS);
                let c_coef = -2. * dt * k / (rho * cp);
                let disc = b_coef * b_coef - 4. * a_coef * c_coef;
                // this should never happen...?
                debug_assert!(disc >= 0.);

                // One solution is apparently always negative...
                // i.e. it is meaningless
                debug_assert!((-b_coef - disc.sqrt()) / (2. * a_coef) < 0.);

                // The positive solution is the one we care about
                let min_dx = (-b_coef + disc.sqrt()) / (2. * a_coef);

                if min_dx > thickness {
                    // This means that this layer cannot comply with the
                    // given timestep because its thickness leads to a dx that
                    // does not ensure convergence...
                    // check if there is room for reducing dt (hence reducing min_dx)
                    let next_dt = main_dt / ((n + 1) as Float);
                    if next_dt > min_dt {
                        // If there is room for that, do it.
                        return aux(construction, model, main_dt, n + 1, max_dx, min_dt);
                    } else {
                        // otherwise, mark this layer as no-mass
                        n_elements.push(0);
                    }
                } else {
                    // subdivide the layer, making all the elements of equal thickness
                    let m = (thickness / min_dx).floor();
                    // this case belongs to the other branch of this if/else
                    debug_assert!(m as usize != 0);
                    let dx = thickness / m;
                    if dx > max_dx {
                        // If the found dx is larger than the max allowed d_x, try to change timestep
                        // check if there is room for reducing dt...
                        let next_dt = main_dt / ((n + 1) as Float);
                        if next_dt > min_dt {
                            // If there is room for that, do it.
                            return aux(construction, model, main_dt, n + 1, max_dx, min_dt);
                        } else {
                            // otherwise, mark this layer as no-mass
                            n_elements.push(0);
                        }
                    } else {
                        // "dx" is smaller than max_dx, and thus this works
                        // fine.
                        n_elements.push(m as usize)
                    }
                }
            }

            // Check stability requirements...
            // stability is assured by (alpha * dt / dx^2 <= 1/2 )
            #[cfg(debug_assertions)]
            {
                for (n_layer, _) in n_elements.iter().enumerate() {
                    let mat_name = &construction.materials[n_layer];
                    let material = model.get_material(mat_name)?;
                    let sub_name = &material.substance;
                    let substance = model.get_substance(sub_name)?;

                    // Calculate the optimum_dx
                    let thickness = material.thickness;
                    let (k, rho, cp) = match substance {
                        Substance::Normal(s) => {
                            let k = s.thermal_conductivity().unwrap();
                            let rho = s.density().unwrap();
                            let cp = s.specific_heat_capacity().unwrap();
                            (*k, *rho, *cp)
                        }
                        Substance::Gas(_) => continue,
                    };
                    let dt = main_dt / n as Float;
                    let dx = thickness / n_elements[n_layer] as Float;

                    // assert!(alpha * dt / dx / dx <= 0.5);
                    let lambda1 = -dt / (MAX_RS * rho * cp * dx);
                    let r = dx / k;
                    let lambda2 = lambda1 - 2. * dt / (r * rho * cp * dx);
                    assert!(lambda1 >= -2.);
                    assert!(lambda1 <= 0.);
                    assert!(lambda2 >= -2.);
                    assert!(lambda2 <= 0.);
                }
            }

            // return
            Ok((n, n_elements))
        }
        aux(construction, model, model_dt, 1, max_dx, min_dt)
    }

    /// Produces $`\overline{K}`$ and $`\vec{q}`$ (as in the equation $`\overline{C} \dot{\vec{T}} =  \overline{K} \vec{T} + \vec{q}`$),
    /// allowing to update the node temperatures in a surface. This method returns the matrix $`\overline{k}`$
    /// representing the thermal network, and the vector $`\vec{q}`$), accounting for the heat flows induced by the
    /// environment.
    ///
    /// # How it works
    ///
    /// This matrix is constructed based ona discretization of the layers of the
    /// construction; that is, each layer is subdivided into `n` elements all of equal
    /// thickness. One node is placed at the beginning and end of each elements. Each element can
    /// be represented by the 2x2 matrix
    ///
    /// ```math
    /// \overline{K}=\begin{bmatrix} -U & U \\
    /// U & -U
    /// \end{bmatrix}   
    ///```
    ///
    /// Then—ignoring all external inputs (i.e., a system disconnected from the environment—the K matrix
    /// for a construction subdivided into 3 elements (i.e., 4 nodes) can be written as follows:
    ///
    /// ```math
    /// \overline{K}=\begin{bmatrix}
    /// -U_{1\rightarrow2} & U_{1\rightarrow2} & 0 & 0 \\
    /// U_{1\rightarrow2} & -U_{1\rightarrow2} - U_{2\rightarrow3} & U_{2\rightarrow3} & 0 \\
    /// 0 & U_{2\rightarrow3} & -U_{2\rightarrow3} - U_{3\rightarrow4} & U_{3\rightarrow4} \\
    /// 0 & 0 & U_{3\rightarrow4} & -U_{3\rightarrow4} \\
    /// \end{bmatrix}   
    /// ```
    ///
    /// Now, these nodes are also connected to a front and back border conditions.
    /// This means that the Matrix $`\overline{K}`$ needs to become:
    ///
    /// ```math
    /// \overline{K}=\begin{bmatrix}
    /// -U_{1\rightarrow2} - h_{so} & U_{1\rightarrow2} & 0 & 0 \\
    /// U_{1\rightarrow2} & -U_{1\rightarrow2} - U_{2\rightarrow3} & U_{2\rightarrow3} & 0 \\
    /// 0 & U_{2\rightarrow3} & -U_{2\rightarrow3} - U_{3\rightarrow4} & U_{3\rightarrow4} \\
    /// 0 & 0 & U_{3\rightarrow4} & -U_{3\rightarrow4}- h_{si} \\
    /// \end{bmatrix}   
    /// ```
    ///
    /// Additionally, the vector $`\vec{q}`$ will have to account for the border conditions. How this is done depends
    /// on the border condition. If the border leads to a zone, a value of $`  T_{env} h_s + E_{ir}\epsilon_s - \epsilon_s \sigma {T_s}^4`$
    /// should be added to $`\vec{q}`$.  Note that $`T_{env}`$ is the temparture of the air in the environment,
    /// $`h_s`$ is the convection coefficient, $`E_{ir}`$ is the incident infrared radiation and $`\epsilon_s`$ is the
    /// emissivity of the surface. On the contrary, if the border condition is a cavity, then a value of $`T_{pane} U_{cavity}`$
    /// should be added. $`T_{pane}`$ is the temperature of the surface before or after.
    ///         
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn get_k_q(
        &self,
        ini: usize,
        fin: usize,
        temperatures: &Matrix,
        front_env: &ConvectionParams,
        front_hs: Float,
        front_rad_hs: Float,
        back_env: &ConvectionParams,
        back_hs: Float,
        back_rad_hs: Float,
        memory: &mut ChunkMemory,
    ) -> Result<(), String> {
        let (nrows, ncols) = temperatures.size();
        assert_eq!(
            ncols, 1,
            "Expecting `temperatures` to be a Column matrix... but found {} columns",
            ncols
        );
        assert_eq!(
            nrows,
            self.segments.len(),
            "Expecting `temperatures` to have {} elements... found {}",
            self.segments.len(),
            nrows
        );
        memory.k *= 0.0;
        memory.q *= 0.0;
        let nnodes = fin - ini;

        // this is just quite helpful
        let get_t_after = |i: usize| -> Float {
            match temperatures.get(i + 1, 0) {
                Ok(v) => v,
                Err(_) => back_env.air_temperature,
            }
        };

        for local_i in 0..nnodes - 1 {
            let global_i = ini + local_i;
            let t_this = temperatures.get(global_i, 0)?;
            let t_next = get_t_after(global_i);
            let (.., uvalue) = &self.segments[global_i];
            let u = uvalue.u_value(t_this, t_next);

            // Top left... should be there
            memory.k.add_to_element(local_i, local_i, -u)?;
            // Bottom right, only if within range.
            if let Ok(old_value) = memory.k.get(local_i + 1, local_i + 1) {
                memory.k.set(local_i + 1, local_i + 1, old_value - u)?;
            }
            // Top right, only if within range.
            if let Ok(old_value) = memory.k.get(local_i, local_i + 1) {
                memory.k.set(local_i, local_i + 1, old_value + u)?;
            }
            // Bottom left, only if within range.
            if let Ok(old_value) = memory.k.get(local_i + 1, local_i) {
                memory.k.set(local_i + 1, local_i, old_value + u)?;
            }
        }

        // Add front border conditions
        let (hs_front, front_q) = if ini == 0 {
            let ts = temperatures.get(0, 0)?;
            // Solar radiation is added later because it also depends
            // on the solar absorption of different layers.

            let front_q = front_env.air_temperature * front_hs  // convection
                + front_rad_hs * (front_env.rad_temperature - ts);

            (front_hs, front_q)
        } else {
            let (.., uvalue) = &self.segments[ini - 1];
            let t_before = temperatures.get(ini - 1, 0)?; // this should NEVER fail
            let t_after = temperatures.get(ini, 0)?; // this should NEVER fail
            let u = uvalue.u_value(t_before, t_after);

            (u, u * t_before)
        };

        memory.q.add_to_element(0, 0, front_q).unwrap();
        memory.k.add_to_element(0, 0, -hs_front).unwrap();

        // Add back border conditions
        let (hs_back, back_q) = if fin == nrows {
            let ts = temperatures.get(fin - 1, 0).unwrap();
            // Solar radiation is added later because it also depends
            // on the solar absorption of different layers.
            let back_q = back_env.air_temperature * back_hs  // convection
                + back_rad_hs * (back_env.rad_temperature - ts);

            (back_hs, back_q)
        } else {
            let (.., uvalue) = &self.segments[fin - 1];
            let t_before = temperatures.get(fin - 1, 0)?; // this should NEVER fail
            let t_after = get_t_after(fin - 1);
            let u = uvalue.u_value(t_before, t_after);

            (u, u * t_after)
        };
        memory.q.add_to_element(nnodes - 1, 0, back_q)?;
        memory.k.add_to_element(nnodes - 1, nnodes - 1, -hs_back)?;

        Ok(())
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    impl std::default::Default for ConvectionParams {
        fn default() -> Self {
            // const DEFAULT_ENV_EMISSIVITY: Float = 1.;
            const DEFAULT_AIR_TEMP: Float = 22.;
            ConvectionParams {
                air_temperature: DEFAULT_AIR_TEMP,
                surface_temperature: DEFAULT_AIR_TEMP,
                air_speed: 0.,
                rad_temperature: DEFAULT_AIR_TEMP,
                roughness_index: 1,
                cos_surface_tilt: 0.0,
            }
        }
    }

    /// Creates a Construction with a single layer of Material, of a certain Substance.
    /// Returns a model containing them and also an Rc to the construction
    fn get_normal(
        thermal_cond: Float,
        density: Float,
        cp: Float,
        thickness: Float,
    ) -> (Model, Arc<Construction>) {
        let mut model = Model::default();

        let mut s = model::substance::Normal::new("the substance");
        s.set_thermal_conductivity(thermal_cond)
            .set_density(density)
            .set_specific_heat_capacity(cp);
        let s = s.wrap();
        let s = model.add_substance(s);

        let material = model::Material::new(
            "the mat".to_string(), // mat name
            s.name().clone(),      // substance name
            thickness,             // thickness
        );
        let material = model.add_material(material);

        let mut construction = Construction::new("the construction");
        construction.materials.push(material.name().clone());
        let construction = model.add_construction(construction);
        (model, construction)
    }

    #[test]
    fn build_normal_mass() {
        let thermal_cond = 1.;
        let density = 2.1;
        let cp = 1.312;
        let thickness = 12.5 / 1000.;
        let tstep_sub = 10;

        let (model, construction) = get_normal(thermal_cond, density, cp, thickness);
        let d = Discretization::build(&construction, &model, tstep_sub, vec![1], 1., 0.).unwrap();
        // normal --> linear

        assert_eq!(d.tstep_subdivision, tstep_sub);
        assert_eq!(d.segments.len(), 2);

        // mass of node 0
        let exp_mass = thickness * density * cp / 2.;
        let mass = d.segments[0].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Solid(u) = d.segments[0].1 {
            assert!((u - thermal_cond / thickness).abs() < 1e-16)
        } else {
            panic!("Expecting Solid!")
        }

        let mass = d.segments[1].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        assert!(matches!(d.segments[1].1, UValue::Back));
    }

    #[test]
    fn test_build_normal_no_mass() {
        let thermal_cond = 1.;
        let density = 2.1;
        let cp = 1.312;
        let thickness = 12.5 / 1000.;
        let tstep_sub = 10;

        let (model, construction) = get_normal(thermal_cond, density, cp, thickness);

        let d = Discretization::build(&construction, &model, tstep_sub, vec![0], 1., 0.).unwrap();

        // normal --> linear
        assert_eq!(d.tstep_subdivision, tstep_sub);
        assert_eq!(d.segments.len(), 2);
        // mass of node 0
        let exp_mass = 0.0;
        let mass = d.segments[0].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Solid(u) = d.segments[0].1 {
            assert!((u - thermal_cond / thickness).abs() < 1e-16)
        } else {
            panic!("Expecting Solid!")
        }

        let mass = d.segments[1].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Back = d.segments[1].1 {
            assert!(true)
        } else {
            panic!("Expecting Back!")
        }
    }

    #[test]
    fn build_normal_gas_normal_mass() {
        let thermal_cond = 1.;
        let density = 2.1;
        let cp = 1.312;
        let thickness = 12.5 / 1000.;
        let mut model = Model::default();

        let tstep_sub = 10;

        // Create normal substance
        ///////////////////////////
        let mut substance = model::substance::Normal::new("the substance");
        substance
            .set_thermal_conductivity(thermal_cond)
            .set_density(density)
            .set_front_thermal_absorbtance(0.9)
            .set_back_thermal_absorbtance(0.8)
            .set_specific_heat_capacity(cp);
        let substance = substance.wrap();
        let substance = model.add_substance(substance); // push to model

        // Create normal Material
        ///////////////////////////
        let normal =
            model::Material::new("the mat".to_string(), substance.name().clone(), thickness);
        let normal = model.add_material(normal);

        // Create Gas substance
        ///////////////////////////
        let mut gas = model::substance::Gas::new("the gas");
        gas.set_gas(model::substance::gas::GasSpecification::Air);
        let gas = gas.wrap();
        let gas = model.add_substance(gas);

        // Create Gas Material
        ///////////////////////////
        let gas = model::Material::new(
            "the_gas".to_string(), // name of gas
            gas.name().clone(),    // name of substance
            thickness,
        );
        let gas = model.add_material(gas);

        // Assemble construction
        ///////////////////////////
        let mut construction = model::Construction::new("the construction");
        construction.materials.push(normal.name().clone());
        construction.materials.push(gas.name().clone());
        construction.materials.push(normal.name().clone());
        let construction = model.add_construction(construction);

        // Test
        ///////////////////////////
        let d =
            Discretization::build(&construction, &model, tstep_sub, vec![1, 1, 1], 1., 0.).unwrap();

        // has gas --> linear
        assert_eq!(d.tstep_subdivision, tstep_sub);
        assert_eq!(d.segments.len(), 4); // normal, gas, normal, back

        // node 0
        let exp_mass = thickness * density * cp / 2.;
        let mass = d.segments[0].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Solid(u) = d.segments[0].1 {
            assert!((u - thermal_cond / thickness).abs() < 1e-16)
        } else {
            panic!("Expecting Solid!")
        }

        // node 1
        let mass = d.segments[1].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Cavity(c) = &d.segments[1].1 {
            let u = c.u_value(10., 10.);
            dbg!(u);
        } else {
            panic!("Expecting a Cavity!")
        }

        // node 2
        let mass = d.segments[2].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Solid(u) = d.segments[2].1 {
            assert!((u - thermal_cond / thickness).abs() < 1e-16)
        } else {
            panic!("Expecting Solid!")
        }

        // node 3
        let mass = d.segments[3].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Back = d.segments[3].1 {
            assert!(true)
        } else {
            panic!("Expecting Solid!")
        }
    }

    #[test]
    fn build_normal_gas_normal_no_mass() {
        let thermal_cond = 1.;
        let density = 2.1;
        let cp = 1.312;
        let thickness = 12.5 / 1000.;

        let mut model = Model::default();

        let tstep_sub = 10;

        // Create normal Substance
        ///////////////////////////
        let mut normal = model::substance::Normal::new("the substance");
        normal
            .set_thermal_conductivity(thermal_cond)
            .set_density(density)
            .set_specific_heat_capacity(cp);
        let normal = normal.wrap();
        let normal = model.add_substance(normal); // push to model

        // Create normal Material
        ///////////////////////////
        let normal = model::Material::new(
            "the mat".to_string(), // name of material
            normal.name().clone(), // name of substance
            thickness,
        );
        let normal = model.add_material(normal); // push to model

        // Create gas substance
        ///////////////////////////
        let mut gas = model::substance::Gas::new("the gas");
        gas.set_gas(model::substance::gas::GasSpecification::Air);
        let gas = gas.wrap();
        let gas = model.add_substance(gas); // push to model

        // Create gas Material
        ///////////////////////////
        let gas = model::Material::new("the_gas".to_string(), gas.name().clone(), thickness);
        let gas = model.add_material(gas); // push to model

        // Assemble Construction
        ///////////////////////////
        let mut construction = model::Construction::new("the construction");
        construction.materials.push(normal.name().clone());
        construction.materials.push(gas.name().clone());
        construction.materials.push(normal.name().clone());
        let construction = model.add_construction(construction); // push to model

        // Test
        ///////////////////////////
        let d =
            Discretization::build(&construction, &model, tstep_sub, vec![0, 0, 0], 1., 0.).unwrap();

        // has gas --> linear
        assert_eq!(d.tstep_subdivision, tstep_sub);
        assert_eq!(d.segments.len(), 4); // normal, gas, normal, back

        // node 0
        let exp_mass = 0.0;
        let mass = d.segments[0].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Solid(u) = d.segments[0].1 {
            assert!((u - thermal_cond / thickness).abs() < 1e-16)
        } else {
            panic!("Expecting Solid!")
        }

        // node 1
        let mass = d.segments[1].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Cavity(c) = &d.segments[1].1 {
            // assert!( (r - thickness/thermal_cond).abs() < 1e-16 )
            let u = c.u_value(10., 20.);
            dbg!(u);
        } else {
            panic!("Expecting Solid!")
        }

        // node 2
        let mass = d.segments[2].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Solid(u) = d.segments[2].1 {
            assert!(
                (u - thermal_cond / thickness).abs() < 1e-16,
                "u = {} | exp = {}",
                u,
                thermal_cond / thickness
            )
        } else {
            panic!("Expecting Solid!")
        }

        // node 3
        let mass = d.segments[3].0;
        assert!(
            (exp_mass - mass).abs() < 1e-17,
            "Expecting mass to be {exp_mass}... found {mass}"
        );
        if let UValue::Back = d.segments[3].1 {
            assert!(true)
        } else {
            panic!("Expecting Solid!")
        }
    }

    fn get_solid_test_system(
        thickness: Float,
        thermal_cond: Float,
    ) -> (
        Discretization,
        Matrix,
        ConvectionParams,
        // Float,
        Float,
        ConvectionParams,
        // Float,
        Float,
    ) {
        let n = 5;
        let dx = thickness / n as Float;
        let u = thermal_cond / dx;

        let mut segments = Vec::with_capacity(n + 1);
        for mass in 0..n {
            segments.push((mass as Float, UValue::Solid(u)));
        }
        segments.push((n as Float, UValue::Back));
        assert_eq!(segments.len(), n + 1);

        let d = Discretization {
            segments,
            tstep_subdivision: 1,
            n_elements: vec![n],
        };

        let front_env = ConvectionParams {
            air_temperature: 0.,
            air_speed: 0.,
            rad_temperature: 0.0,
            ..ConvectionParams::default()
        };

        let back_env = ConvectionParams {
            air_temperature: 7.,
            air_speed: 0.,
            rad_temperature: 5.,
            ..ConvectionParams::default()
        };

        let temperatures = Matrix::from_data(n + 1, 1, vec![1., 2., 3., 4., 5., 6.]);
        let front_hs = front_env.get_tarp_natural_convection_coefficient();
        let back_hs = back_env.get_tarp_convection_coefficient(1., 4., false);

        return (d, temperatures, front_env, front_hs, back_env, back_hs);
    }

    #[test]
    fn test_get_q_k_solid() {
        let n = 5;
        let thickness = 0.5;
        let thermal_cond = 2.12;

        let dx = thickness / n as Float;
        let u = thermal_cond / dx;
        let (d, temperatures, front_env, front_hs, back_env, back_hs) =
            get_solid_test_system(thickness, thermal_cond);

        let front_rad_hs = 1.0;
        let back_rad_hs = 1.0;
        let mut memory = ChunkMemory {
            aux: Matrix::new(0.0, n + 1, 1),
            k: Matrix::new(0.0, n + 1, n + 1),
            c: Matrix::new(0.0, n + 1, n + 1),
            q: Matrix::new(0.0, n + 1, 1),
            temps: Matrix::new(0.0, n + 1, 1),
            k1: Matrix::new(0.0, n + 1, 1),
            k2: Matrix::new(0.0, n + 1, 1),
            k3: Matrix::new(0.0, n + 1, 1),
            k4: Matrix::new(0.0, n + 1, 1),
        };
        d.get_k_q(
            0,
            n + 1,
            &temperatures,
            &front_env,
            front_hs,
            front_rad_hs,
            &back_env,
            back_hs,
            back_rad_hs,
            &mut memory,
        )
        .unwrap();
        println!("k = {}", memory.k);
        println!("heat_flows = {}", memory.q);

        for r in 0..n + 1 {
            // Check q
            let heat_flow = memory.q.get(r, 0).unwrap();
            if r == 0 {
                // exterior temp is lower, so heat flow is negative
                assert!(heat_flow < -1e-5);
            } else if r == n {
                assert!(heat_flow > 1e-5);
            } else {
                // it is Zero
                assert!(heat_flow.abs() < 1e-29);
            }

            for c in 0..n + 1 {
                let v = memory.k.get(r, c).unwrap();

                if c == r {
                    // Diagonal
                    if c == 0 {
                        // Exterior node...
                        assert!((v - (-front_hs - u)).abs() < 1e-20);
                    } else if c == n {
                        // interior node
                        assert!((v - (-back_hs - u)).abs() < 1e-20);
                    } else {
                        // Any other node.
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    }
                } else {
                    // Not diagonal
                    let r = r as i32;
                    let c = c as i32;
                    if (c - r).abs() <= 1 {
                        // if it is within tri-diagonal
                        assert!((v - u).abs() < 1e-20);
                    } else {
                        // if not
                        assert!(v.abs() < 1e-29);
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_q_k_solid_partial() {
        let n = 5;
        let thickness = 0.5;
        let thermal_cond = 2.12;

        let dx = thickness / n as Float;
        let u = thermal_cond / dx;
        let (d, temperatures, front_env, front_hs, back_env, back_hs) =
            get_solid_test_system(thickness, thermal_cond);

        let front_rad_hs = 1.0;
        let back_rad_hs = 1.0;
        let mut memory = ChunkMemory {
            aux: Matrix::new(0.0, n + 1, 1),
            k: Matrix::new(0.0, n + 1, n + 1),
            c: Matrix::new(0.0, n + 1, n + 1),
            q: Matrix::new(0.0, n + 1, 1),
            temps: Matrix::new(0.0, n + 1, 1),
            k1: Matrix::new(0.0, n + 1, 1),
            k2: Matrix::new(0.0, n + 1, 1),
            k3: Matrix::new(0.0, n + 1, 1),
            k4: Matrix::new(0.0, n + 1, 1),
        };

        d.get_k_q(
            0,
            3,
            &temperatures,
            &front_env,
            front_hs,
            front_rad_hs,
            &back_env,
            back_hs,
            back_rad_hs,
            &mut memory,
        )
        .unwrap();
        println!("k = {}", memory.k);
        println!("heat_flows = {}", memory.q);

        for r in 0..3 {
            // Check q
            let heat_flow = memory.q.get(r, 0).unwrap();
            if r == 0 {
                // exterior temp is lower, so heat flow is negative
                assert!(heat_flow < -1e-5);
            } else if r == 2 {
                assert!(heat_flow > 1e-5);
            } else {
                // it is Zero
                assert!(heat_flow.abs() < 1e-29);
            }

            for c in 0..3 {
                let v = memory.k.get(r, c).unwrap();

                if c == r {
                    // Diagonal
                    if c == 0 {
                        // Exterior node...
                        assert!((v - (-front_hs - u)).abs() < 1e-20);
                    } else if c == n {
                        // interior node
                        assert!((v - (-back_hs - u)).abs() < 1e-20);
                    } else {
                        // Any other node.
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    }
                } else {
                    // Not diagonal
                    let r = r as i32;
                    let c = c as i32;
                    if (c - r).abs() <= 1 {
                        // if it is within tri-diagonal
                        assert!((v - u).abs() < 1e-20);
                    } else {
                        // if not
                        assert!(v.abs() < 1e-29);
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_q_k_solid_partial_2() {
        let n = 5;
        let thickness = 0.5;
        let thermal_cond = 0.1;

        let dx = thickness / n as Float;
        let u = thermal_cond / dx;
        let (d, temperatures, front_env, front_hs, back_env, back_hs) =
            get_solid_test_system(thickness, thermal_cond);

        let front_rad_hs = 1.0;
        let back_rad_hs = 1.0;
        let mut memory = ChunkMemory {
            aux: Matrix::new(0.0, n + 1, 1),
            k: Matrix::new(0.0, n + 1, n + 1),
            c: Matrix::new(0.0, n + 1, n + 1),
            q: Matrix::new(0.0, n + 1, 1),
            temps: Matrix::new(0.0, n + 1, 1),
            k1: Matrix::new(0.0, n + 1, 1),
            k2: Matrix::new(0.0, n + 1, 1),
            k3: Matrix::new(0.0, n + 1, 1),
            k4: Matrix::new(0.0, n + 1, 1),
        };
        d.get_k_q(
            2,
            5,
            &temperatures,
            &front_env,
            front_hs,
            front_rad_hs,
            &back_env,
            back_hs,
            back_rad_hs,
            &mut memory,
        )
        .unwrap();
        println!("k = {}", memory.k);
        println!("heat_flows = {}", memory.q);

        for r in 0..3 {
            // Check q
            let heat_flow = memory.q.get(r, 0).unwrap();
            if r == 1 {
                // This element is Zero
                assert!(heat_flow.abs() < 1e-29);
            } else {
                // This should not be Zero
                assert!(heat_flow.abs() > 1e-3);
            }

            for c in 0..3 {
                let v = memory.k.get(r, c).unwrap();

                if c == r {
                    // Diagonal
                    if c == 0 {
                        // Exterior node...
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    } else if c == n {
                        // interior node
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    } else {
                        // Any other node.
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    }
                } else {
                    // Not diagonal
                    let r = r as i32;
                    let c = c as i32;
                    if (c - r).abs() <= 1 {
                        // if it is within tri-diagonal
                        assert!((v - u).abs() < 1e-20);
                    } else {
                        // if not
                        assert!(v.abs() < 1e-29);
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_q_k_partial() {
        let thickness = 0.5;
        let n = 5;
        let dx = thickness / n as Float;
        let thermal_cond = 1.;
        let u = thermal_cond / dx;

        let mut segments = Vec::with_capacity(n + 1);
        for _ in 0..n {
            segments.push((0.0, UValue::Solid(u)));
        }
        segments.push((0.0, UValue::Back));
        assert_eq!(segments.len(), n + 1);

        let d = Discretization {
            segments,
            tstep_subdivision: 1,
            n_elements: vec![n],
        };

        let front_env = ConvectionParams {
            air_temperature: 1.,
            air_speed: 0.,
            rad_temperature: 0.,
            ..ConvectionParams::default()
        };

        let back_env = ConvectionParams {
            air_temperature: 6.,
            air_speed: 0.,
            rad_temperature: 5.,
            ..ConvectionParams::default()
        };

        let temperatures = Matrix::from_data(n + 1, 1, vec![1., 2., 3., 4., 5., 6.]);
        let front_hs = 1.739658084820765;
        let back_hs = 1.739658084820765;
        let front_rad_hs = 1.0;
        let back_rad_hs = 1.0;
        let mut memory = ChunkMemory {
            aux: Matrix::new(0.0, n + 1, 1),
            k: Matrix::new(0.0, n + 1, n + 1),
            c: Matrix::new(0.0, n + 1, n + 1),
            q: Matrix::new(0.0, n + 1, 1),
            temps: Matrix::new(0.0, n + 1, 1),
            k1: Matrix::new(0.0, n + 1, 1),
            k2: Matrix::new(0.0, n + 1, 1),
            k3: Matrix::new(0.0, n + 1, 1),
            k4: Matrix::new(0.0, n + 1, 1),
        };
        d.get_k_q(
            1,
            n,
            &temperatures,
            &front_env,
            front_hs,
            front_rad_hs,
            &back_env,
            back_hs,
            back_rad_hs,
            &mut memory,
        )
        .unwrap();
        println!("k = {}", memory.k);
        println!("heat_flows = {}", memory.q);

        for r in 0..n - 1 {
            // Check q
            let heat_flow = memory.q.get(r, 0).unwrap();
            if r == 0 {
                assert!(heat_flow > 1e-5);
            } else if r == n - 2 {
                assert!(heat_flow > 1e-5);
            } else {
                assert!(heat_flow.abs() < 1e-29);
            }

            for c in 0..n - 1 {
                let v = memory.k.get(r, c).unwrap();

                if c == r {
                    // Diagonal
                    if c == 0 {
                        // Exterior node
                        assert!((v + 2. * u).abs() < 1e-20, "v = {} | found = {}", v, 2. * u);
                    } else if c == n - 2 {
                        // interior node
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    } else {
                        // Any other node.
                        assert!((v - (-2. * u)).abs() < 1e-20);
                    }
                } else {
                    // Not diagonal
                    let r = r as i32;
                    let c = c as i32;
                    if (c - r).abs() <= 1 {
                        // if it is within tri-diagonal
                        assert!((v - u).abs() < 1e-20);
                    } else {
                        // if not
                        assert!(v.abs() < 1e-29);
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_chunks() {
        // Single node, massive
        let d = Discretization {
            tstep_subdivision: 1,
            segments: vec![(1., UValue::None)],
            n_elements: vec![1], // Does not matter for this test
        };

        let (mass_chunks, nomass_chunks) = d.get_chunks();
        assert!(nomass_chunks.is_empty());
        assert_eq!(mass_chunks.len(), 1);
        assert_eq!(mass_chunks, vec![(0, 1)]);

        // Single node, no-mass
        let d = Discretization {
            tstep_subdivision: 1,
            segments: vec![(0., UValue::None)],
            n_elements: vec![1], // Does not matter for this test
        };

        let (mass_chunks, nomass_chunks) = d.get_chunks();
        assert!(mass_chunks.is_empty());
        assert_eq!(nomass_chunks.len(), 1);
        assert_eq!(nomass_chunks, vec![(0, 1)]);

        // Several nodes, massive
        let d = Discretization {
            tstep_subdivision: 1,
            segments: vec![(1., UValue::None); 10],
            n_elements: vec![1], // Does not matter for this test
        };

        let (mass_chunks, nomass_chunks) = d.get_chunks();
        assert!(nomass_chunks.is_empty());
        assert_eq!(mass_chunks.len(), 1);
        assert_eq!(mass_chunks, vec![(0, 10)]);

        // Several nodes, no-mass
        let d = Discretization {
            tstep_subdivision: 1,
            segments: vec![(0., UValue::None); 10],
            n_elements: vec![1], // Does not matter for this test
        };

        let (mass_chunks, nomass_chunks) = d.get_chunks();
        assert!(mass_chunks.is_empty());
        assert_eq!(nomass_chunks.len(), 1);
        assert_eq!(nomass_chunks, vec![(0, 10)]);

        // Mixed 1
        let d = Discretization {
            tstep_subdivision: 1,
            segments: vec![
                (0., UValue::None),
                (1., UValue::None),
                (1., UValue::None),
                (0., UValue::None),
                (0., UValue::None),
            ],
            n_elements: vec![0, 1, 1, 0, 0], // Does not matter for this test
        };

        let (mass_chunks, nomass_chunks) = d.get_chunks();
        assert_eq!(mass_chunks.len(), 1);
        assert_eq!(mass_chunks, vec![(1, 3)]);
        assert_eq!(nomass_chunks.len(), 2);
        assert_eq!(nomass_chunks, vec![(0, 1), (3, 5)]);

        // Mixed 2
        let d = Discretization {
            tstep_subdivision: 1,
            segments: vec![
                (1., UValue::None),
                (1., UValue::None),
                (1., UValue::None),
                (0., UValue::None),
                (0., UValue::None),
            ],
            n_elements: vec![1, 1, 1, 0, 0], // Does not matter for this test
        };

        let (mass_chunks, nomass_chunks) = d.get_chunks();
        assert_eq!(mass_chunks.len(), 1);
        assert_eq!(mass_chunks, vec![(0, 3)]);
        assert_eq!(nomass_chunks.len(), 1);
        assert_eq!(nomass_chunks, vec![(3, 5)]);
    }
}
