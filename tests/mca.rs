#![cfg(feature = "region")]

use std::fs::{self, File};
use std::io::{self, Write};

use rsnbt::region::RegionFile;

#[test]
fn test_region() -> io::Result<()> {
    // Unpacks chunk 6,4 from the region file at data/region/r.0.0.mca
    // and writes the Debug output of the stored NBT tree to
    // out/mca/c.6.4.txt
    //
    // This test should fail if the chunk did not load correctly

    let mut region = RegionFile::open_file("data/region/r.0.0.mca")?;

    let header = region.header(6, 4)?.unwrap();
    dbg!(header);

    let chunk = region.load(6, 4)?.unwrap();

    let (name, data) = chunk.decode_nbt()?.unwrap();

    println!("tag name was '{}'", name);

    // Writing to a file because the printed data is HUGE (22k lines)
    fs::create_dir_all("out/mca")?;
    let mut file = File::create("out/mca/c.6.4.txt")?;
    let printed = format!("{:#?}", data);
    file.write_all(printed.as_bytes())?;
    file.flush()?;

    println!("tag data was written to 'out/mca/c.6.4.txt'");

    Ok(())
}