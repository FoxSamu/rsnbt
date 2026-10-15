#![cfg(feature = "region")]

use std::fs::{self, File};
use std::io::{self, Write};

use rsnbt::region::RegionFile;


// TODO RegionFile needs more testing
//  currently we only test reading chunks

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

#[test]
#[cfg(feature = "bitpack")]
fn test_unpack_blocks() -> io::Result<()> {
    // Unpacks chunk 6,4 from the region file at data/region/r.0.0.mca,
    // and decodes the block state data from section 3 using BitStorage.
    //
    // A more integrated test compared to the above one, as it also uses
    // Tag manipulation and the use of BitStorage.

    use rsnbt::bitpack::BitStorage;
    
    use rsnbt::tag::Tag;
    use rsnbt::tag::TagList;

    let mut region = RegionFile::open_file("data/region/r.0.0.mca")?;

    let chunk = region.load(6, 4)?.unwrap();

    let (_, data) = chunk.decode_nbt()?.unwrap();

    // extract root tag map
    let Tag::Compound(mut map) = data else {
        panic!()
    };

    // extract sections list
    let Some(Tag::List(TagList::Compounds(mut list))) = map.remove(&String::from("sections")) else {
        panic!()
    };

    // get section 3
    let mut section = list.remove(3);

    // extract block states table from section
    let Some(Tag::Compound(mut block_states)) = section.remove(&String::from("block_states")) else {
        panic!()
    };


    // get palette from block states table
    let Some(Tag::List(TagList::Compounds(palette))) = block_states.remove(&String::from("palette")) else {
        panic!()
    };

    // get data from block states table
    let Some(Tag::LongArray(data)) = block_states.remove(&String::from("data")) else {
        panic!()
    };

    
    // compute max bit width
    let len = palette.len();
    let wdt = (64 - (len as u64 - 1).leading_zeros()).max(4);

    // create bit storage
    let storage = BitStorage::from_raw(wdt as u8, 4096, data.into_iter().map(|v| v as u64));


    // extract block ids from bit storage and format to file
    fs::create_dir_all("out/mca")?;
    let mut file = File::create("out/mca/c.6.4.blocks.txt")?;

    for i in 0..4096 {
        let val = storage.get(i);
        write!(file, "{:3}", val)?;

        if i % 256 == 255 {
            writeln!(file)?;
            writeln!(file)?;
        } else if i % 16 == 15 {
            writeln!(file)?;
        } else {
            write!(file, "")?;
        }
    }

    println!("block data was written to 'out/mca/c.6.4.block.txt'");

    Ok(())
}