/*!
This crate mainly deals with issuing and maintaining stability of indices.
It provides 4 structs and each helps in different area.

This library was created for my game development endeavor.
Not going great on that front as I kept restarting the project.
However, I saw these utility structures coming back multiple times so I'm making a crate for them.

# Use cases
| Struct        | Type      | Suggestion    | Description |
| -----------   | ----      | ----          |-----------  |
| [`Eids`]      | Id        | Dense data    | You want a way to create ids, and **do** care about recovering ids. |
| [`Sequence`]  | Id        | Sparse data   | You want a way to create ids, and **don't** care about recovering ids, but you don't want to use the HashMap-based [`Entities`] struct. |
| [`Entities`]  | Memory    | Sparse data   | You want mix sequence (ids not recycled) and HashMap together. |
| [`Tec`]       | Memory    | Dense data    | You want to use a vec to store data, but need constant entity removal. [`Tec`] reclaims the spaces for you as you insert more new items.
 */
use std::collections::{BTreeSet, HashMap};

mod cast_usize;
mod eids;
mod entities;
mod maximum;
mod predecessor;
mod sequence;
mod successor;
mod tomb_vec;

/**
Stands for Entity Id generator (ids are redeemable).
Basically a counter with a B-tree based free "set" (list).

# Use case
- you want to recycle ids due to frequent entity removal
- you want to use custom data structure but need id management
- ids start from zero

# Example
```
use stable_id::Eids;

let mut entities: Eids<u8> = Default::default();
let id = entities.claim();
entities.unclaim(id);
```

See [`Self::coalesce()`] if you want to pack ids together, like when you're trying to tighten up an array and
saving it into a database/save file (i.e. when game players are saving their progress).
*/
#[derive(Default)]
pub struct Eids<IndexT>
where
    IndexT: Ord,
{
    freed: BTreeSet<IndexT>,
    next: IndexT,
}

/**
An abstracted monotonically increasing counter structure.
Once you claim an id you can't go back.

# Example
```
use stable_id::Sequence;

let mut s: Sequence<u8> = Default::default();
assert_eq!(s.next(), 0);
assert_eq!(s.next(), 1);
assert_eq!(s.next(), 2);

let mut s = Sequence::continue_from(1234u16);
assert_eq!(s.next(), 1234);
assert_eq!(s.next(), 1235);
assert_eq!(s.next(), 1236);
```
 */
#[derive(Default)]
pub struct Sequence<IndexT> {
    counter: IndexT,
}

/// inspired by https://github.com/fitzgen/generational-arena/blob/72975c8355949c2338976d944e047c9d9f447174/src/lib.rs#L178
/// but without the generation stuff.
#[derive(Debug)]
pub(crate) enum Slot<DataT, IndexT> {
    Dead { next_free: IndexT },
    Alive(DataT),
}

/**
Short for [tombstone](https://en.wikipedia.org/wiki/Tombstone_(programming))-based vector.
Inspired by [generational-arena](https://github.com/fitzgen/generational-arena/blob/72975c8355949c2338976d944e047c9d9f447174/src/lib.rs#L178), but without the generation stuff.

# Features
- index stability when deleting an element
- maintain freed list, and is basically free for large structs

Use case: you have compact data that needs to be inserted & deleted while other objects maintain their index-based references.

Don't use it if:
- the data are sparse (use a HashMap or [`Entities`] instead)
- you don't need to remove data (use a Vec **with** [`Sequence`] instead)
*/
pub struct Tec<DataT, IndexT = usize> {
    vec: Vec<Slot<DataT, IndexT>>,
    /// invariants: the free index must be either
    ///      - pointer some dead slot within the `vec`
    ///      - or the end of the `vec`
    /// In other words, the `vec` cannot have trailing dead slots
    next_free: IndexT,
    count: usize,
}

/**
This is a sandwich of HashMap and [`Sequence`].

# Features:
- stable indices and not redeemable
- generated indices

Use case: you have sparse data or you just want something simple for prototyping.
*/
pub struct Entities<DataT, IndexT = usize> {
    data: HashMap<IndexT, DataT>,
    seq: Sequence<IndexT>,
}

/**
Predecessor trait for numbers.
*/
pub trait Predecessor {
    /// Return `self` - 1. Panics when `self` is at 0.
    fn prev(self) -> Self;
}

/**
Successor trait for numbers.
*/
pub trait Successor {
    /// Return `self` + 1. Panics if `self` is at maximum value.
    fn next(self) -> Self;
}

/**
A trait that describes the max value of an unsigned integer. This trait is used to detect overflow.
Also, it's used like a NULL terminator for the free list in [`Tec`].
*/
pub trait Maximum {
    /// Generally this should be `X::MAX`, where `X` is an unsigned integer. The value is used to detect overflow.
    fn max_value() -> Self;
}

/**
Trait for casting between an unsigned integer (up to 32 bits) to a usize.
Note: to() would panic if the value is greater or equal to the type's max.
*/
pub trait CastUsize {
    /** Turning an unsigned integer into a usize. */
    fn to(self) -> usize;
    /** Turning a usize into an unsigned integer. */
    fn from(val: usize) -> Self;
}
