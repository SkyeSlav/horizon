//! Client library integration tests linked into one binary.

#[path = "../../../horizon-client/tests/it/custom_component_api.rs"]
mod custom_component_api;
#[path = "../../../horizon-client/tests/it/custom_component_unregistered.rs"]
mod custom_component_unregistered;
#[path = "../../../horizon-client/tests/it/custom_component_wire.rs"]
mod custom_component_wire;
#[path = "../../../horizon-client/tests/it/datapack_join.rs"]
mod datapack_join;
#[path = "../../../horizon-client/tests/it/forced_chunks.rs"]
mod forced_chunks;
#[path = "../../../horizon-client/tests/it/generation_saves.rs"]
mod generation_saves;
#[path = "../../../horizon-client/tests/it/lan_bot.rs"]
mod lan_bot;
#[path = "../../../horizon-client/tests/it/load_defaults.rs"]
mod load_defaults;
#[path = "../../../horizon-client/tests/it/streaming_meshes.rs"]
mod streaming_meshes;
#[path = "../../../horizon-client/tests/it/tall_column.rs"]
mod tall_column;
#[path = "../../../horizon-client/tests/it/wide_states.rs"]
mod wide_states;
