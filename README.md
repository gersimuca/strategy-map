# Strategy RTS Engine

A real-time strategy (RTS) engine. A modular game architecture with tile-based maps, unit selection, and A* pathfinding.

Built using:

* eframe
* egui

---

# Features

## World system

* Tile-based grid map
* Walkable and blocked tiles
* Scalable map size

## Unit system

* Click-to-select units
* Multiple unit support
* Move commands using pathfinding

## Pathfinding

* A* algorithm implementation
* Obstacle avoidance
* Grid-based navigation

## Camera controls

* Pan using mouse drag
* Zoom using mouse wheel

---

# Project structure

```text
strategy_rts/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── app.rs
    ├── engine/
    │   ├── mod.rs
    │   ├── camera.rs
    │   ├── combat.rs
    │   ├── effects.rs
    │   ├── enemy.rs
    │   ├── input.rs
    │   ├── map.rs
    │   ├── pathfinding.rs
    │   ├── projectile.rs
    │   ├── selection.rs
    │   └── unit.rs
    └── rendering/
    │   ├── mod.rs
    │   └── draw.rs
    └── math/
        └── mod.rs

```

---

# Requirements

## Install Rust

If Rust is not installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:

```bash
rustc --version
cargo --version
```

---

# Setup

## Build project

Cargo will automatically download dependencies:

```bash
cargo build
```

---

# Run

Run the application in development mode:

```bash
cargo run
```

---

# Controls

| Action      | Input                |
| ----------- | -------------------- |
| Select unit | Left click on unit   |
| Move unit   | Left click on ground |
| Pan camera  | Mouse drag           |
| Zoom        | Mouse wheel          |

---

# Architecture overview

## Engine layer

Contains all simulation logic:

* Map system
* Unit system
* Pathfinding
* Selection logic

## Rendering layer

Responsible only for drawing:

* Tiles
* Units
* Visual feedback

## Application layer

Connects input, engine, and rendering into a single loop.

---

# Extending the engine

This project is designed to be extended into a full RTS system.

Possible upgrades:

## Combat system

* Health points
* Attack ranges
* Projectiles

## Economy and buildings

* Resource collection
* Building placement system

## Fog of war

* Visibility system per unit
* Exploration tracking

## AI system

* Enemy behaviors
* State machines or behavior trees

## Networking

* Authoritative Rust server
* Client prediction and synchronization

---

# Performance notes

This implementation is intentionally simple:

* No ECS architecture
* Single-threaded simulation
* CPU-based rendering via egui

It is suitable for:

* Prototyping RTS mechanics
* Learning engine architecture
* Small-scale strategy games

For larger scale projects, consider migrating to:

* ECS architecture (for example Bevy-style design)
* GPU rendering (wgpu)
* Server-client separation for multiplayer
