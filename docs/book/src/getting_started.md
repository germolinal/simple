# Getting Started with `SIMPLE`

`SIMPLE` requires occasion interactions with the terminal and code, so we have made this guide very simple for beginners. Throughout, you will learn how the tool works, how to prepare input files and analyze outputs.

## Running a `SIMPLE` Simulation

`SIMPLE` is a command-line based tool, therefore, it doesn't have a user interface (UI). Here’s a short crash course on how the terminal works:

The terminal is an interface that allows you to access the command line. The command line is a text-based interface within the operating system to control the software.

To run `SIMPLE`, you will use the terminal to navigate to the folder where your `SIMPLE` executable file is located. This can be done by executing a sequence of `cd` (change directory) commands.

Once you are in the correct directory, you can call `SIMPLE` and start a simulation. Here's an example:

```sh
simple -i model.spl -w weather_file.epw -n 1 -o results.csv
```

In this command:
- `-i` stands for input, followed by the name of your model file (in this case, `model.spl`).
- `-w` is used to specify the weather file (here, `weather_file.epw`).
- `-n 1` defines the timestep, which is used to report variables and control the building.
- `-o` is used to define the output file (`results.csv` in this case).

## The Building Model

`SIMPLE` uses a text file as its primary input. This file describes a model, which can contain multiple buildings, although it is most common to simulate just one. (Describing the building in single-building simulations might hel[] estimate some things, like air leakage, based on the number of stories.)

A `SIMPLE` model contains—among others—the following elements (a more accurate and detailed list is available in the Input/Output reference)

* **Spaces**: Basically, volumes of air that will have a single temperature. Each of them becomes a single Thermal Zone.
* **Buildings**: Can contain Spaces
* **Surfaces**: Physical elements—of a certain `Material`—that receive sun, transmit heat, and separate spaces.
* **Substances**: A substance with a set of physical properties (e.g., density)
* **Materials**: A layer of a certain thickness made up of a certain `Substance`
* **Constructions**: A set of `Materials`
* **Hvacs**: Heaters, Air Conditioners and so on.
* **Luminaires**: Elements made for lighting. They produce light and also heat.
* **Objects**: Mostly decorative at the moment (i.e., they do not interfere in simulation results), this is furniture, toiletes and so on.
* **Site Details**: Describes the site and environment in which the model is located.
* **Outputs**: Describe the outputs we want to be reported.


## Describing a model

A `SIMPLE` model can be written in a `.spl` or `.json` file. The `.spl` format is meant to be more human-readable. (`.json` files are meant to be read and written by machines, mostly).

A `.spl` file starts by identifying the object type (e.g., `SiteDetails` or `Substance`) followed by a set of corresponding properties of the object being described.

Comments can be added with `//` for single lines and `/*... */` for multi lines.

Here’s a sample basic model written in `.spl` syntax:


```rs
// This is a single line comment
/*
Comments can have multiple lines
So you can explain thigs better:

1. By adding lists
2. And longer comments.

*/

// This describes the environment of the model
SiteDetails {
   altitude : 0.0, // it is at sea level
   terrain : "City", // in a city
   latitude : -41.0, // at this latitude
   longitude : 174.0,  // at this loingitude
   standard_meridian : 180.0,  // this defines the timezone, basically
}

// This describes a building
Building {
    name: "Art Deco Building",
    n_storeys: 2, // it has 2 storeys
    shelter_class: "Urban",  // It is shelter as you would expect in a city
}
```

## Asking for output

You can request specific outputs to be included in the results. This is done with an `Output` keyword followed by the desired output and an identifier. Here's an example:

```rs
Output {
    SpaceDryBulbTemperature : "Kids Bedroom" // Report the Dry Bulb Temperature of the Kids Bedroom.
}
```

Check out the [Output](./auto-output.md) section for a full list of possible output options.
