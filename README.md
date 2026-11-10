# constructionstick (Horizon port)

A Horizon port of [constructionstick](Mrbysco/ConstructionSticks) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- A right click with a stick on a block claims the click and fills the **plane
- of the clicked face** with copies of the clicked block: start at
- `clicked + face.normal`, breadth-first over both in-plane axes and their
- diagonals, accepting a cell only when the block behind it (`cell - normal`)
- is still the clicked block.
- Bounded by the tier limit (`3/9/27/128/1024`) and the player's supply; each
- placed cell consumes one item of the clicked block's kind and wears the held
- stick by one; if the supply runs out mid-run the last block is rolled back.

## Not yet ported

- Placement uses `default_state` of the clicked block (item id = block id); a
- directional block loses its facing, and blocks whose item id differs from the
- block id are not supported. Upstream computes `getStateForPlacement` with a
- fake hit context.
- Only `minecraft:air` counts as placeable (upstream also replaces replaceable
- blocks when the `replace` option is on); no living-entity or block-entity
- blacklist check yet.
- No options screen/undo/keys, no angel/destruction/replacement modes, no

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: Mrbysco/ConstructionSticks. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
