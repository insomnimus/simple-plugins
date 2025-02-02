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
Note: There's more than what's listed here. You can consider what's not listed as being used during development for testing out ideas or code.

### Depth
Inspired by [this video from Dan Worrall](https://www.youtube.com/watch?v=uZ9WQDojQt8), Depth is a stereo depth enhancer.

Use it on hard panned elements of your mix! Great on double tracked guitars.
Don't use on mono sources; it'll only add delay, no depth enhancement.

Parameters:
- Depth: 0 to 40; amount of depth. The sweet spot is usually around 4.
	* Currently this number is used to calculate the mid channel delay; a value of `4` means `4 / 40` of a millisecond, or `0.1ms`.

### Simple Channel Strip
A work-in-progress channel strip. Currently has tube drive, a 5-band eq + high and low-pass filters, and input / output gain knobs.

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
