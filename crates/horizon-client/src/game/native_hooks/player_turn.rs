use super::*;

/// The hook points. Each does nothing in Horizon.
impl Game<'_> {
    /// The local player's view is about to turn by (`xo`, `yo`) in `Entity.turn`
    /// units (0.15 degrees each), from the mouse, a pad, the arrow keys, touch
    /// drag or an agent's `turn`. Change either to redirect the turn, or set
    /// `cancelled` to leave the view where it is. Client-side.
    pub(in crate::game) fn native_player_turn(
        &self,
        xo: &mut f64,
        yo: &mut f64,
        cancel: &mut bool,
    ) {
        self.fire_player_turn(xo, yo, cancel);
    }
}

#[cfg(test)]
mod player_turn_tests {
    use mod_api::{ClientHost, Flow, Hooks, ModEntry, PlayerTurnEvent};
    use mod_platform::testing::with_mods;
    use std::sync::{Arc, Mutex};

    /// Records the turn deltas it saw, then cancels the turn.
    struct RecordAndCancel(Arc<Mutex<Vec<(f64, f64)>>>);
    #[mod_api::__subscriptions]
    impl Hooks for RecordAndCancel {
        fn player_turn(&self, _: &dyn ClientHost, event: &mut PlayerTurnEvent) -> Flow {
            self.0.lock().unwrap().push((event.xo, event.yo));
            event.cancelled = true;
            Flow::Continue
        }
    }

    /// Halves the turn deltas without cancelling.
    struct Halve;
    #[mod_api::__subscriptions]
    impl Hooks for Halve {
        fn player_turn(&self, _: &dyn ClientHost, event: &mut PlayerTurnEvent) -> Flow {
            event.xo *= 0.5;
            event.yo *= 0.5;
            Flow::Continue
        }
    }

    /// A cancelling mod hears the turn with the deltas the game sent and the view
    /// does not move; a halving mod's turn moves the view by half; without a
    /// listener the turn moves the view as vanilla does.
    #[test]
    #[ignore = "daily"]
    fn a_client_mod_cancels_and_redirects_the_player_turn() {
        crate::game::native_entity_refs::tests::with_client_game(|game| {
            let (yaw, pitch) = (game.yaw, game.pitch);
            game.turn_player(10.0, 5.0);
            assert!((game.yaw - yaw - 1.5).abs() < 1e-6, "vanilla yaw turn");
            assert!((game.pitch - pitch - 0.75).abs() < 1e-6, "vanilla pitch turn");
        });
        let heard = Arc::new(Mutex::new(Vec::new()));
        let mods: mod_api::Mods = vec![ModEntry::new(
            "record-cancel",
            Box::new(RecordAndCancel(Arc::clone(&heard))),
        )]
        .into();
        with_mods(mods, || {
            crate::game::native_entity_refs::tests::with_client_game(|game| {
                let (yaw, pitch) = (game.yaw, game.pitch);
                game.turn_player(10.0, 5.0);
                assert_eq!((game.yaw, game.pitch), (yaw, pitch), "cancelled turn");
            });
        });
        assert_eq!(*heard.lock().unwrap(), [(10.0, 5.0)]);
        let mods: mod_api::Mods =
            vec![ModEntry::new("halve", Box::new(Halve))].into();
        with_mods(mods, || {
            crate::game::native_entity_refs::tests::with_client_game(|game| {
                let (yaw, pitch) = (game.yaw, game.pitch);
                game.turn_player(10.0, 5.0);
                assert!((game.yaw - yaw - 0.75).abs() < 1e-6, "halved yaw turn");
                assert!((game.pitch - pitch - 0.375).abs() < 1e-6, "halved pitch turn");
            });
        });
    }
}
