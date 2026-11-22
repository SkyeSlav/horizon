//! The `player_turn` hook: declarations the mod API generator reads (see `hooks/mod.rs`).

pub trait Hooks {
    /// The local player's view is about to turn by (`xo`, `yo`) in `Entity.turn`
    /// units (0.15 degrees each), from the mouse, a pad, the arrow keys, touch
    /// drag or an agent's `turn`. Change either to redirect the turn, or set
    /// `cancelled` to leave the view where it is. Client-side.
    #[hook(
        side = "client",
        contract = "cancel",
        out = "cancel",
        neoforge = "none (NeoForge's CalculatePlayerTurnEvent carries no deltas)",
        cancellable = true
    )]
    fn player_turn();
}
