`rsnbt` is a Rust toolkit for handling Minecraft data files.
It allows analysing and modifying NBT data, parsing and
serialising SNBT, reading MCA files, and encoding and decoding
Minecraft-specific compression schemes.

Where some crates support NBT, and others support region files,
`rsnbt` is a complete collection of tools that integrate with
eachother while remaining structurally decoupled.


# Roadmap
This crate is currently in heavy development. Here is a roadmap of the things
we want to achieve with this crate. Currently, my focus is mainly on handling
_Minecraft: Java Edition_ files.

## High priority
- [x] Base NBT tree model -> `rsnbt::tag`
- [x] NBT codec -> `rsnbt::io`
- [ ] NBT visitors
- [ ] SNBT parsing & writing -> WIP `rsnbt::snbt`
- [x] LZ4Block codec[^lz4b] -> `rsnbt::lz4b`
- [ ] MCR/MCA region file toolkit -> WIP `rsnbt::region`
- [ ] Concurrent chunk database
- [ ] Packing integers[^intpack]
- [ ] Release on [crates.io](https://crates.io/)

## Low priority
- [ ] Bedrock LevelDB support[^leveldb]
- [ ] Serialisation interface
- [ ] Further understanding of data

[^lz4b]: Minecraft uses the [lz4-java](https://github.com/yawkat/lz4-java/) 
library to handle LZ4 compression. This library comes with a non-conventional 
container format, LZ4Block, that Minecraft uses for chunk storage somtimes. 
This module provides the tools to decode and encode that format.

[^intpack]: To store blocks, Minecraft uses a palette that maps integers to 
block states. It then compresses the blocks of each chunk in small integers 
that don't take up more bits than absolutely necessary to store every palette 
entry, which is then encoded as a `LongArray` tag. This module provides tools 
to decode and encode such packed integer arrays.

[^leveldb]: _Minecraft: Bedrock Edition_ uses a fork of LevelDB with support for
[zlib](https://zlib.net/). This is unconventional and not usually supported by
LevelDB crates.
[`rusty_leveldb`](https://docs.rs/rusty-leveldb/latest/rusty_leveldb/)
supports custom compression schemes and is pure Rust, which is probably what
I will build upon.


# Usage
**The crate is not yet ready for use.** Once this crate is published on
[crates.io](https://crates.io/) I will further document usage.


# Examples

## Handling region files using `rsnbt::region`
Minecraft stores chunks in region files of 32 by 32 chunks. There region 
files have file extension `.mca` (or `.mcr` in very old versions) and are 
a container for chunk NBT tags. The `rsnbt::region` module allows you to 
read and modify these region files.

You must enable the `region` feature to use this module.

```rust
use rsnbt::tag::Tag;
use rsnbt::region::RegionFile;

// Open a RegionFile
//
// open_file(...) is sugar for open(File::open(...)). RegionFile can
// be used by anything that implements Read+Seek but it can only modify
// anything if it also implements Write.
let mut region = RegionFile::open_file("path/to/region/r.0.0.mca")?;

// Load a chunk blob
//
// This loads the chunk data into a buffer. Use region.header(...)
// if you just want to inspect the chunk header without loading
// all the data.
let Some(blob) = region.load(6, 4)? else {
    panic!("Chunk does not exist.");
};

// Decode the chunk NBT
//
// The decode_nbt method will automatically choose the correct
// compression decoder based on the chunk header.
let Some((_root_name, root_tag)) = blob.decode_nbt()? else {
    // Either the chunk was stored in a separate file
    // or it uses an unconventional compression scheme.
    // This pretty much never happens in vanilla worlds.
    panic!("Chunk was stored externally or did not use vanilla compression.");
};

// Inspect the NBT
let Tag::Compound(tag_map) = root_tag else {
    // Chunk data is a compound tag.
    panic!("Root is not a compound tag.");
};

let block_entities = tag_map.get(&String::from("block_entities"));
dbg!(block_entities);
```


# License
<p>
<a href="https://www.gnu.org/licenses/gpl-3.0.en.html">
<img src="assets/gplv3_logo.png"/>
</a>
</p>

**Copyright &copy; 2026 &nbsp; O. W. Nankman**

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
