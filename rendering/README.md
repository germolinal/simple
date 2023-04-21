# Welcome

![build badge](https://github.com/SIMPLE-BuildingSimulation/rendering/actions/workflows/build.yaml/badge.svg)
![docs badge](https://github.com/SIMPLE-BuildingSimulation/rendering/actions/workflows/docs.yaml/badge.svg)
![tests badge](https://github.com/SIMPLE-BuildingSimulation/rendering/actions/workflows/tests.yaml/badge.svg)
[![codecov](https://codecov.io/gh/SIMPLE-BuildingSimulation/rendering/branch/main/graph/badge.svg?token=DPUWCNLBRF)](https://codecov.io/gh/SIMPLE-BuildingSimulation/rendering)
![style badge](https://github.com/SIMPLE-BuildingSimulation/rendering/actions/workflows/style.yaml/badge.svg)


This is a library that intends to help developing Physically Based rendering engines **that can be used for daylight simulation**. Some results seem to be OK when comparing
against [Radiance](https://www.radiance-online.org), but it is still slower than it. If you can help 

![Vs Rad](./readme_img/vsRad.png "Vs Radiance")

# Why this library?

I have been working in Building Simulation Tool space for a number of years now. During this time, I have tried to find ways to combine simulation tools and methods, and also to make simulation more accessible to people. For example, my masters focused on the integration of Radiance and EnergyPlus. It was then when I started developing [Groundhog](www.groundhoglighting.com), which intended to put Radiance in everyone’s hands. Developing Groundhog I realized that Radiance could use some sort of wrapper to make it a bit friendlier. This motivated Emp,  a project that has died already. 

During this time I have learned about an enormous amount of undeniable virtues that Radiance offer to us: it has been extensively validated, it is open source, and so on. Unfortunately, I have also seen many drawbacks. Radiance confesses one of them  quite clearly:

> “These routines are designed to aid the programmer who wishes to call Radiance as a library.  Unfortunately, the system was not originally intended to be run this way, and there are some awkward limitations to contend with…” (Radiance’s raycalls.c)

This is a big issue, as I want this library to become the core of the lighting and solar calculations in [SIMPLE Sim Tools](https://www.simplesim.tools).

Also, during the past decade, we have witnessed the gaming and animation industry invest millions in improving rendering techniques. Unfortunately, implementing these techniques into Radiance might be too big of an intervention, fundamentally changing how Radiance works.

Don’t be mistaken, though, I am not proposing to reinvent the wheel. The wheel—i.e., the knowledge of physics and rendering and computer sciences—is all available… what we need to do is to build a wheel with new materials and methods.


## A note on performance

This library could surely use some optimization (CONTRIBUTE!). However, it is also
worth mentioning that—internally—all raycasting is done by using Triangles 
as the single primitive (the reason for this is that SIMPLE is, first and foremost, an architectural tool, and buildings do not tend to be sphere). 
This implies that Spheres are triangulated, becoming more than 2000 triangles.  **So, do not just compare the performance of this library with a nother package using simls made out of just triangles, please**

# Contribute!

Check the Rust documentation [HERE](https://simple-buildingsimulation.github.io/rendering/rustdoc/doc/rendering/index.html).

Also, check the [validation report](https://simple-buildingsimulation.github.io/rendering/) (We need some help)


# What is included


### Rendering

`spict` calls a basic Ray-tracer. It attempts to 
emulate Radiance's `rpict` behaviour with `-aa 0` (i.e., there is no irradiance caching). 

This is, of course, quite naive when compared to Radiance. Contributions to make it faster are welcome.

```bash
# Create a render

spict -p 3 -5 2.25 -d 0 1 0 -b 3 -a 280 -s 10 -i ./cornell.rad -o ./cornell.hdr
```

> Note that `spict`—as the rest of this library—creates acceleration structres on the fly (i.e., we don't have an `oconv` program). Is this a good decision? let me know. In my experience, creating octrees is rarely a time-consuming process.



![Cornell](./readme_img/cornell_small.png "Cornell Box")

### "Falsecoloring"

`sfalsecolor` is a program that can create falsecolor versions of HDRE images.

> Note that, for now, it can only handle [uncompressed HDRE images](https://discourse.radiance-online.org/t/missing-pixels-in-hdre-image/5906/4).

```bash
# Create a falsecolour version of the previous image.
sfalsecolor -i ./cornell.hdr -o ./cornell_fc.jpeg -s 100
```


![Cornell](./readme_img/cornell_small_fc.jpeg "Cornell Box FC")

### Comparing images

`scompare` allows you to compare the absolute difference betwee two images.

```bash
# Compare the image we produced with one that only estimates direct lighting, producing a Black and White HDRE image

scompare -a ./rad_cornell.hdr -b ./cornell.hdr -o ./diff.hdr

```
![Cornell](./readme_img/diff.png "Cornell Box FC")

```bash
# Do the same, but produce a falsecolour JPEG

scompare -a ./rad_cornell.hdr -b ./cornell.hdr -o ./diff.hdr

```
![Cornell](./readme_img/diff_fc.jpeg "Cornell Box FC")

## Building and testing


I recommend using this command for building, which enables parallelism.

```bash
cargo build --release --features parallel
```

Then, run the unit tests as follows (we need more coverage... it is kind of hard
to design tests for this, I have found)

```bash
cargo test
```

Render some pre-built scenes (they'll end up in `./tests/scenes/images/`)

```bash
cargo test --features parallel --release --package rendering --test test_scenes -- --ignored 

```


### Rust features
* `default`: Uses `f64` by default and does not run on parallel.
* `parallel`: Enables [Rayon](https://docs.rs/rayon/latest/rayon/) for running  ray-tracing processes in parallel.
* `float`: Switches the default floating point number to `f32`




