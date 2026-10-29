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

Then, you can read and write their properties:

| Object | Properties |
|--------|------------|
| `space` | `dry_bulb_temperature`, `infiltration_volume`, `infiltration_temperature`, `ventilation_volume`, `ventilation_temperature`, `brightness`, `loudness` |
| `surface` | `front_temperature`, `back_temperature`, `front_convection_coefficient`, `back_convection_coefficient`, `front_convective_heat_flow`, `back_convective_heat_flow`, `front_incident_solar_irradiance`, `back_incident_solar_irradiance`, `front_ir_irradiance`, `back_ir_irradiance` |
| `fenestration` | The same as `surface`, plus `open_fraction` |
| `hvac` | `power_consumption` |
| `luminaire` | `power_consumption` |

Each object's page in the Input/Output reference has an _API_ section with these
too (e.g., [Space](./auto-space.md#api)). Some properties only exist when the physics
calculate them (e.g., the `infiltration_volume` of a `Space` with no `infiltration`),
and reading them fails with an error.

The units are the usual ones: °C for temperatures, W for power, W/m² for irradiance,
and so on.

> **Operating vs. cheating:** `power_consumption` and `open_fraction` are
> _operational_ properties: they are what people (or control systems) actually
> change in a building. The others are _physical_ properties, calculated by `SIMPLE`.
> You can read them all—that's how controls know what is going on. But please,
> only write the operational ones: setting the temperature of a space directly
> is not something people can do in real life, so it makes the simulation
> meaningless. (`SIMPLE` does not stop you yet. See [research mode](#research-mode).)

#### How heaters and coolers use `power_consumption`

The physics deliver whatever `power_consumption` you set to the air of the
`target_space` (they all have an efficiency of 1). Positive values heat, and
negative values cool. In other words, you can do proportional control, not just
on/off:

```rs
// 100W per degree below 20C, between 0 and 1500W
let power = 100.0 * (20.0 - bedroom.dry_bulb_temperature);
heater.power_consumption = power.max(0.0).min(1500.0);
```

> **Careful:** The physics do not check the `max_heating_power` (or
> `max_cooling_power`) of the device; staying within limits is the control's job.

### Time

The script knows the date of the timestep it is controlling:

| Variable | Meaning |
|----------|---------|
| `month` | From 1 to 12 |
| `day` | Day of the month, from 1 |
| `hour` | The hour at the **end** of the timestep, from 0 to 24 (e.g., `7.5` is 7:30am). Midnight is `24.0` of the day that ends. |

For example, `hour <= 7.0` is true for every timestep that ends at or before 7am,
i.e., the ones between midnight and 7am.

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

`simple` has a `-r` (`--research-mode`) option. The idea is to enable functions
in the control API that are useful for research but not realistic—e.g., things
that a real control system could not do. There are none yet, so it currently has
no effect.
