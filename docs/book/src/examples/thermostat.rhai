// A thermostat with a cooler setpoint at night, for the model in
// bedroom.spl. Run it with:
//
//     simple -i bedroom.spl -w wellington.epw -n 60 -c thermostat.rhai -o results.csv

// ANCHOR: objects
// Get the objects we want to read and operate
let bedroom = space("Bedroom");
let heater = hvac("Bedroom Heater");
// ANCHOR_END: objects

// ANCHOR: setpoint
// A cooler setpoint at night (hour is the end of the timestep)
let is_night = hour > 22.0 || hour <= 7.0;
let setpoint = if is_night { 16.0 } else { 20.0 };
// ANCHOR_END: setpoint

// ANCHOR: control
// Turn the heater on and off, with a 1C dead band (in between, we
// leave it as it was)
let temperature = bedroom.dry_bulb_temperature;
if temperature < setpoint - 0.5 {
    heater.power_consumption = 1500.0;
} else if temperature > setpoint + 0.5 {
    heater.power_consumption = 0.0;
}
// ANCHOR_END: control

// ANCHOR: memory
// Add up the energy, in kWh (with -n 60, each timestep lasts 1/60 hours)
memory.kwh = (memory.kwh ?? 0.0) + heater.power_consumption / 1000.0 / 60.0;

// ... and tell us at the end of the year
if month == 12 && day == 31 && hour == 24.0 {
    print(`Heating energy: ${memory.kwh.round()} kWh`);
}
// ANCHOR_END: memory
