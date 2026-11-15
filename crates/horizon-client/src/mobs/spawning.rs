//! Natural spawning rules: `SpawnPlacements` placement types and each
//! species' spawn predicate (`checkAnimalSpawnRules`, `checkMonsterSpawnRules`,
//! water animals, bats...), and `finalizeSpawn` (variants, colours, babies,
//! equipment).
use super::{Mob, MobWorld, Species, farm::Climate};

/// Biome facts finalizeSpawn and the spawn rules read.
#[derive(Clone, Copy, Debug, Default)]
pub struct BiomeFacts {
    pub climate: Climate,
    /// Biome temperature at the position (strays in cold biomes, snow foxes).
    pub temperature: f32,
    pub snowy: bool,
    /// `#minecraft:spawns_snow_foxes` (`Fox.Variant.byBiome`: snow, else red).
    pub snow_fox: bool,
    pub swamp: bool,
    pub ocean: bool,
    pub mushroom: bool,
    pub desert: bool,
    pub jungle: bool,
    /// Frog variant from `spawns_warm/cold_variant_frogs` (0 temperate, 1 warm, 2 cold).
    pub frog: u8,
    /// Wolf variant index (`tame::WOLF_VARIANTS`).
    pub wolf: u8,
    /// `MoonBrightnessCheck(>= 0.9)`: the full moon.
    pub full_moon: bool,
    /// `spawns_coral_variant_zombie_nautilus` (the warm ocean).
    pub coral_nautilus: bool,
    /// `#minecraft:spawns_white_rabbits` (`Rabbit.getRandomRabbitVariant`).
    pub white_rabbit: bool,
    /// `#minecraft:spawns_gold_rabbits` (`Rabbit.getRandomRabbitVariant`).
    pub gold_rabbit: bool,
}
/// `SpawnPlacements.isSpawnPositionOk` for the entity type `entity`, of species `species`.
pub fn placement_ok(entity: &str, species: Species, w: &MobWorld, x: i32, y: i32, z: i32) -> bool {
    super::spawn_rules::is_spawn_position_ok(entity, species, w, x, y, z)
}
/// `Spider.SpiderEffectsGroupData.setRandomEffect`: Speed on two of five
/// draws, then Strength, Regeneration and Invisibility on one each.
pub fn spider_random_effect(random: &mut impl horizon_core::random::RandomSource) -> &'static str {
    match random.next_int_bound(5).unwrap_or(0) {
        0 | 1 => "minecraft:speed",
        2 => "minecraft:strength",
        3 => "minecraft:regeneration",
        _ => "minecraft:invisibility",
    }
}
/// Mob.finalizeSpawn for natural, chunk-generation and summoned mobs.
/// `DifficultyInstance`: the local difficulty at a position.
#[derive(Clone, Copy, Debug)]
pub struct LocalDifficulty {
    /// The world difficulty id, 0 peaceful to 3 hard.
    pub base: u8,
    pub effective: f32,
    /// The world difficulty is normal.
    pub normal: bool,
    /// The world difficulty is hard.
    pub hard: bool,
    /// `Difficulty.getId`: 0 peaceful to 3 hard.
    pub id: u8,
}
impl From<LocalDifficulty> for mod_api::Difficulty {
    fn from(d: LocalDifficulty) -> Self {
        Self::new(d.base, d.effective)
    }
}
impl From<mod_api::Difficulty> for LocalDifficulty {
    /// A mod's difficulty; a base past hard is hard.
    fn from(d: mod_api::Difficulty) -> Self {
        let base = d.base.min(3);
        Self {
            base,
            effective: d.effective,
            normal: base == 2,
            hard: base == 3,
            id: base,
        }
    }
}
/// `DimensionType.MOON_BRIGHTNESS_PER_PHASE`.
pub const MOON_BRIGHTNESS: [f32; 8] = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];
impl LocalDifficulty {
    /// `calculateDifficulty` from the difficulty id (0 peaceful to 3 hard),
    /// the overworld clock, the chunk's inhabited time and the moon.
    pub fn new(base: u8, total: i64, inhabited: i64, moon: f32) -> Self {
        let hard = base == 3;
        let normal = base == 2;
        if base == 0 {
            return Self {
                base,
                effective: 0.0,
                normal,
                hard,
                id: base,
            };
        }
        let global = ((total as f32 - 72000.0) / 1_440_000.0).clamp(0.0, 1.0) * 0.25;
        let mut local =
            (inhabited as f32 / 3_600_000.0).clamp(0.0, 1.0) * if hard { 1.0 } else { 0.75 };
        local += (moon * 0.25).clamp(0.0, global);
        if base == 1 {
            local *= 0.5;
        }
        Self {
            base,
            effective: f32::from(base) * (0.75 + global + local),
            normal,
            hard,
            id: base,
        }
    }
    /// `getSpecialMultiplier`.
    pub fn special(&self) -> f32 {
        mod_api::Difficulty::from(*self).special_multiplier()
    }
}
/// `Mob.populateDefaultEquipmentSlots`: sometimes a set of armour, from the
/// head down, of one random material.
<<<<<<< ours
||||||| base
fn populate_armor(m: &mut Mob, d: &LocalDifficulty) {
    if m.random.next_float() >= 0.15 * d.special() {
        return;
    }
    let mut kind = m.random.next_int_bound(3).unwrap_or(0);
    for _ in 1..=3 {
        if m.random.next_float() < 0.1087 {
            kind += 1;
        }
    }
    let partial = if d.hard { 0.1 } else { 0.25 };
    let material = [
        "leather",
        "copper",
        "golden",
        "chainmail",
        "iron",
        "diamond",
    ][kind as usize];
    for (i, piece) in ["helmet", "chestplate", "leggings", "boots"]
        .into_iter()
        .enumerate()
    {
        if i > 0 && m.random.next_float() < partial {
            break;
        }
        let slot = match i {
            0 => &mut m.extra.head,
            1 => &mut m.extra.chest,
            2 => &mut m.extra.legs,
            _ => &mut m.extra.feet,
        };
        if slot.is_none() {
            *slot = Some(format!("minecraft:{material}_{piece}").into());
        }
    }
}
/// `Spider.SpiderEffectsGroupData.setRandomEffect`: Speed on two of five
/// draws, then Strength, Regeneration and Invisibility on one each.
pub fn spider_random_effect(random: &mut impl RandomSource) -> &'static str {
    match random.next_int_bound(5).unwrap_or(0) {
        0 | 1 => "minecraft:speed",
        2 => "minecraft:strength",
        3 => "minecraft:regeneration",
        _ => "minecraft:invisibility",
    }
}
/// `Spider.finalizeSpawn`: everything it adds to `Mob.finalizeSpawn` is the
/// skeleton jockey (a spawn in the level, `Game::spawn_jockeys`) and, on Hard,
/// one of four effects for good (`MobEffectInstance(effect, -1)`) for
/// `0.1 * getSpecialMultiplier()` of spiders, for every spawn reason. Chunk
/// generation rolls the same through [`super::finalize`]'s `Group::Spider`;
/// the draw here is the mob's own random, as the rest of this finalizer's is.
pub fn finalize(m: &mut Mob, biome: &BiomeFacts, difficulty: &LocalDifficulty) {
    use Species as S;
    let climate_index = match biome.climate {
        Climate::Temperate => 0,
        Climate::Warm => 1,
        Climate::Cold => 2,
    };
    // `Registry.getRandom` of the `*_sound_variant` registry (pig, cow,
    // chicken, cat and wolf), drawn by their finalizeSpawn.
    let sounds = super::finalize::sound_variants(m.species()).len();
    if sounds > 0 {
        m.extra.sound_variant = Some(m.random.next_int_bound(sounds as i32).unwrap_or(0) as u8);
    }
    // Mob.finalizeSpawn, reached through `super` by every mob but the cave
    // spider (CaveSpider.finalizeSpawn returns without calling it).
    if m.species() != S::CaveSpider {
        let mut random = m.random.clone();
        super::finalize::mob_spawn(m, &mut random);
        m.random = random;
    }
    match m.species() {
        // Warden.finalizeSpawn: a minute before it digs back down.
        S::Warden => super::warden::finalize(m, false, &mut Vec::new()),
        S::Pig | S::Cow | S::Chicken => m.extra.variant = climate_index,
        S::Sheep => m.extra.color = super::farm::sheep_color(biome.climate, &mut m.random),
        S::Mooshroom => m.extra.variant = 0,
        // Turtle.finalizeSpawn: home is where it spawned.
        S::Turtle => {
            m.extra.home = Some([
                m.position.x.floor() as i32,
                m.position.y.floor() as i32,
                m.position.z.floor() as i32,
            ])
        }
        S::Frog => m.extra.variant = biome.frog,
        S::Wolf => m.extra.variant = biome.wolf,
        S::Cat => m.extra.variant = super::tame::cat_variant(m, biome.full_moon),
        S::Zombie | S::Husk | S::Drowned | S::ZombieVillager => {
            // Zombie.finalizeSpawn: pick up loot 55% of the time (scaled by
            // difficulty), 5% babies (normal difficulty).
            m.extra.pick_up_loot = m.random.next_float() < 0.55 * difficulty.special();
            if m.random.next_float() < 0.05 {
                m.extra.age = i32::MIN / 2;
            }
            // setCanBreakDoors(nextFloat() < special * 0.1), before the gear.
            let can_break_doors = m.random.next_float() < difficulty.special() * 0.1;
            super::doors::set_can_break_doors(m, can_break_doors);
            if m.species() != S::Drowned {
                populate_armor(m, difficulty);
            }
            let weapon = if difficulty.hard { 0.05 } else { 0.01 };
            if m.species() != S::Drowned && m.random.next_float() < weapon {
                let r = m.random.next_int_bound(6).unwrap_or(0);
                m.extra.set_main_hand(Some(
                    match r {
                        0 => "minecraft:iron_sword",
                        1 => "minecraft:iron_spear",
                        _ => "minecraft:iron_shovel",
                    }
                    .into(),
                ));
            }
            if m.species() == S::Drowned && m.random.next_float() > 0.9 {
                m.extra.set_main_hand(Some(
                    if m.random.next_int_bound(16).unwrap_or(1) < 10 {
                        "minecraft:trident"
                    } else {
                        "minecraft:fishing_rod"
                    }
                    .into(),
                ));
            }
            if m.species() == S::ZombieVillager {
                // `finalizeVillagerType` only picks the type; the profession is
                // the constructor's draw (`defineSynchedData`).
                m.extra.variant = villager_type(biome);
            }
        }
        // ZombifiedPiglin.populateDefaultEquipmentSlots: a golden sword, or
        // one time in 20 a golden spear.
        S::ZombifiedPiglin => {
            // Zombie.finalizeSpawn's setCanBreakDoors.
            let can_break_doors = m.random.next_float() < difficulty.special() * 0.1;
            super::doors::set_can_break_doors(m, can_break_doors);
            let item = super::piglin::default_equipment(&mut m.random);
            m.extra.set_main_hand(Some(item.into()));
        }
        // AbstractSkeleton.populateDefaultEquipmentSlots: armour, then a bow.
        S::Skeleton | S::Stray | S::Bogged | S::Parched => {
            populate_armor(m, difficulty);
            m.extra.set_main_hand(Some("minecraft:bow".into()));
            m.extra.pick_up_loot = m.random.next_float() < 0.55 * difficulty.special();
        }
        // WitherSkeleton.populateDefaultEquipmentSlots: a stone sword only;
        // WitherSkeleton.finalizeSpawn then sets the attack damage base to 4.
        S::WitherSkeleton => {
            m.extra.set_main_hand(Some("minecraft:stone_sword".into()));
            m.extra.pick_up_loot = m.random.next_float() < 0.55 * difficulty.special();
            super::finalize::set_base(m, super::finalize::Attr::AttackDamage, 4.0);
        }
        S::Pillager => {
            m.extra.set_main_hand(Some("minecraft:crossbow".into()));
            m.extra.pick_up_loot = true;
        }
        S::Horse | S::Donkey | S::Mule | S::SkeletonHorse | S::ZombieHorse => {
            super::equine::finalize(m)
        }
        S::Llama | S::TraderLlama => super::llama::finalize(m),
        // Camel.finalizeSpawn: standing, with fixed attributes.
        S::Camel | S::CamelHusk => super::camel::stand_up_instantly(m),
        S::Nautilus | S::ZombieNautilus => super::nautilus::finalize(m, biome.coral_nautilus),
        S::SulfurCube => super::sulfur_cube::finalize(m),
        // Piglin.createSpawnWeapon: half crossbows, half golden swords, and
        // each piece of golden armour one time in ten.
        S::Piglin if !m.is_baby() => {
            m.extra.pick_up_loot = true;
            for i in 0..4 {
                if m.random.next_float() < 0.1 {
                    let (slot, piece) = match i {
                        0 => (&mut m.extra.head, "helmet"),
                        1 => (&mut m.extra.chest, "chestplate"),
                        2 => (&mut m.extra.legs, "leggings"),
                        _ => (&mut m.extra.feet, "boots"),
                    };
                    *slot = Some(format!("minecraft:golden_{piece}").into());
                }
            }
            let crossbow = m.random.next_float() < 0.5;
            m.extra.set_main_hand(Some(
                if crossbow {
                    "minecraft:crossbow"
                } else {
                    "minecraft:golden_sword"
                }
                .into(),
            ));
        }
        S::PiglinBrute => {
            m.extra.set_main_hand(Some("minecraft:golden_axe".into()));
            m.extra.pick_up_loot = true;
        }
        S::Illusioner => super::illusioner::finalize(m),
        // Spider.finalizeSpawn: on Hard, a tenth carry a permanent effect.
        S::Spider => {
            if difficulty.hard && m.random.next_float() < 0.1 * difficulty.special() {
                let effect = spider_random_effect(&mut m.random);
                m.add_effect(effect, -1, 0);
            }
        }
        S::Vindicator => m.extra.set_main_hand(Some("minecraft:iron_axe".into())),
        S::Villager => {
            m.extra.variant = villager_type(biome);
            m.extra.pick_up_loot = true;
        }
        // Fox.finalizeSpawn: the variant by biome, then the mouth's item.
        S::Fox => {
            m.extra.variant = u8::from(biome.snow_fox);
            m.extra.pick_up_loot = true;
            super::fox::populate_default_equipment(m);
        }
        S::Panda | S::Dolphin => m.extra.pick_up_loot = true,
        S::Goat => super::finalize::goat_spawn(m),
        S::TropicalFish => super::aquatic::finalize(m),
        // Salmon.finalizeSpawn: a weighted size (small 30, medium 50, large 15).
        S::Salmon => {
            let r = m.random.next_int_bound(95).unwrap_or(0);
            m.extra.variant = if r < 30 {
                0
            } else if r < 80 {
                1
            } else {
                2
            };
        }
        S::Slime | S::MagmaCube => super::cube::finalize(m, difficulty.special()),
        _ => {}
    }
    // Zombie.finalizeSpawn ends with handleAttributes: the reinforcement
    // chance (and a leader's bonus) that `Zombie.hurtServer` reads.
    if matches!(
        m.species(),
        S::Zombie | S::Husk | S::Drowned | S::ZombieVillager | S::ZombifiedPiglin
    ) {
        super::finalize::zombie_attributes(
            m,
            difficulty.special(),
            super::finalize::Reason::Natural,
        );
    }
    // Drowned.finalizeSpawn, once Zombie.finalizeSpawn is done: a nautilus
    // shell in an empty offhand, as a guaranteed drop. Java draws it from the
    // level's random; this route stands in the mob's own, as above.
    if m.species() == S::Drowned && m.extra.off_hand.is_none() && m.random.next_float() < 0.03 {
        super::finalize::set_item(m, super::finalize::OFFHAND, "minecraft:nautilus_shell");
        super::finalize::set_drop_chance(m, super::finalize::OFFHAND, 2.0);
    }
}
=======
fn populate_armor(m: &mut Mob, d: &LocalDifficulty, r: &mut impl RandomSource) {
    if r.next_float() >= 0.15 * d.special() {
        return;
    }
    let mut kind = r.next_int_bound(3).unwrap_or(0);
    for _ in 1..=3 {
        if r.next_float() < 0.1087 {
            kind += 1;
        }
    }
    let partial = if d.hard { 0.1 } else { 0.25 };
    let material = [
        "leather",
        "copper",
        "golden",
        "chainmail",
        "iron",
        "diamond",
    ][kind as usize];
    for (i, piece) in ["helmet", "chestplate", "leggings", "boots"]
        .into_iter()
        .enumerate()
    {
        if i > 0 && r.next_float() < partial {
            break;
        }
        let slot = match i {
            0 => &mut m.extra.head,
            1 => &mut m.extra.chest,
            2 => &mut m.extra.legs,
            _ => &mut m.extra.feet,
        };
        if slot.is_none() {
            *slot = Some(format!("minecraft:{material}_{piece}").into());
        }
    }
}
/// `Spider.SpiderEffectsGroupData.setRandomEffect`: Speed on two of five
/// draws, then Strength, Regeneration and Invisibility on one each.
pub fn spider_random_effect(random: &mut impl RandomSource) -> &'static str {
    match random.next_int_bound(5).unwrap_or(0) {
        0 | 1 => "minecraft:speed",
        2 => "minecraft:strength",
        3 => "minecraft:regeneration",
        _ => "minecraft:invisibility",
    }
}
/// `finalizeSpawn` of a fresh mob. `random` is the level's
/// (`ServerLevelAccessor.getRandom()`), which `Mob.finalizeSpawn` and most
/// overrides draw from; `Zombie.handleAttributes`, `Piglin.createSpawnWeapon`
/// and the salmon's size draw from the mob's own.
pub fn finalize(
    m: &mut Mob,
    biome: &BiomeFacts,
    difficulty: &LocalDifficulty,
    random: &mut impl RandomSource,
) {
    finalize_with(m, biome, difficulty, random, |_, _| {});
}
/// [`finalize`] with every roll on the mob's own random, in one stream.
pub fn finalize_on_own_random(m: &mut Mob, biome: &BiomeFacts, difficulty: &LocalDifficulty) {
    let mut random = m.random.clone();
    // The stream changes hands around the draws `finalize` makes on the mob's own.
    finalize_with(m, biome, difficulty, &mut random, |m, r| {
        std::mem::swap(&mut m.random, r)
    });
    m.random = random;
}
/// `own` runs before and after each group of draws on the mob's own random.
fn finalize_with<R: RandomSource>(
    m: &mut Mob,
    biome: &BiomeFacts,
    difficulty: &LocalDifficulty,
    random: &mut R,
    own: impl Fn(&mut Mob, &mut R),
) {
    use Species as S;
    let climate_index = match biome.climate {
        Climate::Temperate => 0,
        Climate::Warm => 1,
        Climate::Cold => 2,
    };
    // `Registry.getRandom` of the `*_sound_variant` registry.
    fn sound(m: &mut Mob, random: &mut impl RandomSource) {
        let sounds = super::finalize::sound_variants(m.species()).len() as i32;
        m.extra.sound_variant = Some(random.next_int_bound(sounds).unwrap_or(0) as u8);
    }
    // The overrides that draw before they call `super.finalizeSpawn`.
    match m.species() {
        S::Pig | S::Cow | S::Chicken | S::Wolf => {
            // VariantUtils.selectVariantToSpawn: `Util.getRandomSafe` over
            // the variants of the highest priority, the biome's one.
            let _ = random.next_int_bound(1);
            m.extra.variant = if m.species() == S::Wolf {
                biome.wolf
            } else {
                climate_index
            };
            sound(m, random);
        }
        S::Sheep => m.extra.color = super::farm::sheep_color(biome.climate, random),
        S::Horse | S::Donkey | S::Mule | S::SkeletonHorse | S::ZombieHorse => {
            super::equine::finalize(m, random)
        }
        S::Llama | S::TraderLlama => super::llama::finalize(m, random),
        S::Nautilus | S::ZombieNautilus => {
            super::nautilus::finalize(m, biome.coral_nautilus, random)
        }
        S::Goat => super::finalize::goat_spawn(m, random),
        // Fox.finalizeSpawn: the variant by biome, then the mouth's item.
        S::Fox => {
            m.extra.variant = u8::from(biome.snow_fox);
            m.extra.pick_up_loot = true;
            super::fox::populate_default_equipment(m, random);
        }
        // Piglin.createSpawnWeapon: half crossbows, half golden swords, and
        // each piece of golden armour one time in ten.
        S::Piglin if !m.is_baby() => {
            m.extra.pick_up_loot = true;
            for i in 0..4 {
                if random.next_float() < 0.1 {
                    let (slot, piece) = match i {
                        0 => (&mut m.extra.head, "helmet"),
                        1 => (&mut m.extra.chest, "chestplate"),
                        2 => (&mut m.extra.legs, "leggings"),
                        _ => (&mut m.extra.feet, "boots"),
                    };
                    *slot = Some(format!("minecraft:golden_{piece}").into());
                }
            }
            own(m, random);
            let crossbow = m.random.next_float() < 0.5;
            own(m, random);
            m.extra.set_main_hand(Some(
                if crossbow {
                    "minecraft:crossbow"
                } else {
                    "minecraft:golden_sword"
                }
                .into(),
            ));
        }
        _ => {}
    }
    // Mob.finalizeSpawn, reached through `super` by every mob but the cave
    // spider (CaveSpider.finalizeSpawn returns without calling it).
    if m.species() != S::CaveSpider {
        super::finalize::mob_spawn(m, random);
    }
    match m.species() {
        // Warden.finalizeSpawn: a minute before it digs back down.
        S::Warden => super::warden::finalize(m, false, &mut Vec::new()),
        S::Mooshroom => m.extra.variant = 0,
        // Turtle.finalizeSpawn: home is where it spawned.
        S::Turtle => {
            m.extra.home = Some([
                m.position.x.floor() as i32,
                m.position.y.floor() as i32,
                m.position.z.floor() as i32,
            ])
        }
        S::Frog => m.extra.variant = biome.frog,
        S::Cat => {
            m.extra.variant = super::tame::cat_variant(biome.full_moon, random);
            sound(m, random);
        }
        S::Zombie | S::Husk | S::Drowned | S::ZombieVillager => {
            // Zombie.finalizeSpawn: pick up loot 55% of the time (scaled by
            // difficulty), 5% babies (normal difficulty).
            m.extra.pick_up_loot = random.next_float() < 0.55 * difficulty.special();
            if random.next_float() < 0.05 {
                m.extra.age = i32::MIN / 2;
            }
            // setCanBreakDoors(nextFloat() < special * 0.1), before the gear.
            let can_break_doors = random.next_float() < difficulty.special() * 0.1;
            super::doors::set_can_break_doors(m, can_break_doors);
            if m.species() != S::Drowned {
                populate_armor(m, difficulty, random);
            }
            let weapon = if difficulty.hard { 0.05 } else { 0.01 };
            if m.species() != S::Drowned && random.next_float() < weapon {
                let r = random.next_int_bound(6).unwrap_or(0);
                m.extra.set_main_hand(Some(
                    match r {
                        0 => "minecraft:iron_sword",
                        1 => "minecraft:iron_spear",
                        _ => "minecraft:iron_shovel",
                    }
                    .into(),
                ));
            }
            if m.species() == S::Drowned && random.next_float() > 0.9 {
                m.extra.set_main_hand(Some(
                    if random.next_int_bound(16).unwrap_or(1) < 10 {
                        "minecraft:trident"
                    } else {
                        "minecraft:fishing_rod"
                    }
                    .into(),
                ));
            }
            if m.species() == S::ZombieVillager {
                // `finalizeVillagerType` only picks the type; the profession is
                // the constructor's draw (`defineSynchedData`).
                m.extra.variant = villager_type(biome);
            }
        }
        // ZombifiedPiglin.populateDefaultEquipmentSlots: a golden sword, or
        // one time in 20 a golden spear.
        S::ZombifiedPiglin => {
            // Zombie.finalizeSpawn's setCanBreakDoors.
            let can_break_doors = random.next_float() < difficulty.special() * 0.1;
            super::doors::set_can_break_doors(m, can_break_doors);
            let item = super::piglin::default_equipment(random);
            m.extra.set_main_hand(Some(item.into()));
        }
        // AbstractSkeleton.populateDefaultEquipmentSlots: armour, then a bow.
        S::Skeleton | S::Stray | S::Bogged | S::Parched => {
            populate_armor(m, difficulty, random);
            m.extra.set_main_hand(Some("minecraft:bow".into()));
            m.extra.pick_up_loot = random.next_float() < 0.55 * difficulty.special();
        }
        // WitherSkeleton.populateDefaultEquipmentSlots: a stone sword only;
        // WitherSkeleton.finalizeSpawn then sets the attack damage base to 4.
        S::WitherSkeleton => {
            m.extra.set_main_hand(Some("minecraft:stone_sword".into()));
            m.extra.pick_up_loot = random.next_float() < 0.55 * difficulty.special();
            super::finalize::set_base(m, super::finalize::Attr::AttackDamage, 4.0);
        }
        S::Pillager => {
            m.extra.set_main_hand(Some("minecraft:crossbow".into()));
            m.extra.pick_up_loot = true;
        }
        // Camel.finalizeSpawn: standing, with fixed attributes.
        S::Camel | S::CamelHusk => super::camel::stand_up_instantly(m),
        S::SulfurCube => super::sulfur_cube::finalize(m),
        S::PiglinBrute => {
            m.extra.set_main_hand(Some("minecraft:golden_axe".into()));
            m.extra.pick_up_loot = true;
        }
        S::Illusioner => super::illusioner::finalize(m),
        // Spider.finalizeSpawn: on Hard, a tenth carry a permanent effect,
        // for every spawn reason. Vanilla rolls the skeleton jockey first;
        // here that roll follows in `Game::spawn_jockeys`.
        S::Spider => {
            if difficulty.hard && random.next_float() < 0.1 * difficulty.special() {
                let effect = spider_random_effect(random);
                m.add_effect(effect, -1, 0);
            }
        }
        S::Vindicator => m.extra.set_main_hand(Some("minecraft:iron_axe".into())),
        S::Villager => {
            m.extra.variant = villager_type(biome);
            m.extra.pick_up_loot = true;
        }
        S::Panda | S::Dolphin => m.extra.pick_up_loot = true,
        S::TropicalFish => super::aquatic::finalize(m, random),
        // Salmon.finalizeSpawn: a weighted size (small 30, medium 50, large 15).
        S::Salmon => {
            own(m, random);
            let r = m.random.next_int_bound(95).unwrap_or(0);
            own(m, random);
            m.extra.variant = if r < 30 {
                0
            } else if r < 80 {
                1
            } else {
                2
            };
        }
        S::Slime | S::MagmaCube => super::cube::finalize(m, difficulty.special(), random),
        _ => {}
    }
    // Zombie.finalizeSpawn ends with handleAttributes: the reinforcement
    // chance (and a leader's bonus) that `Zombie.hurtServer` reads.
    if matches!(
        m.species(),
        S::Zombie | S::Husk | S::Drowned | S::ZombieVillager | S::ZombifiedPiglin
    ) {
        own(m, random);
        super::finalize::zombie_attributes(
            m,
            difficulty.special(),
            super::finalize::Reason::Natural,
        );
        own(m, random);
    }
    // Drowned.finalizeSpawn, once Zombie.finalizeSpawn is done: a nautilus
    // shell in an empty offhand, as a guaranteed drop.
    if m.species() == S::Drowned && m.extra.off_hand.is_none() && random.next_float() < 0.03 {
        super::finalize::set_item(m, super::finalize::OFFHAND, "minecraft:nautilus_shell");
        super::finalize::set_drop_chance(m, super::finalize::OFFHAND, 2.0);
    }
}
>>>>>>> theirs
/// A baby from `Animal.spawnChildFromBreeding`: `getBreedOffspring` creates
/// it without `finalizeSpawn`, so it never ran `Mob.finalizeSpawn`. Undoes
/// that part of finalizeSpawn: no `random_spawn_bonus`, right-handed.
pub fn bred_offspring(m: &mut Mob) {
    use super::finalize::{Attr, remove_modifier};
    remove_modifier(m, Attr::FollowRange, "minecraft:random_spawn_bonus");
    m.extra.left_handed = false;
}
/// VillagerType.byBiome
pub fn villager_type(b: &BiomeFacts) -> u8 {
    if b.desert {
        1
    } else if b.jungle {
        2
    } else if b.snowy {
        4
    } else if b.swamp {
        5
    } else if b.climate == Climate::Warm {
        3
    } else if b.climate == Climate::Cold && b.temperature < 0.4 {
        6
    } else {
        0
    }
}
#[cfg(test)]
pub(in crate::mobs) mod tests {
    use super::*;
    use crate::mobs::finalize::{Attr, has_modifier, value};

    fn types() -> Option<crate::mobs::MobTypes> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/horizon-client")
            .join(horizon_core::VANILLA_VERSION);
        if !root.join("items.json").exists() {
            return None;
        }
        Some(crate::mobs::MobTypes::load(&root).unwrap())
    }

    struct Level {
        registries: crate::mobs::finalize::SpawnRegistries,
        enchantments: crate::enchant::Enchantments,
        items: crate::items::Items,
    }
    impl crate::mobs::finalize::FinalizeLevel for Level {
        fn biome(&self, _: [i32; 3]) -> String {
            "minecraft:plains".into()
        }
        fn biome_tag(&self, _: &str, _: [i32; 3]) -> bool {
            false
        }
        fn structures_with_piece_at(&self, _: [i32; 3]) -> Vec<String> {
            Vec::new()
        }
        fn moon_phase(&self, _: [i32; 3]) -> usize {
            0
        }
        fn spawning_monsters(&self) -> bool {
            true
        }
        fn no_collision(&self, _: horizon_core::geometry::Aabb) -> bool {
            true
        }
        fn free_chicken(&self, _: horizon_core::geometry::Aabb) -> Option<u32> {
            None
        }
        fn ride_chicken(&mut self, _: u32, _: u32) {}
        fn game_time(&self) -> i64 {
            0
        }
        fn halloween(&self) -> bool {
            false
        }
        fn create(&mut self, _: &str, _: crate::mobs::spawn_rules::Reason) -> Option<Mob> {
            None
        }
        fn spawn_data(&self) -> crate::mobs::finalize::SpawnData<'_> {
            crate::mobs::finalize::SpawnData {
                registries: &self.registries,
                enchantments: &self.enchantments,
                items: &self.items,
                camel_husk_size: (1.0, 1.0),
            }
        }
    }
    pub(in crate::mobs) fn run_finalize(m: &mut Mob, difficulty: LocalDifficulty) {
        static LEVEL: std::sync::OnceLock<std::sync::Mutex<Level>> = std::sync::OnceLock::new();
        let mut level = LEVEL
            .get_or_init(|| {
                let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../.local/horizon-client")
                    .join(horizon_core::VANILLA_VERSION);
                let defaults = horizon_world::item_defaults::read(&root).unwrap();
                std::sync::Mutex::new(Level {
                    registries: crate::mobs::finalize::SpawnRegistries::load(&root),
                    enchantments: crate::enchant::Enchantments::load(&root, &defaults).unwrap(),
                    items: crate::items::Items::tags_only(
                        &root,
                        &horizon_world::loot::PackOverlay::NONE,
                        &defaults,
                    )
                    .unwrap(),
                })
            })
            .lock()
            .unwrap();
        let mut random = m.random.clone();
        let mut cx = crate::mobs::finalize::Ctx {
            level: &mut *level,
            random: &mut random,
            difficulty,
            created: Vec::new(),
            jockey_rider: None,
        };
        cx.finalize(m, crate::mobs::spawn_rules::Reason::Natural, None);
    }

    fn spawn(types: &crate::mobs::MobTypes, kind: &str, id: u32) -> Mob {
        spawn_with(types, kind, id, &LocalDifficulty::new(3, 0, 0, 1.0))
    }
    fn spawn_with(
        types: &crate::mobs::MobTypes,
        kind: &str,
        id: u32,
        difficulty: &LocalDifficulty,
    ) -> Mob {
        let kind = std::sync::Arc::new(types.types.get(kind).unwrap().clone());
        let mut m = Mob::new(
            id,
            kind,
            horizon_core::geometry::Vec3::new(0.5, 64.0, 0.5),
            0.0,
        );
<<<<<<< ours
        run_finalize(&mut m, *difficulty);
||||||| base
        finalize(&mut m, &BiomeFacts::default(), difficulty);
=======
        let mut level = horizon_core::random::LegacyRandom::new(i64::from(id));
        finalize(&mut m, &BiomeFacts::default(), difficulty, &mut level);
>>>>>>> theirs
        m
    }
    /// The four effects `Spider.SpiderEffectsGroupData.setRandomEffect` picks.
    const SPIDER_EFFECTS: [&str; 4] = [
        "minecraft:speed",
        "minecraft:strength",
        "minecraft:regeneration",
        "minecraft:invisibility",
    ];
    /// `Spider.finalizeSpawn`: on Hard, `0.1 * getSpecialMultiplier()` of
    /// spiders get one of those four for good (`MobEffectInstance(effect, -1)`).
    /// `/summon`, eggs, spawners and natural spawns all run it; Normal and
    /// CaveSpider's own `finalizeSpawn` never roll.
    #[test]
    fn runtime_spiders_roll_a_permanent_effect_on_hard() {
        let Some(types) = types() else { return };
        // Long played-in time: `getSpecialMultiplier` at its 1.0 maximum.
        let hard = LocalDifficulty::new(3, 1_512_000, 3_600_000, 1.0);
        assert_eq!(hard.special(), 1.0);
        let mut carried = 0;
        let mut kinds = [0usize; 4];
        for id in 1..=400 {
            let m = spawn_with(&types, "minecraft:spider", id, &hard);
            let effects = &m.extra.effects;
            assert!(effects.len() <= 1, "{id}: {effects:?}");
            for stored in effects {
                let (effect, duration, amplifier) = (stored.id, stored.duration, stored.amplifier);
                let i = SPIDER_EFFECTS
                    .iter()
                    .position(|e| *e == effect)
                    .unwrap_or_else(|| panic!("{id}: {effect}"));
                assert_eq!((duration, amplifier), (-1, 0), "{id}: {effect}");
                kinds[i] += 1;
                carried += 1;
            }
        }
        // 10 percent of 400 is 40.
        assert!((25..=60).contains(&carried), "{carried} of 400 spiders");
        // Vanilla's `nextInt(5)`: Speed on 0 or 1, then one each.
        assert!(kinds.iter().all(|n| *n > 0), "{kinds:?}");
        assert!(
            kinds[0] >= kinds[1] && kinds[0] >= kinds[2] && kinds[0] >= kinds[3],
            "{kinds:?}"
        );
        // Normal: `level.getDifficulty() == Difficulty.HARD` is false.
        let normal = LocalDifficulty::new(2, 1_512_000, 3_600_000, 1.0);
        for id in 1..=400 {
            let m = spawn_with(&types, "minecraft:spider", id, &normal);
            assert!(
                m.extra.effects.is_empty(),
                "normal {id}: {:?}",
                m.extra.effects
            );
        }
        // CaveSpider.finalizeSpawn returns the group data without rolling.
        for id in 1..=400 {
            let m = spawn_with(&types, "minecraft:cave_spider", id, &hard);
            assert!(
                m.extra.effects.is_empty(),
                "cave spider {id}: {:?}",
                m.extra.effects
            );
        }
    }
    /// #2783: `/summon`, eggs, spawners and natural spawns run
    /// `WitherSkeleton.finalizeSpawn`: attack damage base 4.0, not the
    /// default 2.0.
    #[test]
    fn runtime_wither_skeletons_hit_for_four() {
        let Some(types) = types() else { return };
        let w = spawn(&types, "minecraft:wither_skeleton", 1);
        assert_eq!(value(&w, Attr::AttackDamage), 4.0);
        assert_eq!(w.attrs.attack_damage, 4.0);
    }
    /// #2783: runtime spawns run `Mob.finalizeSpawn`: the follow range
    /// bonus and 5 percent left-handed mobs.
    #[test]
    fn runtime_spawns_run_the_mob_finalize_spawn() {
        let Some(types) = types() else { return };
        let mut left = 0;
        for id in 1..=400 {
            let m = spawn(&types, "minecraft:skeleton", id);
            assert!(has_modifier(
                &m,
                Attr::FollowRange,
                "minecraft:random_spawn_bonus"
            ));
            left += usize::from(m.extra.left_handed);
        }
        // 5 percent of 400 is 20.
        assert!((5..=40).contains(&left), "{left} of 400 left-handed");
        // A bred baby never ran it.
        let mut cow = spawn(&types, "minecraft:cow", 1);
        cow.extra.left_handed = true;
        bred_offspring(&mut cow);
        assert!(!has_modifier(
            &cow,
            Attr::FollowRange,
            "minecraft:random_spawn_bonus"
        ));
        assert!(!cow.extra.left_handed);
        assert_eq!(cow.attrs.follow_range, value(&cow, Attr::FollowRange));
        // CaveSpider.finalizeSpawn does not call super.
        let c = spawn(&types, "minecraft:cave_spider", 1);
        assert!(!has_modifier(
            &c,
            Attr::FollowRange,
            "minecraft:random_spawn_bonus"
        ));
    }
    /// #11651: unfinalized spawns (EntityType.create) skip finalizeSpawn's rolls.
    #[test]
    fn unfinalized_spawns_skip_finalize_rolls() {
        let Some(types) = types() else { return };
        let kind = std::sync::Arc::new(types.types.get("minecraft:zombie").unwrap().clone());
        let m = Mob::new(
            1,
            kind,
            horizon_core::geometry::Vec3::new(0.5, 64.0, 0.5),
            0.0,
        );
        assert!(!has_modifier(
            &m,
            Attr::FollowRange,
            "minecraft:random_spawn_bonus"
        ));
        assert!(!m.extra.left_handed);
        assert!(m.equipment_stacks()[5].is_none());
        assert!(m.equipment_stacks()[0].is_none());
    }
    /// `Zombie` / `AbstractSkeleton` halloween garb: 25% of empty heads, never
    /// on other mobs, and not when it is not Halloween.
    #[test]
    fn halloween_pumpkins_only_undead_on_halloween() {
        let Some(types) = types() else { return };
        let mut pumpkins = 0;
        let mut lanterns = 0;
        for id in 1..=400 {
            let mut m = spawn(&types, "minecraft:skeleton", id);
            m.set_equipment_stack(5, None);
            let mut random = m.random.clone();
            crate::mobs::finalize::halloween_head(&mut m, &mut random, true);
            match m.equipment_stacks()[5]
                .as_ref()
                .map(|stack| stack.item.as_ref())
            {
                Some("minecraft:jack_o_lantern") => {
                    pumpkins += 1;
                    lanterns += 1;
                    assert_eq!(crate::mobs::finalize::stored_drop_chance(&m, 5), 0.0);
                }
                Some("minecraft:carved_pumpkin") => {
                    pumpkins += 1;
                    assert_eq!(crate::mobs::finalize::stored_drop_chance(&m, 5), 0.0);
                }
                Some(other) => panic!("unexpected head {other}"),
                None => {}
            }
        }
        assert!(
            (70..=130).contains(&pumpkins),
            "{pumpkins} of 400 skeletons"
        );
        assert!(lanterns < pumpkins / 2, "{lanterns} lanterns of {pumpkins}");
        let mut m = spawn(&types, "minecraft:skeleton", 1);
        m.set_equipment_stack(5, None);
        let mut random = m.random.clone();
        crate::mobs::finalize::halloween_head(&mut m, &mut random, false);
        assert!(m.equipment_stacks()[5].is_none());
        let mut cow = spawn(&types, "minecraft:cow", 1);
        cow.set_equipment_stack(5, None);
        let mut random = cow.random.clone();
        crate::mobs::finalize::halloween_head(&mut cow, &mut random, true);
        assert!(cow.equipment_stacks()[5].is_none());
    }
}
