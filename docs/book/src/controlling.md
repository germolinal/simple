# Controlling the building

At the end of the [Getting started](./getting_started.md) chapter, our bedroom had
a 1500W heater that never consumed a single Watt. This chapter explains why, and
how to turn it on.

## Nothing happens unless someone does it

This is one of `SIMPLE`'s principles: **the model describes what the building
_is_, and the physics describe what happens to it. But nobody operates the building
unless you say so.** A heater that is not turned on does not heat; a window that
is not opened stays closed; a light that is not switched on stays off.

This might seem inconvenient—most tools just heat whenever the temperature is below
the setpoint—but it is on purpose. Remember that `SIMPLE` was born to
[simulate buildings for real people](./what_is_simple.md), and real people do not
always turn the heater on when it is cold. Maybe they can't afford it, maybe they
are not home, maybe they would rather put on a jumper. Whatever the reason, _how_
a building is operated is a big part of how it performs. So, `SIMPLE` treats it
as an explicit part of the simulation: the **control**.

> **Note:** The properties of devices that sound like control—e.g., the
> `heating_setpoint` of an `ElectricHeater`—are not used by the physics at all.
> They are information that controls can use (or ignore).

The control runs **once at the beginning of every timestep**: it looks at the
current state of the building (e.g., temperatures), decides what to do (e.g., turn
on the heater), and then the physics calculate what happens during that timestep.

You can choose between three controls, with the `-c` option:

| Control | How | What it does |
|---------|-----|--------------|
| None | (no `-c`) | Nothing. The building is never operated. |
| People | `-c people` | Turns heaters and coolers on and off, based on their setpoints |
| A script | `-c my_script.rhai` | Whatever you write |

## The `people` control

This built-in control behaves like a simple thermostat. Every timestep, and for
every `HVAC` in the model that has the information it needs:

* An `ElectricHeater` with a `heating_setpoint` and a `max_heating_power` is
  turned on—at full power—if its `target_space` is below the setpoint. Otherwise,
  it is turned off.
* An `IdealHeaterCooler` heats (at `max_heating_power`) when its `target_space`
  is below the `heating_setpoint`, and cools (at `max_cooling_power`) when it is
  above the `cooling_setpoint`.

Let's try it with our bedroom:

```sh
simple -i bedroom.spl -w wellington.epw -c people -o results.csv
```

Now the heater works... maybe a bit too well. During winter days, the bedroom
temperature goes anywhere between 14.2°C and 30.9°C! The reason is that the
control only runs once per timestep, and our timestep is one hour. If the
bedroom is at 18.9°C at the beginning of the hour, the heater runs at full power
_for the whole hour_, which is a lot of energy for such a small room.

The solution is to use shorter timesteps. With 60 timesteps per hour (`-n 60`), the
temperatures stay between 18.1°C and 21.1°C, which is much more like what a thermostat does.

> **Choosing a timestep:** The air in a room holds very little heat (our bedroom's
> air warms up by about 1°C with just 39kJ, or 1500W during 26 seconds). So, if
> you turn devices on and off, use short timesteps. More on this
> [below](#timesteps-matter).

## Writing your own control

The `people` control is fine for some cases, but the whole point is to let you
describe how _your_ people operate _your_ building. For that, `SIMPLE` reads
control scripts written in [Rhai](https://rhai.rs/book/), a small scripting
language that looks a bit like JavaScript or Rust.

Here is a thermostat with a cooler setpoint at night (you can download the
[whole script](./examples/thermostat.rhai)). First, we get the objects we want
to read and operate:

```rs
{{#include examples/thermostat.rhai:objects}}
```

Then we decide the setpoint, depending on the time of the day:

```rs
{{#include examples/thermostat.rhai:setpoint}}
```

And finally, we turn the heater on or off:

```rs
{{#include examples/thermostat.rhai:control}}
```

Run it with:

```sh
simple -i bedroom.spl -w wellington.epw -n 60 -c thermostat.rhai -o results.csv
```

The whole script runs once at the beginning of every timestep. So, the
dead band works because the heater stays as it was (on or off) unless we change it.

### Accessing the building

You can get the objects in the model by name (or by their index, starting from 0,
in the order they appear in the model):

```rs
let bedroom = space("Bedroom");
let first_space = space(0);

let wall = surface("North Wall");
let window = fenestration("North Window");
let heater = hvac("Bedroom Heater");
let light = luminaire("Bedroom light");
```

If the object does not exist, the script fails with an error telling you so.

Then, you can read their properties, and set those that people can change in
real life:

| Object | Can be set | Can only be read |
|--------|------------|------------------|
| `space` | | `dry_bulb_temperature`, `infiltration_volume`, `infiltration_temperature`, `ventilation_volume`, `ventilation_temperature`, `brightness`, `loudness` |
| `surface` | | `front_temperature`, `back_temperature`, `front_convection_coefficient`, `back_convection_coefficient`, `front_convective_heat_flow`, `back_convective_heat_flow`, `front_incident_solar_irradiance`, `back_incident_solar_irradiance`, `front_ir_irradiance`, `back_ir_irradiance` |
| `fenestration` | `open_fraction` | The same as `surface` |
| `hvac` | `power_consumption` | |
| `luminaire` | `power_consumption` | |

Each object's page in the Input/Output reference has an _API_ section with these
too (e.g., [Space](./auto-space.md#api)). Some properties only exist when the physics
calculate them (e.g., the `infiltration_volume` of a `Space` with no `infiltration`),
and reading them fails with an error.

The units are the usual ones: °C for temperatures, W for power, W/m² for irradiance,
and so on.

> **Rhai tip:** Properties are set on variables, so
> `hvac("Bedroom Heater").power_consumption = 1500.0;` fails with _"Expression cannot
> be assigned to"_. Store the object in a variable first, like we did above.

#### How devices use `power_consumption`

The `power_consumption` of an `HVAC` is delivered to the air of its `target_space`
(they all have an efficiency of 1). Positive values heat, and negative values cool.
Similarly, all the `power_consumption` of a `Luminaire` becomes heat in its
`target_space`. You can use any value in between, so you can do proportional
control (or dim lights), not just on/off:

```rs
// 100W per degree below 20C
heater.power_consumption = 100.0 * (20.0 - bedroom.dry_bulb_temperature);
```

But devices can only do what they can do. Whatever the control asks for, the power
is limited to:

| Device | From | To |
|--------|------|----|
| `ElectricHeater` | 0 (it cannot cool) | `max_heating_power` |
| `IdealHeaterCooler` | minus `max_cooling_power` (cooling) | `max_heating_power` (heating) |
| `Luminaire` | 0 | `max_power` |

The results report the power that was actually used. And, as asking a device for
something it cannot do is probably a mistake in the control, `SIMPLE` warns you
(on the terminal, the first time it happens for each device):

```text
Warning: [in Thermal model] 01/01 - 1:00: HVAC 'Bedroom Heater' was asked to use 99999 W, but it can only use between 0 W and 1500 W... so it used 1500 W. (This is reported only once per device.)
```

If a device has no limit
(e.g., no `max_heating_power`), the limit is 10kW, which is handy for sizing:
ask for whatever keeps the room comfortable, and the results tell you how much
power that took. (10kW is a lot for a room, but if your space needs more, give the
device a `max_heating_power` that is large enough.)

> **Careful:** Limits keep devices realistic, but not controls. A script that keeps
> a 10kW heater on all year will cook a small sealed room to temperatures where the
> physics stop making sense (and, eventually, stop working).

### Time

The script knows the date of the timestep it is controlling:

| Variable | Meaning |
|----------|---------|
| `month` | From 1 to 12 |
| `day` | Day of the month, from 1 |
| `hour` | The hour at the **end** of the timestep, from 0 to 24 (e.g., `7.5` is 7:30am). Midnight is `24.0` of the day that ends. |
| `day_of_week` | `"monday"`, `"tuesday"`, ..., `"sunday"` |

For example, `hour <= 7.0` is true for every timestep that ends at or before 7am,
i.e., the ones between midnight and 7am. And this is how you would tell weekends apart:

```rs
let is_weekend = day_of_week == "saturday" || day_of_week == "sunday";
```

`SIMPLE` knows the day of the week because EPW files say on which day they start.
If the weather does not say (e.g., some synthetic weathers), `day_of_week` is `()`
(i.e., nothing). Note that EPW files often combine months from different years, so
the days of the week are not necessarily those of any real calendar.

### Randomness

People are not that predictable, so control scripts can use `random()`, which
returns a random number between 0 and 1. For example, people who open the window
40% of the times they feel hot:

```rs
let window = fenestration("North Window");
if bedroom.dry_bulb_temperature > 25.0 && random() < 0.4 {
    window.open_fraction = 1.0;
}
```

Keep in mind that the script runs every timestep, so "40% of the time" depends on
the timestep: with `-n 60`, that is a 40% chance _every minute_.

By default, each simulation gets different random numbers, so two runs of the same
model give (slightly) different results. To repeat a simulation exactly (e.g., to
compare two designs with the same "people", or for a paper), give it a seed:

```sh
simple -i bedroom.spl -w wellington.epw -c my_script.rhai -s 42
```

The same seed always gives the same random numbers.

### Remembering things

Every variable in the script is forgotten when the script ends, except those
in `memory`. This is a map whose contents persist from one timestep to the next,
and which starts empty. For example, this adds up the heating energy:

```rs
{{#include examples/thermostat.rhai:memory}}
```

(`??` gives the value on the right when the one on the left does not exist, which
is the case the first time the script runs.)

`memory` must stay a map, so do `memory.something = value` (or
`memory["something"] = value`), not `memory = value`.

> **Remember** that properties of the building (e.g., `power_consumption`) persist
> anyway—they are part of the simulation state. `memory` is for things that are
> only in the head of the people in the building (e.g., "I opened the window an
> hour ago").

### Printing

`print()` and `debug()` write to `stderr`, which is the terminal. So, they are
never mixed with the results, even if no `-o` is given:

```rs
print(`It is ${bedroom.dry_bulb_temperature}C in the bedroom`);
```

Careful, though: the script runs every timestep, so printing without an `if`
prints a lot (525,600 lines in a year at `-n 60`).

## Timesteps matter

Here is our thermostat (with a 1°C dead band) at different timesteps:

| Timesteps per hour (`-n`) | Bedroom temperature in winter days | Heating energy in a year |
|---|---|---|
| 4 | 14.7°C – 29.6°C | 2094 kWh |
| 12 | 16.3°C – 27.1°C | 1611 kWh |
| 30 | 18.1°C – 23.4°C | 1362 kWh |
| 60 | 18.4°C – 21.7°C | 1209 kWh |

The energy goes down because, with long timesteps, the heater keeps running long after
the bedroom is warm. So, **if your control turns things on and off, use short
timesteps**, and check that the results do not change much if you make them
even shorter.

> **Note:** If you add up energy in a script (like we did above), remember that
> each timestep lasts `1/n` hours.

## Research mode

`SIMPLE` simulates buildings that could exist, operated by people who could exist.
So, by default, **control scripts can only do what people (or control systems) can
do in real life**: turn a heater on or off, open a window, dim a light, and so on.
They can _read_ everything (people can feel the temperature, and thermostats can
measure it), but they cannot _set_ physical properties, like the temperature of a
space, the temperature of a wall, or the solar radiation reaching a window.

But sometimes, unrealistic is exactly what you need. For example, to answer _"how
much energy would it take to keep this room at exactly 21°C?"_ or to test a model
with a known temperature. For that, there is **research mode**:

```sh
simple -i bedroom.spl -w wellington.epw -c my_script.rhai -r
```

In research mode, control scripts can also set physical properties:

```rs
let bedroom = space("Bedroom");
bedroom.dry_bulb_temperature = 21.0; // fails, unless in research mode
```

The physics take it from there (i.e., the value you set is the starting point
of the timestep, not a value that stays fixed).

> **Rule of thumb:** If you are setting something with research mode, make sure
> you would be happy to explain _why_ in a paper. Results from research-mode
> simulations describe a building that cannot exist.
