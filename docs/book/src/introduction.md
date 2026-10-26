# `SIMPLE` - Simulating for People

```
       [SIM]ulating
for peo[PLE]
```

Thanks for your interest in `SIMPLE`, a simulation tool aiming to
satisfy the present needs of Building Performance Simulation. Long story short,
`SIMPLE` aims to:

1. Allow for a holistic building performance simulation using best-practice
algorithms for each domain
2. Make it easier to account for the complex interactions between Humans
and Buildings
3. Make it possible to implement complex algorithms, such as Model Predictive Control.
4. Be deployable in multiple environments, including Windows, Linux and Mac, the cloud, the browser (i.e., WebAssembly)
5. Use modern languages, toolset, and aim for clarity of source code (I do my best...)

You can read more about this in the
["Do we need `SIMPLE`? section"](what_is_simple.md#do-we-need-simple?)

## What to expect from this book?

For now this book has two parts:

1. **User guide**: This part is written manually, and it is meant to help you learn and understand `SIMPLE`. This part is mainly focused on users. For example, (in the future) it might help you migrate from other tools—if you wanted to—, to understand best building-modelling practices, and so on. If you are looking for information regarding math and equations, then you'd better be looking at the developer documentation (e.g., [this](https://simple-buildingsimulation.github.io/heat/rustdoc/doc/heat/model/struct.ThermalModel.html#method.calculate_zones_abc)). The developer documentation is meant not only for people who want to code but also for people who want to understand the internals of `SIMPLE`.
2. **Input/Output Reference guide**: This is an automatically generated guide, is updated every time the source code of `SIMPLE`'s input model is updated. This section is meant to be a reference guide, helping
