use air::Float;
use geometry::{Loop3D, Point3D, Polygon3D};
use model::{substance, Construction, Material, SolarOptions,  Surface};
use simple::{run_simulation::*, Model};

fn main() -> Result<(), String> {
    // cargo instruments --features parallel --template 'CPU Profiler' --release --example scale

    // time cargo run  --release --example scale


    const N:usize=600;
    let mut options = SimOptions::default();
    options.output = Some("./tests/cold_apartment/check.csv".into());
    options.weather_file = "./tests/wellington.epw".into();
    options.n = 4;

    let mut simple_model = Model::default();

    let mut sub = substance::Normal::new("asd");
    sub.set_density(2400.)
        .set_thermal_conductivity(1.63)
        .set_specific_heat_capacity(900.0);
    let sub = sub.wrap();
    let s = simple_model.add_substance(sub);
    let sub_name = s.name();

    let mat = Material::new("materaial", sub_name, 0.1);
    let mat = simple_model.add_material(mat);
    let mut solar_options = SolarOptions::new();
    solar_options
        .set_optical_data_path(format!("./remove_{}.json",N))
        .set_solar_ambient_divitions(1)
        .set_n_solar_irradiance_points(1);

    simple_model.solar_options = Some(solar_options);

    let mut con = Construction::new("the construction");
    con.materials.push(mat.name().clone());
    let con = simple_model.add_construction(con);
    let con_name = con.name();

    for i in 0..N {
        let mut the_l = Loop3D::new();

        the_l
            .push(Point3D::new(0.0, 0.0, i as Float * 0.1))
            .unwrap();
        the_l
            .push(Point3D::new(1.0, 0.0, i as Float * 0.1))
            .unwrap();
        the_l
            .push(Point3D::new(0.0, 1.0, i as Float * 0.1))
            .unwrap();

        the_l.close().unwrap();

        let poly = Polygon3D::new(the_l).unwrap();
        let s = Surface::new(
            format!("s_{i}"),
            poly,
            con_name.clone(),
            model::Boundary::Outdoor,
            model::Boundary::Outdoor,
        );

        simple_model.add_surface(s);
    }

    let mut state_header = simple_model.take_state().unwrap();

    let controller = simple::void_control::VoidControl {};

    let res = &options.output.clone().ok_or("No output")?;
    let out = std::fs::File::create(res).map_err(|e| e.to_string())?;

    run(
        &simple_model,
        &mut state_header,        
        &options,
        out,
        controller,
    )?;

    Ok(())
}
