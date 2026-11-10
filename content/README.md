# Construction Sticks — carried content (`.hmod`)

Content-only copy of **Construction Sticks 1.5.0** (MC 1.21.1 NeoForge, MIT),
carried with `horizon-port-pack check` and given a folder manifest so it can be
loaded with `--dev-mod` or packed into an `.hmod`. Behaviour (placement, undo,
upgrades, GUI, networking) belongs to the native port; this folder is data and
assets only.

## Provenance

| | |
|---|---|
| Modrinth project | `ooyjDLZt` (`construction-sticks`) |
| Version | `1.5.0`, id `5dwDOCYP` |
| Jar | `ConstructionSticks-1.21.1-1.5.0.jar`, 264 739 bytes |
| SHA-1 | `4d3c031094441d35b54baad1727ae118200bcd2b` (matches Modrinth) |
| Licence | MIT |

Proof (jar, carry output, check/carry reports, built `.hmod`) is in
`../../../_shared/carry-work/constructionstick/`.

## What is included

```
content/
├── horizon-mod.toml                  id constructionstick, API 0.2, both, registries
├── assets/constructionstick/         51 files
│   ├── items/                       10 item model definitions (26.3 format)
│   ├── models/item/                 20 models (5 sticks + 5 templates + angel/destruction variants)
│   ├── textures/item/               13 PNGs
│   └── lang/                        8 files (en_us, ja_jp, lzh, pt_br, ru_ru, tr_tr, zh_cn, zh_tw)
├── data/constructionstick/           91 files
│   ├── advancement/                 42 (root, 5 templates, iron_stick, 10 tools, 25 smithing)
│   ├── horizon/item/                10 item declarations (template minecraft:stick)
│   ├── horizon/creative_tab/        1 tab (constructionstick:tab, 10 items)  [hand-added]
│   ├── recipe/                      10 crafting_shaped recipes
│   ├── recipe/smithing_upgrade/     25 constructionstick:smithing_upgrade (inert, see gaps)
│   └── tags/                        3 (item construction_sticks, block non_placable/non_replaceable)
├── data/c/tags/item/                14 convention tags                              [hand-added]
└── patches/                         hand fixes + apply.sh (not packed)
```

Totals: **156 packed files** (51 assets + 105 data); the `.hmod` has 157
archive entries including `horizon-mod.toml`. A further 16 files under
`patches/` are the re-appliable hand fixes and are not packed.

Item ids (`constructionstick:`): `wooden_stick`, `copper_stick`, `iron_stick`,
`diamond_stick`, `netherite_stick`, `template_angel`, `template_battery`,
`template_destruction`, `template_replacement`, `template_unbreakable`.
There are no blocks, so no block templates.

Recipes: 5 sticks (3 wooden rods + one ingot/gem in a diagonal) and 5 templates
(glass/feather/gold, glass/redstone/potato, glass/ender pearl/sculk,
glass/nether star/crying obsidian, glass/diamond pickaxe/tnt), all
`minecraft:crafting_shaped`, upgraded 1.21.1 → 26.3 by the carry.

## Hand fixes

1. **14 `c:` item tags** under `data/c/tags/item/` — without them every recipe
   and advancement criterion using a convention tag expands to an empty
   ingredient (uncraftable). Mapping: `rods/wooden`→stick,
   `ingots/{copper,iron,gold,netherite}`→the ingot, `gems/diamond`,
   `dusts/redstone`→redstone, `feathers`, `ender_pearls`, `nether_stars`,
   `obsidians/crying`, `storage_blocks/{copper,redstone}`→the block,
   `glass_blocks`→glass + tinted + 16 stained glass.
2. **Creative tab** `data/constructionstick/horizon/creative_tab/tab.json`
   (`constructionstick:tab`): the Java mod registers a tab in code; the title
   key `itemGroup.constructionstick.tab` was already in the lang files.
3. **Lang needed no rewrite.** All 8 files already use vanilla keys
   (`item.constructionstick.*`, `itemGroup.constructionstick.tab`,
   `advancement.*`, `key.*`, `stat.*`); there are no NeoForge-style
   `constructionstick.item.*` keys.
4. **No block-template repair**: the carry reported 0 blocks.

`patches/apply.sh` re-copies the hand-added files (`patches/extra/data`) after a
re-carry; the carry itself never touches top-level `assets/` or `data/`.

## Gaps for the native port / hand follow-up

- **25 `constructionstick:smithing_upgrade` recipes** are carried as files but
  the recipe type does not exist in Horizon, so they never match. Upgrading
  (and the upgrade components `constructionstick:{angel,battery,destruction,
  replacement,unbreakable}`) needs native code.
- **`#c:relocation_not_supported`** is referenced by the carried
  `tags/block/non_replaceable.json` but is not defined; it resolves empty. The
  other member, `constructionstick:non_placable`, only lists Powah reactor
  blocks with `required: false`, so the tag is empty in a vanilla world. Not
  guessed on purpose.
- **Upgrade overlays**: the 1.21.1 item models use `overrides`/`predicate`,
  which the 26.3 item-model format ignores; `*_angel` and `*_destruction`
  models (10 files) load but are never selected. Add `minecraft:select`
  definitions keyed on the upgrade components in the native pass.
- **Item stats**: all 10 declarations are plain `minecraft:stick` (stack 64,
  no damage). The originals stack to 1 and are damageable, with per-tier
  durability and upgrade components — native `max_stack_size`/`max_damage`/
  component work.
- **Skipped by the carry** (native behaviour): 8 item `use`/`on hit` paths,
  1 screen, 5 network messages, 2 game events. `assets/emi/aliases/
  construction_sticks.json` went to `outside/` (foreign namespace).

## Build / check

```sh
cd ports/constructionstick/content
cargo run --locked --offline --quiet \
  --manifest-path ../../../horizon/crates/cargo-horizon-mod/Cargo.toml -- horizon-mod build
cargo run --locked --offline --quiet \
  --manifest-path ../../../horizon/crates/cargo-horizon-mod/Cargo.toml -- horizon-mod check \
  target/horizon-mod/constructionstick.hmod
```

Result: `constructionstick 1.5.0 (mod, API 0.2, both)`, 86 208 bytes,
SHA-256 `3f1cd6f34fceed3a7370358f4d68d405dadda861547af2fd013605e495596d3f`,
unsigned, "no code: content only".
