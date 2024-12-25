# Simple Plugins
Bunch of simple CLAP plugins.

## Build Instructions
You need to have [Rust installed](https://rustup.rs/).

Note that you need nightly to build these plugins due to the [valib](https://github.com/SolarLiner/valib) dependency using nightly.

```shell
git clone --depth=1 https://github.com/insomnimus/simple-plugins
cd simple-plugins
# If you can execute shell scripts:
./build.sh --release
# Otherwise, build plugins manually, one by one
# E.g.
cargo run --bin bundler -- bundle simple-clipper --release
cargo run --bin bundler -- bundle simple-gain --release
```

After building, you can copy `.clap` files in `target/bundled/` to your CLAP plugin directory.

## Included Plugins
### Simple Clipper
This is a basic hard-clipper.

Parameters:
- Threshold: Clipping threshold
- Input Gain: Apply some gain before processing
- Output Gain: Apply some gain after processing
- Oversample: Oversampling amount, up to 32x

### Simple Gain
Just gain.
Comes in mono and stereo variants.

Mono parameters:
- Gain: Gain to apply

Stereo parameters:
- Gain: Gain to apply on both channels
- Left: Gain to apply on the left channel
- Right: Gain to apply on the right channel
