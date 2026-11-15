use super::*;
use crate::testing::FakeHost;
use crate::{ItemResource, Tag};
use std::{cell::RefCell, rc::Rc};

/// A mod's own resource, which the game knows nothing of.
#[derive(Clone, Debug, PartialEq)]
struct Chemical(&'static str);

/// Tanks of up to 100 of a chemical each, shared with the test to read back.
#[derive(Clone)]
struct Tanks(Rc<RefCell<Vec<Option<(Chemical, u32)>>>>);

impl Tanks {
    fn new(held: Vec<Option<(Chemical, u32)>>) -> Self {
        Self(Rc::new(RefCell::new(held)))
    }
    fn held(&self) -> Vec<Option<(Chemical, u32)>> {
        self.0.borrow().clone()
    }
}

impl ResourceHandler<Chemical> for Tanks {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        self.0.borrow().len()
    }
    fn resource(&self, _: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        self.0.borrow().get(index)?.as_ref().map(|(c, _)| c.clone())
    }
    fn amount(&self, _: &mut dyn ServerHost, index: usize) -> u32 {
        self.0
            .borrow()
            .get(index)
            .cloned()
            .flatten()
            .map_or(0, |(_, a)| a)
    }
    fn capacity(&self, _: &mut dyn ServerHost, _: usize, _: &Chemical) -> u32 {
        100
    }
    fn insert(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        let mut tanks = self.0.borrow_mut();
        let Some(tank) = tanks.get_mut(index) else {
            return 0;
        };
        let held = match tank {
            Some((other, _)) if other != chemical => return 0,
            Some((_, held)) => *held,
            None => 0,
        };
        let went = amount.min(100 - held);
        if went > 0 {
            *tank = Some((chemical.clone(), held + went));
        }
        went
    }
    fn extract(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        let mut tanks = self.0.borrow_mut();
        let Some(tank) = tanks.get_mut(index) else {
            return 0;
        };
        let Some((held_chemical, held)) = tank.clone().filter(|(c, _)| c == chemical) else {
            return 0;
        };
        let came = amount.min(held);
        *tank = (held > came).then(|| (held_chemical, held - came));
        came
    }
}

const HYDROGEN: Chemical = Chemical("test:hydrogen");

#[test]
fn a_combined_handler_numbers_both_and_fills_the_second_when_the_first_is_full() {
    let full = Some((HYDROGEN, 100));
    let first = Tanks::new(vec![full.clone(), full.clone()]);
    let second = Tanks::new(vec![None, None]);
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    let combined = CombinedResourceHandler::new(
        host,
        vec![
            Box::new(first.clone()) as Box<dyn ResourceHandler<Chemical>>,
            Box::new(second.clone()),
        ],
    );
    assert_eq!(combined.size(host), 4);
    assert_eq!(combined.amount(host, 1), 100);
    assert_eq!(combined.resource(host, 2), None);
    let went = host
        .transaction(|host, tx| Ok::<_, ()>(combined.insert_resource(host, tx, &HYDROGEN, 150)));
    assert_eq!(went, Ok(150));
    assert_eq!(first.held(), vec![full.clone(), full.clone()]);
    assert_eq!(second.held(), vec![full, Some((HYDROGEN, 50))]);
    // Its size is the one it was built with, whatever its handlers grow to.
    second.0.borrow_mut().push(None);
    assert_eq!(combined.size(host), 4);
    assert_eq!(combined.amount(host, 4), 0);
}

#[test]
fn a_ranged_handler_numbers_its_range_from_0_and_refuses_a_range_the_handler_lacks() {
    let tanks = Tanks::new(vec![None, Some((HYDROGEN, 10)), None]);
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    let boxed = || Box::new(tanks.clone()) as Box<dyn ResourceHandler<Chemical>>;
    let ranged = RangedResourceHandler::of(host, boxed(), 1, 3).unwrap();
    assert_eq!(ranged.size(host), 2);
    assert_eq!(ranged.amount(host, 0), 10);
    let went = host.transaction(|host, tx| Ok::<_, ()>(ranged.insert(host, tx, 1, &HYDROGEN, 30)));
    assert_eq!(went, Ok(30));
    assert_eq!(
        tanks.held(),
        vec![None, Some((HYDROGEN, 10)), Some((HYDROGEN, 30))]
    );
    assert_eq!(ranged.amount(host, 2), 0);
    assert!(RangedResourceHandler::of(host, boxed(), 2, 2).is_none());
    assert!(RangedResourceHandler::of(host, boxed(), 0, 4).is_none());
    let single = RangedResourceHandler::of_single_index(host, boxed(), 2).unwrap();
    assert_eq!(single.size(host), 1);
    assert!(RangedResourceHandler::of_single_index(host, boxed(), 3).is_none());
}

#[test]
fn insert_stacking_tops_up_a_holding_index_before_an_empty_one() {
    let tanks = Tanks::new(vec![None, Some((HYDROGEN, 10))]);
    let handler: &dyn ResourceHandler<Chemical> = &tanks;
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    let went = host.transaction(|host, tx| {
        Ok::<_, ()>(insert_stacking(host, handler, &HYDROGEN, 95, Some(tx)))
    });
    assert_eq!(went, Ok(95));
    // The default `insert_resource` would have filled index 0 first.
    assert_eq!(
        tanks.held(),
        vec![Some((HYDROGEN, 5)), Some((HYDROGEN, 100))]
    );
}

#[test]
fn the_comparator_reads_0_when_empty_15_when_full_and_vanillas_steps_between() {
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    let signal = |host: &mut dyn ServerHost, held| {
        redstone_signal_from_resource_handler(host, &Tanks::new(held))
    };
    assert_eq!(signal(&mut *host, vec![None, None]), 0);
    assert_eq!(
        signal(
            &mut *host,
            vec![Some((HYDROGEN, 100)), Some((HYDROGEN, 100))]
        ),
        15
    );
    // One unit in a tank of 100: any content gives at least 1.
    assert_eq!(signal(&mut *host, vec![Some((HYDROGEN, 1)), None]), 1);
    // Half full: floor(0.5 * 14) + 1.
    assert_eq!(signal(&mut *host, vec![Some((HYDROGEN, 50)), None]), 8);
}

/// Indices holding `(amount, capacity)` of hydrogen each.
struct Fills(Vec<(u32, u32)>);

impl ResourceHandler<Chemical> for Fills {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        self.0.len()
    }
    fn resource(&self, _: &mut dyn ServerHost, _: usize) -> Option<Chemical> {
        Some(HYDROGEN)
    }
    fn amount(&self, _: &mut dyn ServerHost, index: usize) -> u32 {
        self.0[index].0
    }
    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &Chemical) -> u32 {
        self.0[index].1
    }
    fn insert(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0
    }
    fn extract(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0
    }
}

#[test]
fn an_overfull_index_does_not_fill_up_for_a_partial_one() {
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    // Fills of 2.0 and 0.5 read as 1.0 and 0.5, not as 1.25 capped to 15: 0.75 gives 11.
    let fills = Fills(vec![(200, 100), (50, 100)]);
    assert_eq!(redstone_signal_from_resource_handler(host, &fills), 11);
}

/// A handler of `size` indices each holding `u32::MAX` of hydrogen in a capacity of 1: amounts
/// and sizes past anything sane, which no query may panic on.
struct Oversized(usize);

impl ResourceHandler<Chemical> for Oversized {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        self.0
    }
    fn resource(&self, _: &mut dyn ServerHost, _: usize) -> Option<Chemical> {
        Some(HYDROGEN)
    }
    fn amount(&self, _: &mut dyn ServerHost, _: usize) -> u32 {
        u32::MAX
    }
    fn capacity(&self, _: &mut dyn ServerHost, _: usize, _: &Chemical) -> u32 {
        1
    }
    fn insert(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0
    }
    fn extract(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0
    }
}

#[test]
fn oversized_amounts_and_sizes_cap_instead_of_overflowing() {
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    assert_eq!(
        redstone_signal_from_resource_handler(host, &Oversized(1)),
        15
    );
    let combined = CombinedResourceHandler::new(
        host,
        vec![
            Box::new(Oversized(usize::MAX)) as Box<dyn ResourceHandler<Chemical>>,
            Box::new(Oversized(2)),
        ],
    );
    assert_eq!(combined.size(host), usize::MAX);
}

/// One index that holds nothing yet answers one more than it was asked: a mod bug.
struct Liar;

impl<R> ResourceHandler<R> for Liar {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }
    fn resource(&self, _: &mut dyn ServerHost, _: usize) -> Option<R> {
        None
    }
    fn amount(&self, _: &mut dyn ServerHost, _: usize) -> u32 {
        0
    }
    fn capacity(&self, _: &mut dyn ServerHost, _: usize, _: &R) -> u32 {
        u32::MAX
    }
    fn insert(&self, _: &mut dyn ServerHost, _: &Transaction, _: usize, _: &R, amount: u32) -> u32 {
        amount.saturating_add(1)
    }
    fn extract(&self, _: &mut dyn ServerHost, _: &Transaction, _: usize, _: &R, _: u32) -> u32 {
        0
    }
}

/// `handler` as a capability `owner`'s provider answered with.
fn capability<R: 'static>(
    handler: impl ResourceHandler<R> + 'static,
    owner: &'static str,
) -> Capability<dyn ResourceHandler<R>> {
    Capability {
        handler: Box::new(handler),
        owner: Some(owner),
    }
}

#[test]
fn a_part_answering_more_than_asked_shows_through_every_total() {
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    let tank_then_liar = |host: &mut dyn ServerHost| {
        CombinedResourceHandler::new(
            host,
            vec![
                Box::new(Tanks::new(vec![None])) as Box<dyn ResourceHandler<Chemical>>,
                Box::new(Liar),
            ],
        )
    };
    let insert = |host: &mut dyn ServerHost, handler: &dyn ResourceHandler<Chemical>, amount| {
        host.transaction(|host, tx| {
            Ok::<_, ()>(handler.insert_resource(host, tx, &HYDROGEN, amount))
        })
    };
    // The tank takes 100; the liar, asked for 50, answers 51: the total is over 150.
    let combined = tank_then_liar(&mut *host);
    assert_eq!(insert(&mut *host, &combined, 150), Ok(151));
    // The most a total is asked for is one short of `u32::MAX`, so the lie still shows.
    assert_eq!(insert(&mut *host, &combined, u32::MAX - 1), Ok(u32::MAX));
    // The default `insert_resource` and `insert_stacking` add up their indices the same way.
    let indices = CombinedResourceHandler::new(
        host,
        vec![
            Box::new(Tanks::new(vec![Some((HYDROGEN, 90))])) as Box<dyn ResourceHandler<Chemical>>,
            Box::new(Liar),
        ],
    );
    let stacking = host.transaction(|host, tx| {
        Ok::<_, ()>(insert_stacking(host, &indices, &HYDROGEN, 20, Some(tx)))
    });
    assert_eq!(stacking, Ok(21));

    // A move out of an endless source asks for the most there is: the lie is refused, not
    // read as all of it going in.
    let from = capability(InfiniteResourceHandler::new(HYDROGEN), "test:creative");
    let to = capability(tank_then_liar(&mut *host), "test:liar");
    let moved = host
        .transaction(|host, tx| r#move(host, &from, &to, |_: &Chemical| true, u32::MAX, Some(tx)));
    assert_eq!(
        moved,
        Err(Overanswered {
            owner: "test:liar".into()
        })
    );
}

/// A host with `ids` registered, stacking to 64, and their resources.
fn items_host(names: &[&str]) -> (FakeHost, Vec<ItemResource>) {
    use crate::prelude::ItemsHost;
    let mut host = FakeHost::new();
    let ids: Vec<_> = names
        .iter()
        .map(|id| crate::Identifier::parse(id).unwrap())
        .collect();
    host.registries.resolve_items(ids.clone()).unwrap();
    for name in names {
        host.registries
            .item_component(name, "minecraft:max_stack_size", Tag::Int(64));
    }
    let resources = ids
        .iter()
        .map(|id| ItemResource::of(&crate::ItemStack::new(host.item_key(id).unwrap(), 1).unwrap()))
        .collect();
    (host, resources)
}

/// A chest at `pos` holding `slots`, in the journaled level, so an abort puts it back.
fn chest(host: &mut FakeHost, pos: [i32; 3], slots: &[Option<(&ItemResource, u32)>]) {
    let entity = host.level.block_entity(pos).with_items(slots.len());
    for (slot, held) in slots.iter().enumerate() {
        entity.items.as_mut().unwrap().slots[slot] = held.map(|(r, n)| r.to_stack(n).unwrap());
    }
}

fn items_at(host: &mut FakeHost, pos: [i32; 3]) -> Capability<dyn ResourceHandler<ItemResource>> {
    let level = host.level.level();
    let server: &mut dyn ServerHost = host;
    level.capability(server, &crate::ITEMS, pos, None).unwrap()
}

/// How many each slot of the chest at `pos` holds.
fn counts(host: &FakeHost, pos: [i32; 3]) -> Vec<u32> {
    let entity = host.level.block_entity_at(pos).unwrap();
    let slots = &entity.items.as_ref().unwrap().slots;
    slots
        .iter()
        .map(|s| s.as_ref().map_or(0, |s| s.count()))
        .collect()
}

const A: [i32; 3] = [0, 64, 0];
const B: [i32; 3] = [1, 64, 0];

#[test]
fn move_first_moves_one_resource_and_an_abort_puts_it_back() {
    let (mut fake, r) = items_host(&["minecraft:stone", "minecraft:dirt"]);
    let (stone, dirt) = (&r[0], &r[1]);
    chest(&mut fake, A, &[Some((dirt, 3)), Some((stone, 5))]);
    chest(&mut fake, B, &[None, Some((stone, 10))]);
    let (from, to) = (items_at(&mut fake, A), items_at(&mut fake, B));
    let only_stone = |r: &ItemResource| r == stone;

    // Aborted: both chests as they were.
    let host: &mut dyn ServerHost = &mut fake;
    let aborted: Result<(), _> = host.transaction(|host, tx| {
        Err(move_first_stacking(
            host,
            &from,
            &to,
            only_stone,
            64,
            Some(tx),
        ))
    });
    assert_eq!(aborted, Err(Ok(Some((stone.clone(), 5)))));
    assert_eq!(counts(&fake, A), [3, 5]);
    assert_eq!(counts(&fake, B), [0, 10]);

    // Committed, stacking: onto the 10 stone, not into the empty slot.
    let host: &mut dyn ServerHost = &mut fake;
    let moved = host
        .transaction(|host, tx| move_first_stacking(host, &from, &to, only_stone, 64, Some(tx)));
    assert_eq!(moved, Ok(Some((stone.clone(), 5))));
    assert_eq!(counts(&fake, A), [3, 0]);
    assert_eq!(counts(&fake, B), [0, 15]);

    // `move_first` moves the first resource that goes, the dirt, and only it.
    let host: &mut dyn ServerHost = &mut fake;
    let moved = host.transaction(|host, tx| move_first(host, &from, &to, |_| true, 2, Some(tx)));
    assert_eq!(moved, Ok(Some((dirt.clone(), 2))));
    assert_eq!(counts(&fake, A), [1, 0]);
    assert_eq!(counts(&fake, B), [2, 15]);
}

#[test]
fn move_first_goes_on_with_its_resource_from_the_later_indices() {
    let (mut fake, r) = items_host(&["minecraft:stone", "minecraft:dirt"]);
    let (stone, dirt) = (&r[0], &r[1]);
    chest(
        &mut fake,
        A,
        &[Some((stone, 5)), Some((dirt, 3)), Some((stone, 4))],
    );
    chest(&mut fake, B, &[None, None, None]);
    let (from, to) = (items_at(&mut fake, A), items_at(&mut fake, B));
    let host: &mut dyn ServerHost = &mut fake;
    let moved = host.transaction(|host, tx| move_first(host, &from, &to, |_| true, 64, Some(tx)));
    assert_eq!(moved, Ok(Some((stone.clone(), 9))));
    assert_eq!(counts(&fake, A), [0, 3, 0]);
    assert_eq!(counts(&fake, B), [9, 0, 0]);
}

#[test]
fn find_extractable_resource_probes_without_taking_anything() {
    let (mut fake, r) = items_host(&["minecraft:stone", "minecraft:dirt"]);
    let (stone, dirt) = (&r[0], &r[1]);
    chest(&mut fake, A, &[Some((dirt, 3)), Some((stone, 5))]);
    let items = items_at(&mut fake, A);
    let host: &mut dyn ServerHost = &mut fake;
    let found = find_extractable_resource(host, &*items, |r: &ItemResource| r == stone, None);
    assert_eq!(found, Some(stone.clone()));
    assert_eq!(counts(&fake, A), [3, 5]);
}

#[test]
fn an_overanswer_after_an_honest_part_undoes_the_whole_move() {
    let (mut fake, r) = items_host(&["minecraft:stone"]);
    let stone = &r[0];
    chest(&mut fake, A, &[Some((stone, 5))]);
    chest(&mut fake, B, &[Some((stone, 62))]);
    let from = items_at(&mut fake, A);
    let honest = items_at(&mut fake, B).handler;
    let host: &mut dyn ServerHost = &mut fake;
    let to = capability(
        CombinedResourceHandler::new(host, vec![honest, Box::new(Liar)]),
        "test:liar",
    );
    // B takes 2; the liar, asked for 3, answers 4.
    let moved = host.transaction(|host, tx| r#move(host, &from, &to, |_| true, 5, Some(tx)));
    assert_eq!(
        moved,
        Err(Overanswered {
            owner: "test:liar".into()
        })
    );
    assert_eq!(counts(&fake, A), [5]);
    assert_eq!(counts(&fake, B), [62]);
}

#[test]
fn an_overanswer_nested_in_combined_handlers_leaves_nothing_behind_at_the_most_asked() {
    let (mut fake, r) = items_host(&["minecraft:stone"]);
    let stone = &r[0];
    chest(&mut fake, B, &[Some((stone, 60))]);
    let honest = items_at(&mut fake, B).handler;
    let host: &mut dyn ServerHost = &mut fake;
    // An honest chest, then a liar, in a handler that an endless one follows.
    let inner = CombinedResourceHandler::new(host, vec![honest, Box::new(Liar)]);
    let to = capability(
        CombinedResourceHandler::new(
            host,
            vec![
                Box::new(inner) as Box<dyn ResourceHandler<ItemResource>>,
                Box::new(InfiniteResourceHandler::new(stone.clone())),
            ],
        ),
        "test:liar",
    );
    let from = capability(InfiniteResourceHandler::new(stone.clone()), "test:creative");
    let moved = host.transaction(|host, tx| r#move(host, &from, &to, |_| true, u32::MAX, Some(tx)));
    assert_eq!(
        moved,
        Err(Overanswered {
            owner: "test:liar".into()
        })
    );
    // The chest took 4 before the liar answered; the abort put them back.
    assert_eq!(counts(&fake, B), [60]);
}

/// Two indices: hydrogen in index 0 that comes out only by the batch (10 at once), and helium in
/// index 1 that comes out by any amount.
struct Batches(Rc<RefCell<[u32; 2]>>);

impl ResourceHandler<Chemical> for Batches {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        2
    }
    fn resource(&self, _: &mut dyn ServerHost, index: usize) -> Option<Chemical> {
        Some(self.resource_at(index))
    }
    fn amount(&self, _: &mut dyn ServerHost, index: usize) -> u32 {
        self.0.borrow()[index]
    }
    fn capacity(&self, _: &mut dyn ServerHost, _: usize, _: &Chemical) -> u32 {
        10
    }
    fn insert(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        _: usize,
        _: &Chemical,
        _: u32,
    ) -> u32 {
        0
    }
    fn extract(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        chemical: &Chemical,
        amount: u32,
    ) -> u32 {
        let mut held = self.0.borrow_mut();
        if *chemical != self.resource_at(index) || (index == 0 && amount < 10) {
            return 0;
        }
        let came = amount.min(held[index]);
        held[index] -= came;
        came
    }
}

impl Batches {
    fn resource_at(&self, index: usize) -> Chemical {
        [HYDROGEN, Chemical("test:helium")][index].clone()
    }
}

#[test]
fn extract_first_chooses_by_the_whole_index_probe_then_asks_only_that_resource() {
    let held = Rc::new(RefCell::new([10, 5]));
    let handler = Batches(held.clone());
    let mut fake = FakeHost::new();
    let host: &mut dyn ServerHost = &mut fake;
    // Hydrogen gives its whole 10 but not 5, so it is the one chosen and 5 of it does not come
    // out; the helium that would give 5 is not tried.
    let taken = host
        .transaction(|host, tx| Ok::<_, ()>(extract_first(host, &handler, |_| true, 5, Some(tx))));
    assert_eq!(taken, Ok(None));
    assert_eq!(*held.borrow(), [10, 5]);
    // Asked for the batch, it comes out.
    let taken = host
        .transaction(|host, tx| Ok::<_, ()>(extract_first(host, &handler, |_| true, 10, Some(tx))));
    assert_eq!(taken, Ok(Some((HYDROGEN, 10))));
    assert_eq!(*held.borrow(), [0, 5]);
}
