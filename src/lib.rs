/*!
This crate mainly deals with issuing and maintaining stability of indices.
It provides 4 structs and each helps in different area.

This library was created for my game development endeavor.
Not going great on that front as I kept restarting the project.
However, I saw these utility structures coming back multiple times so I'm making a crate for them.

In version 0.2.0, you can supply custom Id tuple structs that are based on unsigned integers (from 8bit to 64bits).
The id type needs to be derived with the following:
```
// Minimal needed for all traits that are introduced by this crate.
#[derive(derive_stable_id::StableId)]
struct Id(u8);


// These are needed under normal circumstances.
#[derive(
    Default,
    Clone, Copy,
    PartialEq, Eq, Hash,
    PartialOrd, Ord,
    derive_stable_id::StableId,
)]
struct Id32(u32);

let x: stable_id::Eids<Id32> = Default::default();
let x: stable_id::Sequence<Id32> = Default::default();
let x: stable_id::SparseEntities<String, Id32> = Default::default();
let x: stable_id::Entities<String, Id32> = Default::default();
let x: stable_id::Tec<String, Id32> = Default::default();
```

# Use cases
| Struct                | Type          | Suggestion    | Description |
| -----------           | ----          | ----          |-----------  |
| [`Eids`]              | Id            | Dense data    | You want a way to create ids, and **do** care about recovering ids. |
| [`Sequence`]          | Id            | Sparse data   | You want a way to create ids, and **don't** care about recovering ids, but you don't want to use the HashMap-based [`Entities`] struct. |
| [`Entities`]          | Collection    | Dense data    | The go-to collection of this library.
| [`SparseEntities`]    | Collection    | Sparse data   | You want mix sequence (ids not recycled) and HashMap together. |
| [`Tec`]               | Collection    | Dense data    | You want to use a vec to store data, but need constant entity removal. [`Tec`] reclaims the spaces for you as you insert more new items.
 */
use std::collections::BTreeSet;

use rustc_hash::FxHashMap;

mod eids;
mod entities;
mod sequence;
mod sparse_entities;
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
#[derive(Clone, Default)]
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
assert_eq!(s.next_value(), 0);
assert_eq!(s.next_value(), 1);
assert_eq!(s.next_value(), 2);

let mut s = Sequence::continue_from(1234u16);
assert_eq!(s.next_value(), 1234);
assert_eq!(s.next_value(), 1235);
assert_eq!(s.next_value(), 1236);
```
 */
#[derive(Clone, Default)]
pub struct Sequence<IndexT> {
    counter: IndexT,
}

/// inspired by https://github.com/fitzgen/generational-arena/blob/72975c8355949c2338976d944e047c9d9f447174/src/lib.rs#L178
/// but without the generation stuff.
#[derive(Clone, Debug)]
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

```
use stable_id::Tec;

// use the `derive_more` crate to shortern the list
#[derive(derive_stable_id::StableId, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Id(u8);
struct Data { field: usize }

let mut storage: Tec<Data, Id> = Default::default();
assert_eq!(storage.alloc(Data {field: 123}), Id(0));
assert_eq!(storage.get(Id(0)).unwrap().field, 123);
```
*/
#[derive(Clone)]
pub struct Tec<DataT, IndexT = usize> {
    vec: Vec<Slot<DataT, IndexT>>,
    /// invariants: the free index must be either
    ///      - pointer some dead slot within the `vec`
    ///      - or the sentinal value of Maximum::maximum()
    /// In other words, the `vec` cannot have trailing dead slots
    next_free: IndexT,
    count: usize,
}

/**
This is a sandwich of HashMap and [`Sequence`].

# Features:
- stable indices and not redeemable
- generated indices

Use cases:
- you're removing more entities than you are adding
- you don't care about relaiming ids
*/
pub struct SparseEntities<DataT, IndexT = usize> {
    data: FxHashMap<IndexT, DataT>,
    seq: Sequence<IndexT>,
}

/**
This is a lazily memory-compact version of [`SparseEntities`].
Use cases are the same but there are different tradeoffs.


# Tradeoff vs [`SparseEntities`].
- this struct uses a hash-based virtual table to translate issued ids into an id used internally by its backing collection [`Tec`].
  So accessing items should be similar -- it's dictated by HashMap's access complexity, since once it finds
  the internal id, a random access follows.
- removing items is O([`Tec::remove()`]) = O(n lg n) though I have plans to make it O(n). An added benefits is [`remove()`] will also
  try to compact the memory by removing dead slots from [`Tec`] when there's a majority of dead slots -- it's another O(n) pass.
*/
#[derive(Clone)]
pub struct Entities<DataT, IndexT = usize> {
    vtable: FxHashMap<IndexT, IndexT>, // virtual id -> physical id
    data: Tec<DataT, IndexT>,
    seq: Sequence<IndexT>,
}
