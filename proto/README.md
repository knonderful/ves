# VES Prototype

This directory contains a prototype implementation for the
[VES Core-Game Architecture](../README.md#concepts). The core architecture for this prototype is
based on the Super Nintendo Entertainment System (SNES).

## Goals

The main goals for this prototype are:

* Provide a reference implementation that demonstrates the VES Core-Game Architecture philosophy.
* Stay close to the restrictions of the SNES without forcing unnecessary complexity onto the architecture.
* Start small (i.e. only support for sprites) and iteratively extend the capabilities.
* Develop the necessary tooling on-the-go.

## Specification

This section describes the specification of the core architecture for the prototype. Since development is done
iteratively, the design also is created iteratively. Therefor, this specification is incomplete and should not be
considered stable any time soon. 

### Architectural decisions

To safeguard the easthetic of the original SNES, the prototype imposes restrictions similar to the original console.

* At most 128 sprites can be specified at a time.

Some SNES restrictions are relaxed, as they don't really significantly impact the aesthetic:

* A simplified video resource approach.
  * No VRAM management and no offsets into the VRAM defined anywhere.
  * Instead, the core provides large tables for tiles and palettes, large enough for most games to not have to do any
    custom resource management in the game code.

### API

The following is a descriptive representation of the APIs between the code and the game. Refer to the actual code for
details.

#### Core

The core must implement the following trait.

```rust
/// The prototype core API.
pub trait Core {
    /// Sets an OAM entry.
    ///
    /// # Arguments
    ///
    /// * `index`: The index into the OAM table.
    /// * `entry`: The entry.
    fn oam_set(&self, index: &OamTableIndex, entry: &OamTableEntry);

    /// Sets a palette entry.
    ///
    /// # Arguments
    ///
    /// * `palette`: The index of the palette in the palette table.
    /// * `index`: The index inside the palette.
    /// * `color`: The color to set.
    fn palette_set(&self, palette: &PaletteTableIndex, index: &PaletteIndex, color: &PaletteColor);
}
```

#### Game

The game must implement the following trait.

```rust
/// The prototype game API.
pub trait Game {
  /// Create a game instance.
  ///
  /// # Arguments
  ///
  /// * `core`: The bootstrap to the core API. This instance should be used by the game
  ///           implementation to interact with the core.
  fn new(core: CoreBootstrap) -> Self;

  /// Advance the game by one step.
  fn step(&mut self);
}
```

`CoreBootstrap` is an implementation of the aforementioned `Core` trait.

#### VROM

In addition to the aforementioned traits, the core and game share read-only data via a WASM section called `vrom`. This
section contains graphical assets, such as tiles, that are part of the game, but do not change over time. This section
is loaded by the core as part of the initialization routine and is not used directly by the game code. However, the
objects inside the VROM are can be referenced in the API calls between the core and the game. For instance, an OAM entry
might include an index to the list of tiles in VROM, such that the core knows which tile to use for the sprite.

The VROM is generated on the game side at build-time, in [`build.rs`](game/build.rs). It is serialized as `bincode`. The
actual data structure is intentionally left unspecified in this document, since it is still subject to regular change.
Have a look at the code for more details.

The mains reason for having VROM, rather than sending the relevant data via API calls to the core, are:

* API simplicity that favors the 99% use-case.
* Reduced bandwidth over the WASM bus.
* Reduced code on the game side.
* A reusable approach for that can also be used for other core architectures.
