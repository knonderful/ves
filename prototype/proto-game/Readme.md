# VES Prototype - Game

## Introduction

This is the module for the game-side of the VES prototype.

## Musings

### Game loop

The game needs to track the following information:

* Active entities in the scene.
  * Active, in this context, means that the entity is either on-screen or not too far off-screen that it doesn't "live" anymore.
  * For each entity:
    * Position in scene. 
    * State of animation (e.g. `Animation`, `AnimationFrame`, `Cel`).
    * Relevant attributes like hit points, momentum, current action, etc.

The OAM is explicitly not tracked on the game side. Instead, a "clean" OAM table is used for each frame and sprites are added anew every frame. This gets around the otherwise hard problem of managing OAM slots between entities, since an entity might be using 4 sprites on the current frame, but 5 sprites on the next frame and then it needs to "push" other sprites out of the way, which might result in a table overflow.

The general game loop could look something like this:

* Perform game logic update.
  * Handle user input.
  * Calculate physics.
  * Advance the AI.
* For each entity:
  * Remove any entities that have left the scene.
  * Load all sprites into OAM.
    * If there is not enough space for the sprites of the entity, the entity should probably be dropped, which means it is despawned.
* Add any new entities to the scene.
  * Repeat previous steps for the new entities.
