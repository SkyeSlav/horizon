//! NeoForge 26.3's stock resource handlers and `ResourceHandlerUtil`: wrappers a mod composes its
//! own [`ResourceHandler`]s with, and the queries and moves every transfer mod writes. Each write
//! runs in the caller's transaction `tx`; to simulate one, abort it.
use crate::content::resource::{Ends, MAX_REQUEST, in_turn, move_once};
use crate::{Capability, HostTransaction, Overanswered, ResourceHandler, ServerHost, Transaction};
use std::convert::Infallible;
use std::ops::Range;

/// Several handlers as one (`CombinedResourceHandler`): their indices one after another, the
/// first handler's from 0. The size is fixed when it is built. An insert or extract without an
/// index asks each handler in turn, so each places it its own way.
// no wire: a mod-side handler, invoked where it was built; never crosses the boundary.
// exhaustive: its fields are private, so mods cannot build or destructure it
pub struct CombinedResourceHandler<R> {
    handlers: Vec<Box<dyn ResourceHandler<R>>>,
    /// Where each handler's indices end, counted from the first's: its sizes added up, saturating
    /// at `usize::MAX`, past which no index is numbered.
    ends: Vec<usize>,
}

impl<R> CombinedResourceHandler<R> {
    /// `handlers`, numbered in this order, as big as they are now.
    pub fn new(host: &mut dyn ServerHost, handlers: Vec<Box<dyn ResourceHandler<R>>>) -> Self {
        let mut end = 0usize;
        let ends = handlers
            .iter()
            .map(|handler| {
                end = end.saturating_add(handler.size(host));
                end
            })
            .collect();
        Self { handlers, ends }
    }
    /// The handler index `index` falls in, and the index within it.
    fn locate(&self, index: usize) -> Option<(&dyn ResourceHandler<R>, usize)> {
        let which = self.ends.partition_point(|&end| end <= index);
        let handler = self.handlers.get(which)?;
        let start = which.checked_sub(1).map_or(0, |before| self.ends[before]);
        Some((handler.as_ref(), index - start))
    }
}

impl<R> ResourceHandler<R> for CombinedResourceHandler<R> {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        self.ends.last().copied().unwrap_or(0)
    }
    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<R> {
        let (handler, at) = self.locate(index)?;
        handler.resource(host, at)
    }
    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        self.locate(index)
            .map_or(0, |(handler, at)| handler.amount(host, at))
    }
    fn capacity(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> u32 {
        self.locate(index)
            .map_or(0, |(handler, at)| handler.capacity(host, at, resource))
    }
    fn is_valid(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> bool {
        self.locate(index)
            .is_some_and(|(handler, at)| handler.is_valid(host, at, resource))
    }
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.locate(index).map_or(0, |(handler, at)| {
            handler.insert(host, tx, at, resource, amount)
        })
    }
    fn extract(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.locate(index).map_or(0, |(handler, at)| {
            handler.extract(host, tx, at, resource, amount)
        })
    }
    fn insert_resource(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        resource: &R,
        amount: u32,
    ) -> u32 {
        in_turn(amount, &self.handlers, |handler, left| {
            handler.insert_resource(host, tx, resource, left)
        })
    }
    fn extract_resource(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        resource: &R,
        amount: u32,
    ) -> u32 {
        in_turn(amount, &self.handlers, |handler, left| {
            handler.extract_resource(host, tx, resource, left)
        })
    }
}

/// Every call passed on to another handler (`DelegatingResourceHandler`), the base of a wrapper
/// that changes some of them: a struct holding it implements [`ResourceHandler`], answers the
/// calls it changes itself and forwards the rest to [`delegate`](Self::delegate). This one
/// takes at most `limit` of anything in per insert and leaves the rest as the handler has it:
///
/// ```
/// use mod_api::{DelegatingResourceHandler, ResourceHandler, ServerHost, Transaction};
///
/// struct Throttled<R> {
///     inner: DelegatingResourceHandler<R>,
///     limit: u32,
/// }
///
/// impl<R> ResourceHandler<R> for Throttled<R> {
///     fn size(&self, host: &mut dyn ServerHost) -> usize {
///         self.inner.size(host)
///     }
///     fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<R> {
///         self.inner.resource(host, index)
///     }
///     fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
///         self.inner.amount(host, index)
///     }
///     fn capacity(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> u32 {
///         self.inner.capacity(host, index, resource)
///     }
///     fn is_valid(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> bool {
///         self.inner.is_valid(host, index, resource)
///     }
///     fn insert(
///         &self,
///         host: &mut dyn ServerHost,
///         tx: &Transaction,
///         index: usize,
///         resource: &R,
///         amount: u32,
///     ) -> u32 {
///         self.inner.insert(host, tx, index, resource, amount.min(self.limit))
///     }
///     fn extract(
///         &self,
///         host: &mut dyn ServerHost,
///         tx: &Transaction,
///         index: usize,
///         resource: &R,
///         amount: u32,
///     ) -> u32 {
///         self.inner.extract(host, tx, index, resource, amount)
///     }
/// }
/// ```
// no wire: a mod-side handler, invoked where it was built; never crosses the boundary.
// exhaustive: its field is private, so mods cannot build or destructure it
pub struct DelegatingResourceHandler<R>(Box<dyn ResourceHandler<R>>);

impl<R> DelegatingResourceHandler<R> {
    /// Passes every call on to `handler`.
    pub fn new(handler: Box<dyn ResourceHandler<R>>) -> Self {
        Self(handler)
    }
    /// The handler the calls go to (`getDelegate`).
    pub fn delegate(&self) -> &dyn ResourceHandler<R> {
        self.0.as_ref()
    }
}

impl<R> ResourceHandler<R> for DelegatingResourceHandler<R> {
    fn set_max_stack_size(&self, host: &mut dyn ServerHost, tx: &Transaction, limit: u32) -> bool {
        self.0.set_max_stack_size(host, tx, limit)
    }
    fn size(&self, host: &mut dyn ServerHost) -> usize {
        self.0.size(host)
    }
    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<R> {
        self.0.resource(host, index)
    }
    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        self.0.amount(host, index)
    }
    fn capacity(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> u32 {
        self.0.capacity(host, index, resource)
    }
    fn is_valid(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> bool {
        self.0.is_valid(host, index, resource)
    }
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.0.insert(host, tx, index, resource, amount)
    }
    fn extract(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.0.extract(host, tx, index, resource, amount)
    }
    fn insert_resource(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.0.insert_resource(host, tx, resource, amount)
    }
    fn extract_resource(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.0.extract_resource(host, tx, resource, amount)
    }
}

/// Indices `start..end` of another handler, numbered from 0 (`RangedResourceHandler`): a
/// machine's input slots as one side reaches them.
// no wire: a mod-side handler, invoked where it was built; never crosses the boundary.
// exhaustive: its fields are private, so mods cannot build or destructure it
pub struct RangedResourceHandler<R> {
    handler: Box<dyn ResourceHandler<R>>,
    range: Range<usize>,
}

impl<R> RangedResourceHandler<R> {
    /// Indices `start..end` of `handler`: none when the range is empty or reaches past the
    /// handler's size now.
    pub fn of(
        host: &mut dyn ServerHost,
        handler: Box<dyn ResourceHandler<R>>,
        start: usize,
        end: usize,
    ) -> Option<Self> {
        (start < end && end <= handler.size(host)).then_some(Self {
            handler,
            range: start..end,
        })
    }
    /// Index `index` of `handler` alone (`ofSingleIndex`): none when it has no such index.
    pub fn of_single_index(
        host: &mut dyn ServerHost,
        handler: Box<dyn ResourceHandler<R>>,
        index: usize,
    ) -> Option<Self> {
        Self::of(host, handler, index, index.checked_add(1)?)
    }
    /// `index`'s index in the wrapped handler, or none past this range.
    fn inner(&self, index: usize) -> Option<usize> {
        (index < self.range.len()).then(|| self.range.start + index)
    }
}

impl<R> ResourceHandler<R> for RangedResourceHandler<R> {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        self.range.len()
    }
    fn resource(&self, host: &mut dyn ServerHost, index: usize) -> Option<R> {
        let at = self.inner(index)?;
        self.handler.resource(host, at)
    }
    fn amount(&self, host: &mut dyn ServerHost, index: usize) -> u32 {
        self.inner(index)
            .map_or(0, |at| self.handler.amount(host, at))
    }
    fn capacity(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> u32 {
        self.inner(index)
            .map_or(0, |at| self.handler.capacity(host, at, resource))
    }
    fn is_valid(&self, host: &mut dyn ServerHost, index: usize, resource: &R) -> bool {
        self.inner(index)
            .is_some_and(|at| self.handler.is_valid(host, at, resource))
    }
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.inner(index)
            .map_or(0, |at| self.handler.insert(host, tx, at, resource, amount))
    }
    fn extract(
        &self,
        host: &mut dyn ServerHost,
        tx: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        self.inner(index)
            .map_or(0, |at| self.handler.extract(host, tx, at, resource, amount))
    }
}

/// No indices, refusing everything (`EmptyResourceHandler`): what a provider answers for a side
/// that reaches nothing.
#[derive(Debug)]
// no wire: a mod-side handler, invoked where it was built; never crosses the boundary.
// exhaustive: its field is private, so mods cannot build or destructure it
pub struct EmptyResourceHandler(());

static EMPTY: EmptyResourceHandler = EmptyResourceHandler(());

impl EmptyResourceHandler {
    /// The one empty handler (`EmptyResourceHandler.instance`).
    pub fn instance() -> &'static Self {
        &EMPTY
    }
}

impl<R> ResourceHandler<R> for EmptyResourceHandler {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        0
    }
    fn resource(&self, _: &mut dyn ServerHost, _: usize) -> Option<R> {
        None
    }
    fn amount(&self, _: &mut dyn ServerHost, _: usize) -> u32 {
        0
    }
    fn capacity(&self, _: &mut dyn ServerHost, _: usize, _: &R) -> u32 {
        0
    }
    fn insert(&self, _: &mut dyn ServerHost, _: &Transaction, _: usize, _: &R, _: u32) -> u32 {
        0
    }
    fn extract(&self, _: &mut dyn ServerHost, _: &Transaction, _: usize, _: &R, _: u32) -> u32 {
        0
    }
}

/// One index of an endless amount of one resource (`InfiniteResourceHandler`): every insert and
/// extract of it is taken whole, and anything else is refused. A creative tank.
#[derive(Clone, Debug, PartialEq)]
// no wire: a mod-side handler, invoked where it was built; never crosses the boundary.
// exhaustive: its fields are private, so mods cannot build or destructure it
pub struct InfiniteResourceHandler<R>(R);

impl<R> InfiniteResourceHandler<R> {
    /// An endless amount of `resource`.
    pub fn new(resource: R) -> Self {
        Self(resource)
    }
}

impl<R: Clone + PartialEq> ResourceHandler<R> for InfiniteResourceHandler<R> {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }
    fn resource(&self, _: &mut dyn ServerHost, index: usize) -> Option<R> {
        (index == 0).then(|| self.0.clone())
    }
    fn amount(&self, _: &mut dyn ServerHost, index: usize) -> u32 {
        if index == 0 { u32::MAX } else { 0 }
    }
    fn capacity(&self, _: &mut dyn ServerHost, index: usize, resource: &R) -> u32 {
        if index == 0 && *resource == self.0 {
            u32::MAX
        } else {
            0
        }
    }
    fn insert(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        if self.is_valid(host, index, resource) {
            amount
        } else {
            0
        }
    }
    fn extract(
        &self,
        host: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        resource: &R,
        amount: u32,
    ) -> u32 {
        if self.is_valid(host, index, resource) {
            amount
        } else {
            0
        }
    }
}

/// One index that destroys whatever goes in and holds nothing (`VoidingResourceHandler`): a
/// trash can.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// no wire: a mod-side handler, invoked where it was built; never crosses the boundary.
// exhaustive: a unit marker
pub struct VoidingResourceHandler;

impl<R> ResourceHandler<R> for VoidingResourceHandler {
    fn size(&self, _: &mut dyn ServerHost) -> usize {
        1
    }
    fn resource(&self, _: &mut dyn ServerHost, _: usize) -> Option<R> {
        None
    }
    fn amount(&self, _: &mut dyn ServerHost, _: usize) -> u32 {
        0
    }
    fn capacity(&self, _: &mut dyn ServerHost, index: usize, _: &R) -> u32 {
        if index == 0 { u32::MAX } else { 0 }
    }
    fn insert(
        &self,
        _: &mut dyn ServerHost,
        _: &Transaction,
        index: usize,
        _: &R,
        amount: u32,
    ) -> u32 {
        if index == 0 { amount } else { 0 }
    }
    fn extract(&self, _: &mut dyn ServerHost, _: &Transaction, _: usize, _: &R, _: u32) -> u32 {
        0
    }
}

/// Runs `f` in `tx`, or in a transaction of its own when there is none (NeoForge's null
/// context opens a root transaction): committed on `Ok`, aborted on `Err`.
fn within<T, E>(
    host: &mut dyn ServerHost,
    tx: Option<&Transaction>,
    f: impl FnOnce(&mut dyn ServerHost, &Transaction) -> Result<T, E>,
) -> Result<T, E> {
    match tx {
        Some(tx) => f(host, tx),
        None => host.transaction(|host, tx| f(host, tx)),
    }
}

/// The first index holding `resource` (`indexOf`).
pub fn index_of<R: PartialEq>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
    resource: &R,
) -> Option<usize> {
    (0..handler.size(host)).find(|&index| {
        handler.amount(host, index) > 0 && handler.resource(host, index).as_ref() == Some(resource)
    })
}

/// Whether an index holds `resource` (`contains`).
pub fn contains<R: PartialEq>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
    resource: &R,
) -> bool {
    index_of(host, handler, resource).is_some()
}

/// Whether every index holds nothing (`isEmpty`); a handler of no indices is.
pub fn is_empty<R>(host: &mut dyn ServerHost, handler: &dyn ResourceHandler<R>) -> bool {
    (0..handler.size(host)).all(|index| handler.amount(host, index) == 0)
}

/// Whether every index holds a resource up to its capacity for it (`isFull`); a handler of no
/// indices is.
pub fn is_full<R>(host: &mut dyn ServerHost, handler: &dyn ResourceHandler<R>) -> bool {
    (0..handler.size(host)).all(|index| {
        handler.resource(host, index).is_some_and(|resource| {
            let amount = handler.amount(host, index);
            amount > 0 && amount >= handler.capacity(host, index, &resource)
        })
    })
}

/// Whether `resource` may go in some index (`ResourceHandlerUtil.isValid`).
pub fn is_valid<R>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
    resource: &R,
) -> bool {
    (0..handler.size(host)).any(|index| handler.is_valid(host, index, resource))
}

/// A comparator's signal reading `handler` (`getRedstoneSignalFromResourceHandler`): the mean
/// fill, amount over capacity and at most 1 for each, of the indices that hold something and
/// have a capacity for it, mapped onto 0..=15 as vanilla's container signal is: 0 when there
/// are none, 15 when all are full.
pub fn redstone_signal_from_resource_handler<R>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
) -> u8 {
    let mut fill = 0f32;
    let mut counted = 0usize;
    for index in 0..handler.size(host) {
        let Some(resource) = handler.resource(host, index) else {
            continue;
        };
        let amount = handler.amount(host, index);
        let capacity = handler.capacity(host, index, &resource);
        if amount > 0 && capacity > 0 {
            // An oversized amount (a mod's, up to `u32::MAX` over a capacity of 1) counts as full.
            fill += (amount as f32 / capacity as f32).min(1.0);
            counted += 1;
        }
    }
    if counted == 0 {
        return 0;
    }
    let fill = fill / counted as f32;
    // `Mth.lerpDiscrete(fill, 0, 15)`.
    (fill * 14.0).floor() as u8 + u8::from(fill > 0.0)
}

/// Puts up to `amount` of `resource` into `handler`, topping up indices that hold it before empty
/// ones (`insertStacking`), in `tx`, or in a transaction of its own without one: how much went in.
pub fn insert_stacking<R: PartialEq>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
    resource: &R,
    amount: u32,
    tx: Option<&Transaction>,
) -> u32 {
    let Ok(went) = within(host, tx, |host, tx| {
        let size = handler.size(host);
        let indices = [true, false]
            .into_iter()
            .flat_map(|holding| (0..size).map(move |index| (holding, index)));
        Ok::<_, Infallible>(in_turn(amount, indices, |(holding, index), left| {
            let held = handler.amount(host, index) > 0
                && handler.resource(host, index).as_ref() == Some(resource);
            if held == holding {
                handler.insert(host, tx, index, resource, left)
            } else {
                0
            }
        }))
    });
    went
}

/// Takes up to `amount` of the first resource `filter` accepts out of `handler`, from wherever it
/// holds it (`extractFirst`), in `tx`, or in a transaction of its own without one: what came out
/// and how much, none when nothing did. The resource is chosen once, by
/// [`find_extractable_resource`], and only that one is asked for `amount`: a resource that gives a
/// whole index's batch but not `amount` is chosen, and nothing comes out.
pub fn extract_first<R>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
    filter: impl Fn(&R) -> bool,
    amount: u32,
    tx: Option<&Transaction>,
) -> Option<(R, u32)> {
    let Ok(taken) = within(host, tx, |host, tx| {
        let Some(resource) = find_extractable_resource(host, handler, filter, Some(tx)) else {
            return Ok::<_, Infallible>(None);
        };
        let came = handler.extract_resource(host, tx, &resource, amount);
        Ok((came > 0).then_some((resource, came)))
    });
    taken
}

/// The first resource `filter` accepts that `handler` can give (`findExtractableResource`): one
/// whole index's amount of it is taken in a transaction nested in `tx` (in a root one without)
/// and aborted, so nothing changes.
pub fn find_extractable_resource<R>(
    host: &mut dyn ServerHost,
    handler: &dyn ResourceHandler<R>,
    filter: impl Fn(&R) -> bool,
    tx: Option<&Transaction>,
) -> Option<R> {
    let Ok(found) = within(host, tx, |host, _| {
        for index in 0..handler.size(host) {
            let amount = handler.amount(host, index).min(MAX_REQUEST);
            if amount == 0 {
                continue;
            }
            let Some(resource) = handler.resource(host, index).filter(|r| filter(r)) else {
                continue;
            };
            let probe: Result<(), u32> = host
                .transaction(|host, tx| Err(handler.extract(host, tx, index, &resource, amount)));
            if probe.is_err_and(|came| came > 0) {
                return Ok::<_, Infallible>(Some(resource));
            }
        }
        Ok(None)
    });
    found
}

/// Moves up to `max` of what `filter` accepts out of `from` into wherever `to` takes it, index by
/// index from 0 (`ResourceHandlerUtil.move`), in `tx`, or in one of its own without: how much
/// moved. Each move is all or nothing, in a transaction of its own: what `to` won't take stays in
/// `from`, and a move `to` takes only part of is tried again with that part (a cauldron gives
/// whole thirds). A handler answering more than it was asked has its move undone and ends the
/// moving with [`Overanswered`], which aborts the whole move, with or without a caller's context.
/// With a context, the move runs in its own nested transaction; catching its error leaves the
/// caller's transaction open without retaining any of this move's earlier steps.
pub fn r#move<R: PartialEq>(
    host: &mut dyn ServerHost,
    from: &Capability<dyn ResourceHandler<R>>,
    to: &Capability<dyn ResourceHandler<R>>,
    filter: impl Fn(&R) -> bool,
    max: u32,
    tx: Option<&Transaction>,
) -> Result<u32, Overanswered> {
    move_by(
        host,
        from,
        to,
        filter,
        max,
        tx,
        false,
        |host, tx, resource, out| to.insert_resource(host, tx, resource, out),
    )
    .map(|(_, moved)| moved)
}

/// As [`move`](self::r#move), inserting with [`insert_stacking`] (`moveStacking`): into `to`'s
/// indices that hold the resource before its empty ones.
pub fn move_stacking<R: PartialEq>(
    host: &mut dyn ServerHost,
    from: &Capability<dyn ResourceHandler<R>>,
    to: &Capability<dyn ResourceHandler<R>>,
    filter: impl Fn(&R) -> bool,
    max: u32,
    tx: Option<&Transaction>,
) -> Result<u32, Overanswered> {
    move_by(
        host,
        from,
        to,
        filter,
        max,
        tx,
        false,
        |host, tx, resource, out| insert_stacking(host, &**to, resource, out, Some(tx)),
    )
    .map(|(_, moved)| moved)
}

/// As [`move`](self::r#move), but moving only the first resource `filter` accepts that `to` takes
/// any of, from the indices after it too (`moveFirst`): what moved and how much, none when
/// nothing did.
pub fn move_first<R: PartialEq>(
    host: &mut dyn ServerHost,
    from: &Capability<dyn ResourceHandler<R>>,
    to: &Capability<dyn ResourceHandler<R>>,
    filter: impl Fn(&R) -> bool,
    max: u32,
    tx: Option<&Transaction>,
) -> Result<Option<(R, u32)>, Overanswered> {
    let (first, moved) = move_by(
        host,
        from,
        to,
        filter,
        max,
        tx,
        true,
        |host, tx, resource, out| to.insert_resource(host, tx, resource, out),
    )?;
    Ok(first.map(|first| (first, moved)))
}

/// As [`move_first`], inserting with [`insert_stacking`] (`moveFirstStacking`).
pub fn move_first_stacking<R: PartialEq>(
    host: &mut dyn ServerHost,
    from: &Capability<dyn ResourceHandler<R>>,
    to: &Capability<dyn ResourceHandler<R>>,
    filter: impl Fn(&R) -> bool,
    max: u32,
    tx: Option<&Transaction>,
) -> Result<Option<(R, u32)>, Overanswered> {
    let (first, moved) = move_by(
        host,
        from,
        to,
        filter,
        max,
        tx,
        true,
        |host, tx, resource, out| insert_stacking(host, &**to, resource, out, Some(tx)),
    )?;
    Ok(first.map(|first| (first, moved)))
}

/// What the moves share: up to `max` of the resources `filter` accepts, index by index, each step
/// `insert`ed into `to`. With `only_first`, once a resource has moved, only indices holding an
/// equal one move. The first resource that moved and the total.
#[allow(clippy::too_many_arguments)]
fn move_by<R: PartialEq>(
    host: &mut dyn ServerHost,
    from: &Capability<dyn ResourceHandler<R>>,
    to: &Capability<dyn ResourceHandler<R>>,
    filter: impl Fn(&R) -> bool,
    max: u32,
    tx: Option<&Transaction>,
    only_first: bool,
    insert: impl Fn(&mut dyn ServerHost, &Transaction, &R, u32) -> u32,
) -> Result<(Option<R>, u32), Overanswered> {
    let transfer = |host: &mut dyn ServerHost, _: &Transaction| {
        let mut first: Option<R> = None;
        let mut moved = 0;
        for index in 0..from.size(host) {
            while moved < max {
                let Some(resource) = from.resource(host, index).filter(|r| {
                    filter(r) && (!only_first || first.as_ref().is_none_or(|first| first == r))
                }) else {
                    break;
                };
                let want = (max - moved).min(from.amount(host, index)).min(MAX_REQUEST);
                if want == 0 {
                    break;
                }
                let went = move_once(
                    host,
                    Ends {
                        from,
                        to,
                        at: &index,
                    },
                    want,
                    |host, tx, want| from.extract(host, tx, index, &resource, want),
                    |host, tx, out| insert(host, tx, &resource, out),
                )?;
                if went == 0 {
                    break;
                }
                moved += went;
                first.get_or_insert(resource);
            }
        }
        Ok((first, moved))
    };
    if only_first {
        within(host, tx, transfer)
    } else {
        host.transaction(transfer)
    }
}

#[cfg(test)]
#[path = "resource_util_tests.rs"]
mod tests;
