use crate::{Boundary, Float, SimulationState, SimulationStateHeader};
use geometry::{BBox3D, Loop3D, Polygon3D, Vector3D};
use matrix::Matrix;

/// The orientation of a Vector, generally used for surfaces
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Orientation {
    /// The normal points up (more than 45 degrees), therefore
    /// the window might be a skylight.
    Horizontal,
    /// Faces North
    North,
    /// Faces NorthEast
    NorthEast,
    /// Faces East
    East,
    /// Faces SouthEast
    SouthEast,
    /// Faces South
    South,
    /// Faces SouthWest
    SouthWest,
    /// Faces West
    West,
    /// Faces NorthWest
    NorthWest,
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Orientation {
    /// Transforms the orientation into a human-readable string
    pub fn to_str(&self) -> &str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
            Self::NorthEast => "north east",
            Self::NorthWest => "north west",
            Self::SouthEast => "south east",
            Self::SouthWest => "south west",
            Self::Horizontal => "horizontal",
        }
    }
}

/// Gets the orientation of a vector, generally the normal
/// of a surface
pub fn get_orientation(normal: Vector3D) -> Orientation {
    const UP: Vector3D = Vector3D::new(0., 0., 1.);

    // more than 45 degrees of elevation is a horizontal window
    let is_up = (normal * UP).abs() >= (45.0 as Float).to_radians().cos();
    if is_up {
        Orientation::Horizontal
    } else {
        // Project normal onto a horizontal
        let normal = Vector3D::new(normal.x, normal.y, 0.0).get_normalized();

        const COS_22_5: Float = 0.923879532511287; // cos(22.5 degrees)

        let faces_north = normal.y > 0.0;
        let faces_east = normal.x > 0.0;

        let is_north = normal.y > COS_22_5;
        let is_south = normal.y < -COS_22_5;
        let is_east = normal.x > COS_22_5;
        let is_west = normal.x < -COS_22_5;

        let is_north_east = faces_north && faces_east && !is_north && !is_east;
        let is_north_west = faces_north && !faces_east && !is_north && !is_west;
        let is_south_east = !faces_north && faces_east && !is_south && !is_east;
        let is_south_west = !faces_north && !faces_east && !is_south && !is_west;

        if is_north {
            Orientation::North
        } else if is_north_east {
            Orientation::NorthEast
        } else if is_north_west {
            Orientation::NorthWest
        } else if is_south {
            Orientation::South
        } else if is_south_east {
            Orientation::SouthEast
        } else if is_south_west {
            Orientation::SouthWest
        } else if is_west {
            Orientation::West
        } else if is_east {
            Orientation::East
        } else {
            unreachable!()
        }
    }
}

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

    /// Borrows a mutable reference to the [`Polygon3D`] describing
    /// this surface
    fn vertices(&self) -> &Polygon3D;

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

    /// Gets the orientation of the normal
    fn orientation(&self) -> Orientation {
        get_orientation(self.normal())
    }

    /// Retrieves the orientation of the side that faces
    /// outdoors, only if the other side faces inside.
    /// If both sides face outdoors, or if no side face indoors, it returns None.
    fn outside_orientation(&self) -> Option<Orientation> {
        let front_b = self.front_boundary();
        let back_b = self.back_boundary();
        if let (Boundary::Outdoor, Boundary::Space { .. }) = (front_b, back_b) {
            // Space is at the back
            let n = self.normal();
            Some(get_orientation(n))
        } else if let (Boundary::Space { .. }, Boundary::Outdoor) = (front_b, back_b) {
            // Space is at the front.
            let n = self.normal() * -1.0;
            Some(get_orientation(n))
        } else {
            // both sides lead to spaces
            None
        }
    }

    /// Retrieves the orientation of the side that faces
    /// indoors, only if the other side faces outdoors.
    /// If both sides face indoors, or if no side face outdoors, it returns None.
    fn inside_orientation(&self) -> Option<Orientation> {
        let front_b = self.front_boundary();
        let back_b = self.back_boundary();
        if let (Boundary::Outdoor, Boundary::Space { .. }) = (front_b, back_b) {
            // Space is at the back
            let n = self.normal() * -1.0;
            Some(get_orientation(n))
        } else if let (Boundary::Space { .. }, Boundary::Outdoor) = (front_b, back_b) {
            // Space is at the front.
            let n = self.normal();
            Some(get_orientation(n))
        } else {
            // both sides lead to spaces
            None
        }
    }

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

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_get_orientation() -> Result<(), String> {
        fn check(exp: Orientation, alpha_deg: Float, phi_deg: Float) -> Result<(), String> {
            let alpha = alpha_deg.to_radians();
            let phi = phi_deg.to_radians();
            let z = alpha.sin();
            let x = alpha.cos() * phi.sin();
            let y = alpha.cos() * phi.cos();
            let normal = Vector3D::new(x, y, z);

            let found = get_orientation(normal);
            if found == exp {
                Ok(())
            } else {
                Err(format!(
                    "Expecting orientation to be '{}'... found '{}' (alpha={}, phi={})",
                    exp, found, alpha_deg, phi_deg
                ))
            }
        }

        // Anything with ALPHA between 45 and 90 is UP
        let exp = Orientation::Horizontal;
        for i in 0..100 {
            let min_alpha = 45.000001;
            let max_alpha = 90.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = 0.0;
                let max_phi = 90.0;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::North;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = -22.49999;
                let max_phi = 22.499999;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::NorthEast;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = 22.5;
                let max_phi = 22.5 + 22.499999;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::East;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = -22.49999 + 90.0;
                let max_phi = 22.499999 + 90.0;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::SouthEast;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = 22.5 + 90.0;
                let max_phi = min_phi + 22.499999;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::South;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = -22.49999 + 180.0;
                let max_phi = 22.499999 + 180.0;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::SouthWest;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = 22.5 + 180.0;
                let max_phi = min_phi + 22.499999;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::West;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = -22.49999 + 270.0;
                let max_phi = 22.499999 + 270.0;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        let exp = Orientation::NorthWest;
        for i in 0..100 {
            let min_alpha = 0.0;
            let max_alpha = 45.0;
            let delta_alpha = (max_alpha - min_alpha) / 100.0;
            let alpha = min_alpha + (i as Float) * delta_alpha;
            for j in 0..100 {
                let min_phi = 270.0 + 22.5;
                let max_phi = min_phi + 22.499999;
                let delta_phi = (max_phi - min_phi) / 100.0;
                let phi = min_phi + (j as Float) * delta_phi;
                check(exp, alpha, phi)?;
            }
        }

        Ok(())
    }

    #[test]
    fn test_to_str() {
        assert_eq!(Orientation::North.to_str(), "north");
        assert_eq!(Orientation::NorthEast.to_str(), "north east");
        assert_eq!(Orientation::East.to_str(), "east");
        assert_eq!(Orientation::SouthEast.to_str(), "south east");
        assert_eq!(Orientation::South.to_str(), "south");
        assert_eq!(Orientation::SouthWest.to_str(), "south west");
        assert_eq!(Orientation::West.to_str(), "west");
        assert_eq!(Orientation::NorthWest.to_str(), "north west");
        assert_eq!(Orientation::Horizontal.to_str(), "horizontal");
    }
}
