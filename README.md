`rsnbt` is a Rust toolkit for handling Minecraft data files. It allows analysing and modifying NBT data, parsing and serialising SNBT, reading MCA files, and encoding and decoding Minecraft-specific compression schemes.

# Roadmap
- [x] Base NBT tree model -> `rsnbt::tag`
- [x] NBT serialization -> `rsnbt::io`
- [ ] NBT visitors
- [ ] SNBT serialization -> WIP `rsnbt::snbt`
- [x] LZ4Block encoding and decoding[^lz4b] -> `rsnbt::lz4b`
- [ ] MCR/MCA region file toolkit -> WIP `rsnbt::region`
- [ ] Integer packager[^intpack]

[^lz4b]: Minecraft uses the [lz4-java](https://github.com/yawkat/lz4-java/) library to handle LZ4 compression. This library comes with a non-conventional container format, LZ4Block, that Minecraft uses for chunk storage somtimes. This module provides the tools to decode and encode that format.
[^intpack]: To store blocks, Minecraft uses a palette that maps integers to block states. It then compresses the blocks of each chunk in small integers that don't take up more bits than absolutely necessary to store every palette entry, which is then encoded as a `LongArray` tag. This module provides tools to decode and encode such packed integer arrays.