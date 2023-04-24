use crate::Float;
use matrix::Matrix;
use model::{
    Fenestration, SimulationState, SimulationStateElement, SimulationStateHeader, Surface,
};

/// A trait for defining shared behaviour between [`Surface`] and
/// [`Fenestration`] objects
pub trait SurfaceTrait : Clone + Send + Sync  {
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
    fn set_front_convective_heat_flow(&self, state: &mut SimulationState, v: Float)->Result<(),String>;

    /// Sets the convective heat flow
    fn set_back_convective_heat_flow(&self, state: &mut SimulationState, v: Float)->Result<(),String>;

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
}

impl SurfaceTrait for Surface {

    fn set_front_convective_heat_flow(&self, state: &mut SimulationState, v: Float)->Result<(),String>{
        self.set_front_convective_heat_flow(state, v)
    }

    fn set_back_convective_heat_flow(&self, state: &mut SimulationState, v: Float)->Result<(),String>{
        self.set_back_convective_heat_flow(state, v)
    }
    


    fn front_infrared_irradiance(&self, state: &SimulationState) -> Float {
        self.front_ir_irradiance(state).unwrap()
    }
    fn back_infrared_irradiance(&self, state: &SimulationState) -> Float {
        self.back_ir_irradiance(state).unwrap()
    }

    fn front_solar_irradiance(&self, state: &SimulationState) -> Float {
        self.front_incident_solar_irradiance(state).unwrap()
    }
    fn back_solar_irradiance(&self, state: &SimulationState) -> Float {
        self.back_incident_solar_irradiance(state).unwrap()
    }

    fn set_front_convection_coefficient(
        &self,
        _state: &mut SimulationState,
        _v: Float,
    ) -> Result<(), String> {
        self.set_front_convection_coefficient(_state, _v)
    }
    fn set_back_convection_coefficient(
        &self,
        _state: &mut SimulationState,
        _v: Float,
    ) -> Result<(), String> {
        self.set_back_convection_coefficient(_state, _v)
    }

    fn front_convection_coefficient(&self, _state: &SimulationState) -> Option<Float> {
        self.front_convection_coefficient(_state)
    }
    fn back_convection_coefficient(&self, _state: &SimulationState) -> Option<Float> {
        self.back_convection_coefficient(_state)
    }

    fn first_node_temperature_index(&self) -> usize {
        self.first_node_temperature_index()
            .expect("Could not get first node index in surface")
    }
    fn last_node_temperature_index(&self) -> usize {
        self.last_node_temperature_index()
            .expect("Could not get last node index in surface")
    }

    fn add_front_convection_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_convection_coefficient_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceFrontConvectionCoefficient(ref_surface_index),
                1.739658084820765,
            )?;
            self.set_front_convection_coefficient_index(i)?;
            Ok(())
        } else {
            Err("SurfaceFrontConvectionCoefficient already in surface".into())
        }
    }

    fn add_back_convection_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_convection_coefficient_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceBackConvectionCoefficient(ref_surface_index),
                1.739658084820765,
            )?;
            self.set_back_convection_coefficient_index(i)?;
            Ok(())
        } else {
            Err("SurfaceBackConvectionCoefficient already in surface".into())
        }
    }

    fn add_front_convective_heatflow_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_convective_heat_flow_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceFrontConvectiveHeatFlow(ref_surface_index),
                0.0,
            )?;
            self.set_front_convective_heat_flow_index(i)?;
            Ok(())
        } else {
            Err("SurfaceFrontConvectiveHeatFlow already in surface".into())
        }
    }
    fn add_back_convective_heatflow_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_convective_heat_flow_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceBackConvectiveHeatFlow(ref_surface_index),
                0.0,
            )?;
            self.set_back_convective_heat_flow_index(i)?;
            Ok(())
        } else {
            Err("SurfaceBackConvectiveHeatFlow already in surface".into())
        }
    }

    fn add_front_solar_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_incident_solar_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceFrontSolarIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_front_incident_solar_irradiance_index(i)?;
            Ok(())
        } else {
            Err("SurfaceFrontSolarIrradiance already in surface".into())
        }
    }
    fn add_back_solar_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_incident_solar_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceBackSolarIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_back_incident_solar_irradiance_index(i)?;
            Ok(())
        } else {
            Err("SurfaceBackSolarIrradiance already in surface".into())
        }
    }

    fn add_front_ir_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_ir_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceFrontIRIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_front_ir_irradiance_index(i)?;
            Ok(())
        } else {
            Err("SurfaceFrontIRIrradiance already in Surface".into())
        }
    }
    fn add_back_ir_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_ir_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::SurfaceBackIRIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_back_ir_irradiance_index(i)?;
            Ok(())
        } else {
            Err("SurfaceBackIRIrradiance already in Surface".into())
        }
    }

    fn add_node_temperature_states(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
        n_nodes: usize,
    ) -> Result<(), String> {
        if self.first_node_temperature_index().is_none() {
            // let n_nodes = d.segments.len();
            let first_node = state.len();
            for node_index in 0..n_nodes {
                state.push(
                    SimulationStateElement::SurfaceNodeTemperature(ref_surface_index, node_index),
                    22.0,
                )?;
            }
            let last_node = state.len();
            self.set_first_node_temperature_index(first_node)?;
            self.set_last_node_temperature_index(last_node - 1)?;
            Ok(())
        } else {
            Err("Surface already has nodes attached".into())
        }
    }
}

impl SurfaceTrait for Fenestration {
    
    fn set_front_convective_heat_flow(&self, state: &mut SimulationState, v: Float)->Result<(),String>{
        self.set_front_convective_heat_flow(state, v)
    }

    fn set_back_convective_heat_flow(&self, state: &mut SimulationState, v: Float)->Result<(),String>{
        self.set_back_convective_heat_flow(state, v)
    }

    fn front_infrared_irradiance(&self, state: &SimulationState) -> Float {
        self.front_ir_irradiance(state).unwrap()
    }
    fn back_infrared_irradiance(&self, state: &SimulationState) -> Float {
        self.back_ir_irradiance(state).unwrap()
    }
    fn front_solar_irradiance(&self, state: &SimulationState) -> Float {
        self.front_incident_solar_irradiance(state).unwrap()
    }
    fn back_solar_irradiance(&self, state: &SimulationState) -> Float {
        self.back_incident_solar_irradiance(state).unwrap()
    }
    fn set_front_convection_coefficient(
        &self,
        state: &mut SimulationState,
        v: Float,
    ) -> Result<(), String> {
        self.set_front_convection_coefficient(state, v)
    }

    fn set_back_convection_coefficient(
        &self,
        state: &mut SimulationState,
        v: Float,
    ) -> Result<(), String> {
        self.set_back_convection_coefficient(state, v)
    }

    fn front_convection_coefficient(&self, _state: &SimulationState) -> Option<Float> {
        self.front_convection_coefficient(_state)
    }
    fn back_convection_coefficient(&self, state: &SimulationState) -> Option<Float> {
        self.back_convection_coefficient(state)
    }

    fn first_node_temperature_index(&self) -> usize {
        self.first_node_temperature_index()
            .expect("Could not get first node index in surface")
    }
    fn last_node_temperature_index(&self) -> usize {
        self.last_node_temperature_index()
            .expect("Could not get last node index in surface")
    }

    fn add_front_convection_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_convection_coefficient_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationFrontConvectionCoefficient(ref_surface_index),
                1.739658084820765,
            )?;
            self.set_front_convection_coefficient_index(i)?;
            Ok(())
        } else {
            Err("FenestrationFrontConvectionCoefficient already in Fenestration".into())
        }
    }

    fn add_back_convection_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_convection_coefficient_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationBackConvectionCoefficient(ref_surface_index),
                1.739658084820765,
            )?;
            self.set_back_convection_coefficient_index(i)?;
            Ok(())
        } else {
            Err("FenestrationBackConvectionCoefficient already in Fenestration".into())
        }
    }

    fn add_front_convective_heatflow_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_convective_heat_flow_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationFrontConvectiveHeatFlow(ref_surface_index),
                0.0,
            )?;
            self.set_front_convective_heat_flow_index(i)?;
            Ok(())
        } else {
            Err("FenestrationFrontConvectiveHeatFlow already in Fenestration".into())
        }
    }
    fn add_back_convective_heatflow_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_convective_heat_flow_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationBackConvectiveHeatFlow(ref_surface_index),
                0.0,
            )?;
            self.set_back_convective_heat_flow_index(i)?;
            Ok(())
        } else {
            Err("FenestrationBackConvectiveHeatFlow already in Fenestration".into())
        }
    }

    fn add_front_solar_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_incident_solar_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationFrontSolarIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_front_incident_solar_irradiance_index(i)?;
        }
        Ok(())
    }
    fn add_back_solar_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_incident_solar_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationBackSolarIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_back_incident_solar_irradiance_index(i)?;
        }
        Ok(())
    }

    fn add_front_ir_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.front_ir_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationFrontIRIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_front_ir_irradiance_index(i)?;
        }
        Ok(())
    }
    fn add_back_ir_irradiance_state(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
    ) -> Result<(), String> {
        if self.back_ir_irradiance_index().is_none() {
            let i = state.push(
                SimulationStateElement::FenestrationBackIRIrradiance(ref_surface_index),
                0.0,
            )?;
            self.set_back_ir_irradiance_index(i)?;
        }
        Ok(())
    }

    fn add_node_temperature_states(
        &self,
        state: &mut SimulationStateHeader,
        ref_surface_index: usize,
        n_nodes: usize,
    ) -> Result<(), String> {
        if self.first_node_temperature_index().is_none() {
            let first_node = state.len();
            for node_index in 0..n_nodes {
                state.push(
                    SimulationStateElement::FenestrationNodeTemperature(
                        ref_surface_index,
                        node_index,
                    ),
                    22.0,
                )?;
            }
            let last_node = state.len();
            self.set_first_node_temperature_index(first_node)?;
            self.set_last_node_temperature_index(last_node - 1)?;
            Ok(())
        } else {
            Err("Fenestration has nodes assigned already".into())
        }
    }
}
