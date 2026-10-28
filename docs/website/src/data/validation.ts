export interface Report {
  /** File in docs/validation, written by `make validate` */
  file: string;
  against: string;
  title: string;
  description: string;
  /** A known limitation, shown as a warning */
  note?: string;
}

export interface ReportGroup {
  title: string;
  reports: Report[];
}

export const groups: ReportGroup[] = [
  {
    title: "Whole buildings",
    reports: [
      {
        file: "cold_wellington_apartment.html",
        against: "vs EnergyPlus",
        title: "Full apartment",
        description: "A year of an eight-zone apartment in Wellington, New Zealand.",
      },
      {
        file: "cold_wellington_box.html",
        against: "vs EnergyPlus",
        title: "Single room",
        description: "A year of a box with a window in Wellington.",
      },
    ],
  },
  {
    title: "Heat and air",
    reports: [
      {
        file: "walls.html",
        against: "vs theory and EnergyPlus",
        title: "Heat transfer through walls",
        description: "Massive and no-mass walls, with and without solar and long-wave radiation.",
      },
      {
        file: "convection_coefficients.html",
        against: "vs EnergyPlus",
        title: "Convection coefficients",
        description:
          "Natural (inside) and forced (outside) convection on vertical, tilted and horizontal surfaces.",
      },
      {
        file: "infiltration.html",
        against: "vs EnergyPlus",
        title: "Infiltration",
        description: "The effective leakage area and design flow rate models.",
      },
    ],
  },
  {
    title: "Solar, light and long-wave radiation",
    reports: [
      {
        file: "incident_solar_radiation.html",
        against: "vs EnergyPlus",
        title: "Incident solar radiation",
        description: "Walls facing north, south, east and west, in Barcelona and Wellington.",
      },
      {
        file: "incident_ir_radiation.html",
        against: "vs EnergyPlus",
        title: "Incident long-wave radiation",
        description: "Long-wave radiation on outside walls, in Barcelona and Wellington.",
      },
      {
        file: "view_factors.html",
        against: "vs analytic solutions",
        title: "View factors",
        description: "Tilted surfaces, street canyons and overhangs.",
      },
      {
        file: "daylight_coefficient.html",
        against: "vs Radiance",
        title: "Daylight coefficients",
        description: "Rooms with and without glass, with direct light only and with bounces.",
      },
      {
        file: "ray_tracer.html",
        against: "vs Radiance",
        title: "Ray tracing",
        description: "Metal, plastic and glass materials rendered by both tools.",
        note: "Works, but some materials need better modelling.",
      },
      {
        file: "light_sources.html",
        against: "vs analytic solutions",
        title: "Light sources",
        description: "Direct light from polygons and spheres, and glossy reflections.",
      },
    ],
  },
  {
    title: "Weather",
    reports: [
      {
        file: "weather.html",
        against: "vs EnergyPlus",
        title: "Weather files",
        description: "Whether SIMPLE samples EPW files the way EnergyPlus does.",
      },
      {
        file: "global_horizontal_from_cloud_cover.html",
        against: "in development",
        title: "Solar radiation from cloud cover",
        description: "Estimating solar radiation when a weather file lacks it.",
        note: "Not accurate yet, and not used by simulations.",
      },
    ],
  },
];
