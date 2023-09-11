use crate::{Boundary, Float, SimulationState, SimulationStateHeader};
use geometry::{BBox3D, Loop3D, Polygon3D, Vector3D};
use matrix::Matrix;

/// A trait utilized to have shared behaviour between
/// Fenestration and Surfaces
pub trait SurfaceTrait: Clone + Send + Sync {
    /// Returns a reference to the name of the surface
    fn name(&self) -> &String;

    /// Returns the area in m2, calculated
    /// based on the [`Polygon3D`] that represents it
    fn area(&self) -> Float;

    /// Borrows a mutable reference to the [`Polygon3D`] describing
    /// this surface
    fn mut_vertices(&mut self) -> &mut Polygon3D;

    /// Borrows the outer loop
    fn outer(&self) -> &Loop3D;

    /// Borrows a mutable reference to the outer loop
    fn mut_outer(&mut self) -> &mut Loop3D;

    /// Gets the front boundary
    fn front_boundary(&self) -> &Boundary;

    /// Gets the back boundary
    fn back_boundary(&self) -> &Boundary;

    /// Sets the front boundary
    fn set_front_boundary(&mut self, boundary: Boundary);

    /// Sets the back boundary
    fn set_back_boundary(&mut self, boundary: Boundary);

    /// Gets the normal based on the [`Polygon3D`] that represents it
    fn normal(&self) -> Vector3D;

    /// Adds the front-convection state element
    fn add_front_convection_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the back-convection state element
    fn add_back_convection_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the front convective heat flow state element
    fn add_front_convective_heatflow_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;
    /// Adds the back convective heat flow state element
    fn add_back_convective_heatflow_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the front solar irradiance state element
    fn add_front_solar_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the back solar irradiance state element
    fn add_back_solar_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the front infra red solar irradiance state element
    fn add_front_ir_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the front infra red solar irradiance state element
    fn add_back_ir_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String>;

    /// Adds the temperature state elements for all the nodes in
    /// the [`Surface`] or [`Fenestration`]
    fn add_node_temperature_states(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
        n_nodes: usize,
    ) -> Result<(), String>;

    /// Gets the index (in the simulation state) of the temperature first (i.e., front) node
    fn first_node_temperature_index(&self) -> usize;

    /// Gets the index (in the simulation state) of the temperature last (i.e., back) node
    fn last_node_temperature_index(&self) -> usize;

    /// Gets a Bounding Box surrounding the surface
    fn bbox(&self) -> Result<BBox3D, String> {
        self.outer().bbox()
    }

    /// Gets the  temperature first (i.e., front) node
    fn front_temperature(&self, state: &SimulationState) -> Float {
        let i = self.first_node_temperature_index();
        state[i]
    }

    /// Gets the  temperature last (i.e., back) node
    fn back_temperature(&self, state: &SimulationState) -> Float {
        let i = self.last_node_temperature_index();
        state[i]
    }

    /// Retrieves a matrix with the temperatures in all the nodes
    fn get_node_temperatures(
        &self,
        state: &SimulationState,
        temperature_matrix: &mut Matrix,
    ) -> Result<(), String> {
        let ini = self.first_node_temperature_index();
        let fin = self.last_node_temperature_index() + 1;
        #[cfg(debug_assertions)]
        {
            let n_nodes = fin - ini;
            debug_assert_eq!(
                fin - ini,
                n_nodes,
                "get_node_temperatures()... setting data into a matrix of wrong size."
            );
        }
        for (node_index, i) in (ini..fin).enumerate() {
            let temp = state[i];
            temperature_matrix.set(node_index, 0, temp)?;
        }
        Ok(())
    }

    /// Sets the temperatures in all the nodes, based on a matrix
    fn set_node_temperatures(&self, state: &mut SimulationState, matrix: &Matrix) {
        let ini = self.first_node_temperature_index();
        let fin = self.last_node_temperature_index() + 1;

        for (node_index, i) in (ini..fin).enumerate() {
            let new_t = matrix.get(node_index, 0).unwrap();
            state[i] = new_t;
        }
    }

    /// Gets the front convection coefficient     
    fn front_convection_coefficient(&self, state: &SimulationState) -> Option<Float>;

    /// Gets the back convection coefficient
    fn back_convection_coefficient(&self, state: &SimulationState) -> Option<Float>;

    /// Sets the front convection coefficient
    fn set_front_convection_coefficient(
        &self,
        state: &mut SimulationState,
        v: Float,
    ) -> Result<(), String>;

    /// Sets the convective heat flow
    fn set_front_convective_heat_flow(
        &self,
        state: &mut SimulationState,
        v: Float,
    ) -> Result<(), String>;

    /// Sets the convective heat flow
    fn set_back_convective_heat_flow(
        &self,
        state: &mut SimulationState,
        v: Float,
    ) -> Result<(), String>;

    /// Sets the back convection coefficient
    fn set_back_convection_coefficient(
        &self,
        state: &mut SimulationState,
        v: Float,
    ) -> Result<(), String>;

    /// Gets the front solar irradiance
    fn front_solar_irradiance(&self, state: &SimulationState) -> Float;

    /// Gets the back solar irradiance
    fn back_solar_irradiance(&self, state: &SimulationState) -> Float;

    /// Gets the front IR irradiance
    fn front_infrared_irradiance(&self, state: &SimulationState) -> Float;

    /// Gets the back IR irradiance
    fn back_infrared_irradiance(&self, state: &SimulationState) -> Float;

    /// Gets the front precalculated_front_convection_coef
    fn fixed_front_hs(&self) -> Option<Float>;

    /// Gets the back precalculated_front_convection_coef
    fn fixed_back_hs(&self) -> Option<Float>;
}
