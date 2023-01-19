# Overview

This crate mainly deals with issuing and maintaining stability of indices.
It provides 4 structs and each helps in different area.
This library was created for my game development endeavor.
Not going great on that front as I kept restarting the project.
However, I saw these utility structures coming back multiple times so I'm making a crate for them.

# installation

```sh
cargo add stable-id
```

# Use cases

| Struct       | Type   | Suggestion  | Description                                                                                                                              |
| ------------ | ------ | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| [`Eids`]     | Id     | Dense data  | You want a way to create ids, and **do** care about recovering ids.                                                                      |
| [`Sequence`] | Id     | Sparse data | You want a way to create ids, and **don't** care about recovering ids, but you don't want to use the HashMap-based [`Entities`] struct.  |
| [`Entities`] | Memory | Sparse data | You want mix sequence (ids not recycled) and HashMap together.                                                                           |
| [`Tec`]      | Memory | Dense data  | You want to use a vec to store data, but need constant entity removal. [`Tec`] reclaims the spaces for you as you insert more new items. |
