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

/// Install-once registries need a fresh process, not just a serial test lock.
fn in_own_process(test: &str) -> bool {
    const CHILD: &str = "HORIZON_CLIENT_LIB_ISOLATED_TEST";
    if std::env::var(CHILD).as_deref() == Ok(test) {
        return false;
    }
    assert!(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--include-ignored", "--exact", test, "--nocapture"])
            .env(CHILD, test)
            .status()
            .unwrap()
            .success(),
        "isolated test {test} failed"
    );
    true
}
