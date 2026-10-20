# Jotego Analogizer Config plugin

A [Pocket plugin](https://github.com/openfpga-library/pocket-plugin) port of the Analogizer settings wizard feature of pupdate, for Jotego's cores with the [Analogizer](https://github.com/RndMnkIII/Analogizer) adapter.

It asks two questions and writes `Assets/jtpatreon/common/crtcfg.bin` on the SD card:

1. Analog video output (RGBS, RGsB, YPbPr, Y/C NTSC, Y/C PAL, scandoubler with 0/25/50/75%
   scanlines, or disabled)
2. SNAC controller (none, DB15, NES, SNES, PCE 2/6 button, PCE multitap, PSX digital)

Re-run to change the settings: the values currently in `crtcfg.bin` are marked
`(current)`. Cancelling at either prompt leaves the file untouched.

## Build

```
rustup target add wasm32-wasip1
scripts/build.sh               # target/wasm32-wasip1/release/plugin.wasm
```

## Test

`cargo test` checks the encoder against `tests/golden.json`, which was generated
from pupdate's original C# encoder (pupdate 5.8, `JotegoAnalogizerSettingsService`): the bytes
for every one of the 70 video/controller letter pairs.

`tools/run_plugin.py` (needs `pip install extism`) runs the built plugin headless against any
folder standing in for the SD card (created if missing), answering each prompt in order:

```
python3 tools/run_plugin.py target/wasm32-wasip1/release/plugin.wasm /tmp/sd --answer "C:" --answer "D:"
xxd /tmp/sd/Assets/jtpatreon/common/crtcfg.bin
```

`--kill N` cancels at prompt N to check that nothing is written.


## License

MIT (see [LICENSE](LICENSE)).
