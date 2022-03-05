# VES

## Introduction

VES stands for Virtual Entertainment System. Essentially, VES is an umbrella for common game development architecture
that is largely insprired by gaming consoles of the '80s and and '90s and initiatives like
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

## Project state

The main development in the VES project is currently driven by a [prototype](proto) implementation that is a
simplification of the Super Nintendo Entertainment System (SNES). Necessary tools are created on-demand and are a part
of the VES project, hopefully resulting in an ecosystem in which different architectures can share the same tooling and
(open) formats.
