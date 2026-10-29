# SIMPLE Simulation

```
       [SIM]ulating
for peo[PLE]
```

![Run Everywhere](/public/img/run_everywhere.png)

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
["Do we need `SIMPLE`?"](https://germolinal.github.io/simple/book/what_is_simple.html)  section of the docs


## Examples

### Runs everywhere

Check [THIS LINK](https://buildingsforpeople.org/simple_demo/) to see `SIMPLE` running on your browser. No servers involved.

### Holistic simulation

Here is an example of Holistic simulations. Check these images, showcasing how 
`SIMPLE` can perform lighting and thermal simulation.

| Lighting simulation | Thermal simulation |
|-|-|
| ![Cornell](/public/img/cornell_glass.png) | ![SIMPLE vs Energy Plus](/public/img//SIMPLEvsEPlus.png)|


Also, HERE is


## Install

### Pre-built binaries (recommended)

Every release attaches pre-built binaries for macOS (Apple silicon), Linux
(x86-64) and Windows (x86-64) to its
[GitHub Release](https://github.com/germolinal/simple/releases/latest), and
the [website](https://germolinal.github.io/simple/#download) links to the
latest ones. Each archive contains the command-line tools `simple`, `spict`,
`sfluxmtx`, `sfalsecolor` and `scompare`.

Releases are the tested and validated versions of `SIMPLE`: the documentation
and validation reports on the website always describe the latest release.

> On macOS the binaries are not signed. If the system refuses to open them,
> run `xattr -d com.apple.quarantine simple`.

### Build from source

There are no nightly builds: the latest development version is the `main`
branch, which is tested but not validated until the next release. To use it,
or a platform without pre-built binaries, build it with
[Rust](https://rustup.rs):

```sh
# A release
cargo install --locked --features parallel --git https://github.com/germolinal/simple --tag v0.2.1 simple

# The development version (main)
cargo install --locked --features parallel --git https://github.com/germolinal/simple simple

# Or, from a clone (uses the Rust version pinned in rust-toolchain.toml)
cargo build --release --features parallel
```


## Documentation

I am hoping that the first people who get here are those niche humans who
nerd about buulding simulation, coding, and more. If you are 
one of them, please help me write tutorials and docs and all that. People who do 
Buildings do not often do programming.


Also, feel free to  [**CHECK THE DOCS**](https://germolinal.github.io/simple/)


## Development and releases

* **Pull requests to `main`** run formatting, clippy, tests, builds with every
  feature combination, the API docs, a website build and a security audit of
  the dependencies (`.github/workflows/test.yaml`). Jobs that can't be
  affected by a PR are skipped (e.g., no Rust tests for a docs-only change).
  Merging requires the "CI passed" check. Run `make ci` and `make test` to do
  the same locally.
* **Merging to `main`** publishes nothing.
* **Releases** are tags. From an up-to-date `main`, run `make release_patch`
  (or `release_minor`, `release_major`): it bumps the version on a
  `release/vX.Y.Z` branch, runs `make validate`, and opens a PR. Once that is
  merged, pull `main` and run `make release` to push the tag. Both refuse to
  go on unless everything is in place (e.g., stale validation reports). The
  tag (`.github/workflows/release.yaml`) is tested again, then gets a GitHub
  Release with binaries, and then the website, book, API docs and validation
  reports are published from it.
* Rust is pinned in `rust-toolchain.toml`, and Dependabot opens weekly PRs to
  update dependencies.


## License

The official legal document is the MIT license. I chose it because 
`SIMPLE` aims to make the world better and I want to encourage use. 

## Code of conduct

Say please and thank you and respect the rules of thermodynamics.