# Getting started with `SIMPLE`

In this chapter we will build a model of a single bedroom, simulate it for a whole
year, and look at the results. Along the way you will learn the main ideas behind
`SIMPLE`'s models. The [next chapter](./controlling.md) will then teach you how to
operate the building (e.g., turn on a heater), which—you'll see—is a big deal in `SIMPLE`.

`SIMPLE` is used from the terminal, and its models are text files. So, you will
need a text editor and a little bit of courage. I promise it is not that bad.

## Installing `SIMPLE`

The easiest way is to download the pre-built binaries from the
[latest release](https://github.com/germolinal/simple/releases/latest), for macOS
(Apple silicon), Linux (x86-64) or Windows (x86-64). Each archive contains a few
command-line tools; the one we care about now is `simple`. Put it somewhere in your
`PATH` (or just in the folder where you will be working) and check that it works:

```sh
simple --help
```

> **On macOS**, the binaries are not signed. If the system refuses to open them,
> run `xattr -d com.apple.quarantine simple`.

If you'd rather build it yourself (or you are on another platform), check the
[README](https://github.com/germolinal/simple#build-from-source).

You will also need a weather file, in [EPW format](https://climate.onebuilding.org).
For this chapter I'll use [the one for Wellington, New Zealand](https://github.com/germolinal/simple/raw/main/tests/wellington.epw),
which is where I live and the reason why I care about cold homes.

## The model in one paragraph

A `SIMPLE` model describes **what the building is**: its spaces, the surfaces that
enclose them, what those surfaces are made of, the devices in it (heaters,
luminaires), and so on. It does not describe **what happens to it**—temperatures,
heat flows, whether the heater is on—which is the _simulation state_ and is
calculated by `SIMPLE`. (Separating these two is what allows `SIMPLE` to
[travel in time](./what_is_simple.md#3-it-was-designed-to-travel-in-time).)

Models are written in `.spl` files, which look like this:

```rs
// Comments start with two slashes...
/* ...or go between these, if they span
   multiple lines */
Space {
    name: "Bedroom",
    volume: 32.4,
}
```

That is: the kind of object (`Space`), followed by its properties between curly
braces. The order of the objects in the file does not matter, and objects refer
to each other by their `name`. The [Input/Output reference](./auto-space.md)
lists every object and property.

> **Note:** `SIMPLE` is strict. A property it does not know (e.g., a typo like
> `volumen`), a missing required property, or a name that refers to nothing
> (e.g., a `Material` made of a `Substance` that does not exist) are all errors.
> This is annoying for about five minutes, and then it saves you hours.

## Our bedroom

Our bedroom is 4m wide (`x`), 3m deep (`y`) and 2.7m tall (`z`). It has a single
window facing North—which, in Wellington, is where the sun is.

You can download [the whole model](./examples/bedroom.spl), but let's go through
it piece by piece.

### What things are made of

`SIMPLE` describes materials in three steps:

1. A `Substance` is _what things are made of_: concrete, glass, wool... It has
   physical properties like density and thermal conductivity.
2. A `Material` is a layer of a `Substance` with a certain thickness (e.g., 20cm of concrete).
3. A `Construction` is a stack of `Material`s, from the front to the back (we'll get to what front and back mean soon).

```rs
{{#include examples/bedroom.spl:materials}}
```

> **Why three steps?** Because it avoids repeating ourselves. The same `Substance`
> can be used in layers of different thicknesses, and the same `Material` in many
> `Construction`s.

### The space

A `Space` is a volume of air with a single temperature. Each `Space` becomes a
thermal zone in the simulation.

```rs
{{#include examples/bedroom.spl:space}}
```

### The walls and the window

`Surface`s are walls, floors, roofs and so on. Each of them has a `Construction`,
a polygon (its `vertices`, as a flat list of `x, y, z` coordinates, in metres),
and two `Boundary`s: what is in front of it and what is behind it.

```rs
{{#include examples/bedroom.spl:north_wall}}
```

A few things are worth explaining here:

* **The front of a surface is the side its normal points to**, and the normal follows
  the right-hand rule: if the fingers of your right hand follow the vertices, your
  thumb points to the front. In this model, all the surfaces have their front
  facing out and their back facing the bedroom.
* **Boundaries** say what is on each side. If none is given, the surface leads
  outdoors (i.e., it gets sun, wind and sky). A `Space` boundary connects the
  surface to a space. Check the [Boundary reference](./auto-boundary.md) for the
  others.
* **Windows and doors are `Fenestration`s.** If you give them a `parent_surface`,
  `SIMPLE` cuts the hole in that surface for you. They follow the same rules
  as surfaces (so the window also faces North).

The other three walls are the same, just in other places:

```rs
{{#include examples/bedroom.spl:other_walls}}
```

### The floor and the ceiling

Our bedroom is in an apartment, so we will not simulate what is under it or above
it. Instead, we will say that these are at a fixed temperature, using an
`AmbientTemperature` boundary.

```rs
{{#include examples/bedroom.spl:floor_ceiling}}
```

> **What about the ground?** There is a `Ground` boundary, but the thermal model
> does not support it yet ([#152](https://github.com/germolinal/simple/issues/152)).
> For now, use an `AmbientTemperature` with your best estimate of the ground's temperature.

### A heater

```rs
{{#include examples/bedroom.spl:heater}}
```

This is an electric heater of 1500W. Hold that thought, because (spoiler alert)
it will not heat anything in this chapter.

### What we want to know

By default, `SIMPLE` reports _everything_ it calculates: every temperature, heat
flow, solar irradiance... which is a lot, even for a single bedroom. With `Output`s,
we can ask for specific values:

```rs
{{#include examples/bedroom.spl:outputs}}
```

The [Output reference](./auto-output.md) lists every value you can ask for.

## Running the simulation

First, let's check that the model is OK, without simulating anything:

```sh
simple -i bedroom.spl --check
```

If nothing is printed, all is good. Otherwise, you will get an error telling you
what is wrong. Now, the real thing:

```sh
simple -i bedroom.spl -w wellington.epw -o results.csv
```

| Option | Meaning |
|--------|---------|
| `-i` | The input model (`.spl` or `.json`) |
| `-w` | The weather file (`.epw`) |
| `-o` | Where to write the results. If not given, they are printed on the terminal |
| `-n` | The number of timesteps per hour (from 1 to 60; 1 by default) |
| `-c` | How to control the building (see the [next chapter](./controlling.md)) |
| `-r` | Research mode (see [Research mode](./controlling.md#research-mode)) |
| `-s` | The seed for random numbers in control scripts (see [Randomness](./controlling.md#randomness)) |

The simulation runs through the whole weather file, telling you what month it is
in. For our bedroom, this takes less than a second.

## Reading the results

`results.csv` is a normal CSV file you can open with Excel, Python, R, or whatever
you like. It starts like this:

```text
Date,{"SpaceDryBulbTemperature":"Bedroom"},{"HeatingCoolingPowerConsumption":"Bedroom Heater"}
01/01 - 1:00,21.298,0.000
01/01 - 2:00,20.992,0.000
01/01 - 3:00,20.551,0.000
```

Each column (after the date) is one of the `Output`s, named the same way you asked
for it. Each row is a timestep, and **the date is the end of that timestep**. For
example, `01/01 - 1:00` is the hour between midnight and 1am. This is how EPW files
(and EnergyPlus) do it too, so a year simulated at one timestep per hour has 8760
rows, from `01/01 - 1:00` to `12/31 - 24:00`.

So, how did our bedroom do? Not great. In winter it gets down to 7.6°C. And the
heater? It never consumed a single Watt. That is not a bug: **in `SIMPLE`, a
heater that nobody turns on does not heat**. The [next chapter](./controlling.md)
is all about this.

## Some more tricks

### Translating between `.spl` and `.json`

`.spl` files are meant for humans. JSON files are better for machines (e.g., if
you generate models with a script). `SIMPLE` reads both, and it can translate
between them:

```sh
simple -i bedroom.spl -t bedroom.json
```

### Saving the solar calculations

Before simulating, `SIMPLE` calculates how the sun and the sky reach each surface,
which can take a while in bigger models. As long as the geometry and materials
don't change, you can store these calculations and reuse them, with
[`SolarOptions`](./auto-solaroptions.md):

```rs
SolarOptions {
    optical_data_path: "./bedroom_optical_data.json",
}
```

If the file does not exist, `SIMPLE` calculates the data and saves it there. If
it exists, it is just read. (So **delete it if you change the geometry or the materials!**)

`SolarOptions` also lets you trade accuracy for speed, and the other way around.

### Where to go from here

* Learn how to operate the building in [Controlling the building](./controlling.md).
* Browse the [Input/Output reference](./auto-boundary.md) to see what else you
  can put in a model.
* Look at the models used for validating `SIMPLE`, in the
  [`tests` folder](https://github.com/germolinal/simple/tree/main/tests) (e.g.,
  the [cold apartment](https://github.com/germolinal/simple/blob/main/tests/cold_apartment/cold.spl)).
