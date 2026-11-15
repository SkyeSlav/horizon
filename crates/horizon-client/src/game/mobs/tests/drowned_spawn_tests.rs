//! A summoned drowned (`summon minecraft:drowned x y z`, no NBT, so
//! `Drowned.finalizeSpawn` runs) holds a nautilus shell in its offhand with a
//! 0.03 chance, as a guaranteed drop (`setGuaranteedDrop(OFFHAND)`). Over 400
//! drowned, the live offhand, the saved offhand and the saved chance agree,
//! and the count is in the original probe's wide range (1..30), not an exact
//! random result.
use crate::game::*;
fn game(resources: &mut Option<std::sync::Arc<Resources>>) -> Game<'_> {
    let mut game = crate::game::game_tests::tests::server_game(resources);
    game.build_registries();
    game.set_allow_commands(true);
    // The fixture does not start streaming: make every forced column resident.
    for x in -1..=0 {
        for z in -1..=0 {
            game.world.chunks.insert(
                (x, z),
                std::sync::Arc::new(horizon_client::chunk::Chunk::empty(x, z)),
            );
        }
    }
    for command in [
        "difficulty easy",
        "gamerule spawn_mobs false",
        "forceload add -16 -16 15 15",
        "fill -16 99 -16 15 99 15 stone",
        "tick freeze",
    ] {
        assert!(game.run_command(command), "{command}");
    }
    game
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn summoned_drowned_hold_shells_as_guaranteed_drops() {
    let mut game_resources = None;
    let mut game = game(&mut game_resources);
    for index in 0..400 {
        let (x, z) = (f64::from(index % 20) - 9.5, f64::from(index / 20) - 9.5);
        assert!(game.run_command(&format!("summon drowned {x} 100 {z}")));
    }
    let drowned: Vec<u32> = game
        .mobs
        .iter()
        .filter(|m| m.species() == horizon_client::mobs::Species::Drowned)
        .map(|m| m.id)
        .collect();
    assert_eq!(drowned.len(), 400);
    let (mut live, mut saved, mut guaranteed, mut offhands) = (0, 0, 0, Vec::new());
    for id in drowned {
        let m = game.mobs.iter().find(|m| m.id == id).unwrap();
        offhands.push(m.extra.off_hand_id().map(str::to_owned));
        if m.extra.off_hand_id() == Some("minecraft:nautilus_shell") {
            live += 1;
            if horizon_client::mobs::drops::drop_chance(m, "offhand") == 2.0 {
                guaranteed += 1;
            }
        }
        let tag = game
            .command_target_nbt(crate::game::commands::Target::Mob(id))
            .unwrap();
        let body = horizon_client::snbt::pretty(&tag).plain();
        if body.contains("offhand: {count: 1, id: \"minecraft:nautilus_shell\"}") {
            saved += 1;
        }
    }
    let other: Vec<_> = offhands.iter().flatten().collect();
    eprintln!("live {live} saved {saved} guaranteed {guaranteed}; other offhands {other:?}");
    assert!((1..=30).contains(&live), "live shells {live}");
    assert_eq!(saved, live, "the saved offhand");
    assert_eq!(guaranteed, live, "every shell is a guaranteed drop");
}

/// The other side of the shell opportunity, through the same
/// `spawning::finalize` route on mobs that `summon` with NBT left
/// unfinalized, from fixed seeds of the level's random it draws from. The
/// expected values were read from this route without the shell branch.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn only_an_empty_offhand_drowned_draws_for_the_shell() {
    use horizon_client::mobs::drops::{DEFAULT_DROP_CHANCE, SLOTS, drop_chance};
    use horizon_core::random::{LegacyRandom, RandomSource};
    let mut game_resources = None;
    let mut game = game(&mut game_resources);
    for command in [
        "summon drowned 0.5 100 0.5 {NoAI:1b}",
        "summon zombie 2.5 100 0.5 {NoAI:1b}",
        "summon husk 4.5 100 0.5 {NoAI:1b}",
    ] {
        assert!(game.run_command(command), "{command}");
    }
    let difficulty = game.local_difficulty([0.5, 100.0, 0.5]);
    let base = |game: &Game<'_>, kind: &str| {
        let m = game.mobs.iter().find(|m| &*m.kind.id == kind).unwrap();
        assert_eq!(
            m.extra.off_hand_id(), None,
            "{kind}: not finalized by summon with NBT"
        );
        m.clone()
    };
    // The fork's finalize funnel draws from the level's random (`world.random`).
    let finalize = |game: &mut Game<'_>, mut m: horizon_client::mobs::Mob, seed: i64| {
        m.random.set_seed(seed);
        game.world.random =
            horizon_core::random::LevelRandom::for_level(LegacyRandom::new(seed));
        game.finalize_plain(
            &mut m,
            &difficulty,
            horizon_client::mobs::spawn_rules::Reason::Natural,
        );
        (m, game.world.random.clone())
    };
    // Other species draw nothing more: the end state of their random is the
    // one the route had before.
    let before = [
        (1, 200055422738958u64),
        (2, 259836486421929),
        (3, 239909465194272),
    ];
    for kind in ["minecraft:zombie", "minecraft:husk"] {
        for (seed, state) in before {
            let (m, level) = finalize(&mut game, base(&game, kind), seed);
            assert_eq!(level.state(), state, "{kind} seed {seed}");
            assert_eq!(m.extra.off_hand_id(), None);
        }
    }
    // An occupied offhand: no shell draw, the item and the chances stay.
    let occupied = [
        (1, 200055422738958u64),
        (2, 241258967538006),
        (3, 63989017391689),
    ];
    for (seed, state) in occupied {
        let mut m = base(&game, "minecraft:drowned");
        m.extra.set_slot_id(
            horizon_items::equipment::OFFHAND,
            Some("minecraft:dandelion".into()),
        );
        let (m, level) = finalize(&mut game, m, seed);
        assert_eq!(level.state(), state, "occupied seed {seed}");
        assert_eq!(m.extra.off_hand_id(), Some("minecraft:dandelion"));
        assert_eq!(m.extra.drop_chances, None);
    }
    // An empty offhand: exactly one more draw, after every other one, and
    // the shell when it is under 0.03.
    let mut shells = 0;
    for seed in 0..400 {
        let mut occupied = base(&game, "minecraft:drowned");
        occupied.extra.set_slot_id(
            horizon_items::equipment::OFFHAND,
            Some("minecraft:dandelion".into()),
        );
        let (occupied, mut after) = finalize(&mut game, occupied, seed);
        let (empty, level) = finalize(&mut game, base(&game, "minecraft:drowned"), seed);
        let draw = after.next_float();
        assert_eq!(level.state(), after.state(), "seed {seed}: one draw");
        let shell = empty.extra.off_hand_id() == Some("minecraft:nautilus_shell");
        assert_eq!(shell, draw < 0.03, "seed {seed}: draw {draw}");
        assert_eq!(
            empty.extra.main_hand_id(), occupied.extra.main_hand_id(),
            "seed {seed}"
        );
        if shell {
            shells += 1;
            // The setter makes the default array (it was absent): the
            // offhand is 2.0. Every other slot reads what it read before, but
            // the saddle: without an array this route reads 2.0 (saddles
            // that `Equippable` puts on), with one 0.085, which is vanilla's
            // `DropChances.byEquipment` for a slot with no entry. The drowned
            // wears no saddle.
            let chances = empty.extra.drop_chances.as_deref().copied();
            let mut expected = [DEFAULT_DROP_CHANCE; 8];
            expected[1] = 2.0;
            assert_eq!(chances, Some(expected), "seed {seed}");
            for slot in SLOTS
                .iter()
                .filter(|s| !matches!(**s, "offhand" | "saddle"))
            {
                assert_eq!(
                    drop_chance(&empty, slot),
                    drop_chance(&occupied, slot),
                    "{slot}"
                );
            }
            assert_eq!(drop_chance(&occupied, "saddle"), 2.0);
            assert_eq!(drop_chance(&empty, "saddle"), DEFAULT_DROP_CHANCE);
            assert!(empty.saddle_item().is_none());
        } else {
            assert_eq!(empty.extra.off_hand_id(), None);
            assert_eq!(empty.extra.drop_chances, None);
        }
    }
    eprintln!("shells from seeds 0..400: {shells}");
    assert!((1..=30).contains(&shells), "{shells}");
}
