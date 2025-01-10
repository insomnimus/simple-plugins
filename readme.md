# Simple Plugins
Bunch of simple CLAP plugins.

## Build Instructions
You need to have [Rust installed](https://rustup.rs/).

```shell
git clone --depth=1 https://github.com/insomnimus/simple-plugins
cd simple-plugins
# If you can execute shell scripts:
./build.sh --release
# Or if you can run Powershell scripts:
./build.ps1 --release
# Otherwise, build plugins manually, one by one
# E.g.
# (the `cargo bundle` command is defined in .cargo/config.toml as an alias)
cargo bundle simple-clipper --release
cargo bundle  simple-gain --release
# ...and so on
```

After building, you can copy `.clap` files in `target/bundled/` to your CLAP plugin directory.

## Included Plugins
### Simple Channel Strip
A work-in-progress channel strip. Currently has a 5-band eq + high and low-pass filters, and input / output gain knobs.

### Simple Clipper
This is a basic hard-clipper.

Parameters:
- Threshold: Clipping threshold
- Input Gain: Apply some gain before processing
- Output Gain: Apply some gain after processing
- Oversample: Enable oversampling, which will improve quality but introduce significant latency
- Oversample On Render: Turn on oversampling while rendering, even if it's not enabled

### Simple Filter
High and low-pass filters.

Parameters:
- HPF Frequency: High-pass cutoff frequency
- HPF Q: Q value for the high-pass filter
- LPF Frequency: Low-pass cutoff frequency
- LPF Q: Q value for the low-pass filter

### Simple Gain
Just gain.
Comes in mono and stereo variants.

Mono parameters:
- Gain: Gain to apply

Stereo parameters:
- Gain: Gain to apply on both channels
- Left: Gain to apply on the left channel
- Right: Gain to apply on the right channel

### Simple Tube
Subtle tube saturation. Ported into a CLAP plugin from [AirWindows'](https://github.com/airwindows/airwindows) Tube plugin, because I don't like working with VST2 plugins.

Parameters:
- Amount: 0 to 100, amount of saturation (or drive)
- Input Gain: Gain before the signal hits the tube saturator
- Output Gain: Gain after the saturator
