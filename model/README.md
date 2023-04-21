# `SIMPLE` model


![build badge](https://github.com/SIMPLE-BuildingSimulation/model/actions/workflows/build.yaml/badge.svg)
![docs badge](https://github.com/SIMPLE-BuildingSimulation/model/actions/workflows/docs.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/model/actions/workflows/tests.yaml/badge.svg)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/model/branch/master/graph/badge.svg?token=RST5L99M3M)](https://codecov.io/gh/SIMPLE-BuildingSimulation/model)
![style badge](https://github.com/SIMPLE-BuildingSimulation/model/actions/workflows/style.yaml/badge.svg)



This repository contains the basic API for creating and 
reading [`SIMPLE` models](http://www.simplesim.tools). This can be used by tools that intend
to help create these models, or by tools that aim to 
use these models for performing simulations (e.g., 
the [Thermal simulation module](https://github.com/SIMPLE-BuildingSimulation/thermal)). 
Find the Rust reference [HERE](https://simple-buildingsimulation.github.io/model/)

## Some features

This library includes a macro for generating automatic [user documentation](https://simple-buildingsimulation.github.io/model/ioreference/book/html/index.html) and a highly 
consistent [RUST API](https://simple-buildingsimulation.github.io/model/rustdoc/doc/model/index.html). 

This means that a structure—and documentation—written as follows

```rs
/// The representation of a physical layer-Material.
/// That is to say, a layer of a certain thickness
/// made of a certain Substance
#[derive(ObjectIO)]
pub struct Material {
    /// The name of the material object
    pub name: String,

    /// A reference to the [`Substance`] of which this
    /// [`Material`] is made of    
    pub substance: Substance,

    /// The thickness of the [`Material`]
    pub thickness: Float,
}
```

Will be automatically transformed into something 
[LIKE THIS](https://simple-buildingsimulation.github.io/model/ioreference/book/html/auto-material.html) 
in the eyes of the users.

Contrary to `Material`, some other structures will include a state. (e.g., an `ElectricHeater` 
can be On, Off or somewhere in between). 

This is represented as follows:


```rs
use serde::{Serialize, Deserialize};

#[derive(Clone, ObjectIO, GroupMemberAPI, Serialize, Deserialize)]
pub struct ElectricHeater {
    /// The name of the system
    pub name: String,

    /// The [`Space`] that this [`ElectricHeater`] heats and/or
    /// cools
    target_space: Option<Rc<Space>>,

    /// Max heating power
    max_heating_power: Option<Float>,

    #[operational("power_consumption")] // this variable is exposed to the API with the alias `power_consumption`
    #[serde(skip)]
    heating_cooling_consumption: StateElementField,
}
```

Now, because of a number of reasons (related to mutability
and to how different modules in `SIMPLE` are communicated, [DETAILS HERE](https://youtu.be/yR7cdchDJYI)), the values of those state variables are stored in a separate structure called 
`SimulationState`. Fortunately, he Derive macro will automatically create a number of methods 
for this structure, such as `heating_cooling_consumption(&self, &SimulationState)->Option<f64>`, 
`heating_acooling_consumption_index(&self)->Option<usize>`, `set_heating_cooling_consumption(&self, &mut SimulationState, v: f64)` 
and `set_heating_cooling_consumption_index(&self, i: usize)`. These methods are crucial for the 
engines that are based on the `SIMPLE` Model.


## `SIMPLE` models from text files

`SIMPLE` models can be creted through the API (written in RUST programming language, check it out [HERE](https://simple-buildingsimulation.github.io/model/rustdoc/doc/model/index.html)), but they can also be written in text files. Check the automatically generated [Input Output reference](https://simple-buildingsimulation.github.io/model/ioreference/book/html/index.html)



