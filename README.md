# VES

## Introduction

VES stands for Virtual Entertainment System. Essentially, VES is an umbrella project for common game development
architecture that is largely inspired by gaming consoles of the '80s and and '90s and initiatives like
[libretro](https://www.libretro.com/).

## Motivation

The main motivations for writing VES are

* A fascination with the game consoles of old.
* Learning Rust in the scope of a project that touches different areas of software development.
* Using interesting technologies like WebAssembly.
* Providing a more ergonomic (and safe) alternative to [libretro](https://www.libretro.com/).

## Concepts

One of the main concepts of the VES project is the Core-Game Architecture. This idea is inspired by the
[libretro](https://www.libretro.com/) design and VES borrows a lot of the terminology from this source.

A *core* is the software equivalent of a game console. It is responsible for providing a useful (and restrictive)
abstraction against which a *game* can be developed. The core is responsible for running the game in a contained
environment. The game can only run its own code and communicate with the core over the API for the relevant
*core architecture*. A game can not interact with anything outside of the contained environment. Essentially, the core
architecture describes the "type" of console and core is an implementation of such an architecture. A game can be run on
any core implementation that adheres to the relevant architecture. Similarly, a core can run any game that is
implemented against its architecture.

The core architecture approach has the following advantages:

* The core architecture can be used to force a desired aesthetic.
* The game does not have to worry about underlying complexities like graphics rendering, audio processing, user input,
  etc.

In addition to this the VES project also seeks to provide components that are common between different core
architectures, such as tooling for extracting and organizing artwork. 

## Technical choices

### Programming language

Rust is used as the main language for the project. Its targets on both safety and speed in combination with a general
focus on ergonomics and code clarity make it an excellent all-rounder for the different types of software components
that the project requires. Additionally, the ecosystem provides a lot of libraries and tools that can be used off the
shelf.

### Containerization

Although not a strict requirement for core architectures, it is assumed that WebAssembly is used as a bridge between the
core and the game. The choice for WebAssembly is motivated by several aspects:

* It provides an efficient and memory-safe sandboxed environment in which the core can execute the game code.
* It is an open standard.
* It is programming-language-agnostic.
* Can easily be integrated on both desktop and web.
* It is natively supported by Rust for both the host side (e.g. [`wasmtime`](https://crates.io/crates/wasmtime)) and the
  sandboxed side (via a dedicated compilation target).

The following is a graphical representation of a common Core-Game Architecture implementation using WebAssembly (WASM).

```mermaid
graph TD
    core_fe[Front-end]
    core[Core]
    wasm_rt["WASM run-time"]
    wasm_binding["WASM binding"]
    game[Game]

    core_fe -->|context| core
    core -->|render frame| core_fe
    core -->|game API calls| wasm_rt
    wasm_rt -->|core API calls| core
    wasm_rt -->|"(WASM) export calls"| wasm_binding
    wasm_binding -->|"(WASM) import calls"| wasm_rt
    wasm_binding -->|game API calls| game
    game -->|core API calls| wasm_binding
```

* **Front-end:** Responsible for handling platform-specific components like user input (including gamepads), graphics,
  audio, and whatever the architecture might require. A front-end can use arbitrary technology, as long as it is
  compatible with the core. Front-ends can be implemented in, for example, [libretro](https://www.libretro.com/) or
  [SDL](https://www.libsdl.org/). Note that, unlike the libretro approach, VES does not dictate anything about the
  front-end/core relationship. This means that a front-end could be entirely Linux terminal-based or a controller could
  have 34 buttons. On the one hand, this makes it harder to have a common front-end for different core types. On the
  other hand, it can greatly simplify the core implementation. Note that an architecture *can* go the libretro route, it
  just doesn't *have to*.
* **Core:** A component that implements the core-side of the architecture (as desribed in some kind of specification)
  and interacts with the front-end and WASM run-time to provide a complete game loop.
* **WASM run-time:** The WebAssembly run-time. This is the component that is responsible for loading up the WASM game
  file, providing exported functions from the game to the core and providing imported functions from the core to the
  game. An example is [`wasmtime`](https://crates.io/crates/wasmtime).
* **WASM binding:** The WebAssembly binding on the game side. The game is compiled as a WebAssembly binary file
  (`.wasm`). The WASM binding provides exported functions from the game to the outside world and imports core functions
  for use in the game code.
* **Game:** The game implementation itself. Its API functions get called by the core, in response to which it can
  execute its internal logic that represents the game.

### Serialization

The [`serde`](https://crates.io/crates/serde) library is used for any serialization/deserialization of data, due to its
low overhead and ergonomic integration into any Rust code.

For binary serialization [`bincode`](https://crates.io/crates/bincode) is the format of choice. The main reasons for
this are compactness and platform-independence (which is a must when storing data to files for later use).

## Project state

The main development in the VES project is currently driven by a [prototype](proto) implementation that is a
simplification of the Super Nintendo Entertainment System (SNES). Necessary tools are created on-demand and are a part
of the VES project, hopefully resulting in an ecosystem in which different architectures can share the same tooling and
(open) formats.
