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
