# Glotaran converter lib

This is a _almost completely_ internal usage library for our lab to convert Time Resolved fluorescence
and Laser Flash Photolysys output files to Glotaran `wavelength explicit` input files.

The equipment we use are:
- Edinburgh Instruments L980 Spectrometer.
- Horiba Jovin-Yvon Spex Fluorolog FL3-11 Fluorometer with TRP equipment.


In the first case we can directly run:

```rust
use glotaran_converter_lib::run_lfp;

let filename = "example_lfp.txt";
let output_filename = run_lfp(filename).unwrap();

```
or if you `cargo install` the lib you can just run

``` bash
lfp filename 
```
If you want to use this lib with Fluorescence data I recommend the [`glotaran_converter_cli`](https://crates.io/crates/glotaran_converter_cli) lib, which is much more ergonomic, altough no documentation is available yet.