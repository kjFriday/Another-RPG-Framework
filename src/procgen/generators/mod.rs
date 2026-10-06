//! One file per generator. Each exposes a unit struct (a struct with
//! no fields) that implements [`super::Generator`].
//!
//! | struct           | makes          | technique                       |
//! |------------------|----------------|---------------------------------|
//! | [`ItemGen`]      | named item     | weighted rarity + affix tables  |
//! | [`CaveGen`]      | cave map       | cellular automaton smoothing    |
//! | [`DungeonGen`]   | rooms+corridor | binary space partitioning (BSP) |
//! | [`TownGen`]      | streets + lots | recursive road subdivision      |
//! | [`OverworldGen`] | terrain        | fractal value noise + biomes    |

mod cave;
mod dungeon;
mod item;
mod overworld;
mod town;

pub use cave::CaveGen;
pub use dungeon::DungeonGen;
pub use item::{ItemData, ItemGen};
pub use overworld::OverworldGen;
pub use town::TownGen;
