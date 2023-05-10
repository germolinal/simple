use simple::{run_simulation::*, Model};
use validate::{valid, ScatterValidator, Validate, Validator};

#[test]
fn apartment_sim() {
    // cargo test --release --package simple --test cold_apartment -- apartment_sim --exact --nocapture 
    let p = "./docs/validation";
    if !std::path::Path::new(&p).exists() {
        std::fs::create_dir(p).unwrap();
    }
    let target_file = format!("{}/cold_wellington_apartment.html", p);
    let mut validations = Validator::new(
        "Simulation of a single room",
        &target_file,
    );

    let options = SimOptions {            
        input_file: "./tests/cold_apartment/cold.spl".into(),
        weather_file: "./tests/wellington.epw".into(),
        output: Some("./tests/cold_apartment/check.csv".into()),
        control_file: None,
        research_mode: false,
        n: 4,
    };

    // Create model
    let (simple_model, mut state_header) =
        Model::from_file(options.input_file.to_string()).unwrap();

    let controller = simple::void_control::VoidControl {};
    // let controller = simple::OccupantBehaviour::new(&simple_model).unwrap();

    let res = &options.output.clone().unwrap();
    let out = std::fs::File::create(res).unwrap();
    run(
        &simple_model,
        &mut state_header,
        // state,
        &options,
        out,
        controller,
    )
    .unwrap();


    fn process_space(i: usize)->Box<dyn Validate>{
        // Load produced data
        let found = validate::from_csv::<simple::Float>("./tests/cold_apartment/check.csv", &[1,2,3,4,5,6,7,8]);
        let expected = validate::from_csv::<simple::Float>("tests/cold_apartment/eplusout.csv", &[1,2,3,4,5,6,7,8]);
                
        Box::new(ScatterValidator {
            chart_title: Some("Dry Bulb Temperature - SIMPLE vs EnergyPlus"),
            units: Some("C"),
            expected_legend: Some("EnergyPlus-calculated temperature"),
            expected: expected[i].iter().skip(20).map(|v| *v).collect(),
            found_legend: Some("SIMPLE-calculated temperature"),
            found: found[i].iter().skip(20).map(|v| *v).collect(),

            allowed_r2: Some(0.93),
            allowed_intersect_delta: Some(0.8),
            allowed_slope_delta: Some(0.12),

            ..Default::default()
        })
        
    }


    
    
    

    #[valid(Kids bedroom in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn kids_bedroom() -> Box<dyn Validate> {        
        process_space(0)                   
    }

    #[valid(Bathrooom in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn bathroom() -> Box<dyn Validate> {        
        process_space(1)                   
    }

    #[valid(Storage in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn storage() -> Box<dyn Validate> {        
        process_space(2)                   
    }
        
    #[valid(Kitchen in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn kitchen() -> Box<dyn Validate> {        
        process_space(3)                   
    }

    #[valid(Laundry in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn laundry() -> Box<dyn Validate> {        
        process_space(4)                   
    }

    #[valid(Livingroom in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn livingroom() -> Box<dyn Validate> {        
        process_space(5)                   
    }
    
    #[valid(Main Bedroom in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn main_bedroom() -> Box<dyn Validate> {        
        process_space(6)                   
    }

    #[valid(Hallway in an apartment in Wellington, New Zealand)]
    /// This simulation runs throughout the whole year at 15-minute timesteps.
    /// 
    /// It includes pretty much everything
    fn hallway() -> Box<dyn Validate> {        
        process_space(7)                   
    }


    validations.push(kids_bedroom());
    validations.push(bathroom());
    validations.push(storage());
    validations.push(kitchen());
    validations.push(laundry());
    validations.push(livingroom());
    validations.push(main_bedroom());
    validations.push(hallway());
    

    validations.validate().unwrap();
}
