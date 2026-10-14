#![cfg(feature = "lz4b")]

use std::fs::File;
use std::io::Cursor;
use std::io::copy;

use rsnbt::lz4b::*;


// It is preferable that the decoder decodes the same way as lz4-java's LZ4BlockInputStream,
// and more importantly, that the encoder encodes the exact same way as lz4-java's
// LZ4BlockOutputStream. Even though the latter isn't necessarily a requirement, it is a nice
// thing to have nonetheless.
//
// In other words:
// - decoding data MUST yield the same data as the Java library (otherwise it is not lossless);
// - encoding data SHOULD yield the same data as the Java library.
//
// We test by compressing uncompressed data and see if it matches the compressed ata we generated
// using the library, and similarly for decoding. We compare data byte by byte: it must match!



// lipsum => a lorem ipsum
// - fast(1), block size 64

#[test]
fn test_lipsum() {
    match_data_decode("data/lz4b/lipsum.lz4b", "data/lz4b/lipsum.txt");
}

#[test]
fn test_lipsum_encode() {
    match_data_encode("data/lz4b/lipsum.lz4b", "data/lz4b/lipsum.txt", 64);
}



// lz4yap => claude's yapping about lz4
// - fast(1), block size 4096

#[test]
fn test_lz4yap() {
    match_data_decode("data/lz4b/lz4yap.lz4b", "data/lz4b/lz4yap.txt");
}

#[test]
fn test_lz4yap_encode() {
    match_data_encode("data/lz4b/lz4yap.lz4b", "data/lz4b/lz4yap.txt", 4096);
}



// helpers

fn match_data_decode(encoded_file: &str, decoded_file: &str) {
    let mut encoded_reader = LZ4BlockDecoder::new(File::open(encoded_file).unwrap(), true);
    let mut decoded_reader = File::open(decoded_file).unwrap();

    let mut encoded_data = Cursor::new(Vec::new());
    let mut decoded_data = Cursor::new(Vec::new());

    copy(&mut encoded_reader, &mut encoded_data).unwrap();
    copy(&mut decoded_reader, &mut decoded_data).unwrap();

    assert_eq!(encoded_data.into_inner(), decoded_data.into_inner());
}

fn match_data_encode(encoded_file: &str, decoded_file: &str, block_size: usize) {
    let mut encoded_reader = File::open(encoded_file).unwrap();
    let mut decoded_reader = File::open(decoded_file).unwrap();

    let mut encoded_data = Cursor::new(Vec::new());
    let mut decoded_data = Cursor::new(Vec::new());

    let mut encoder = LZ4BlockEncoder::new(decoded_data, block_size, LZ4BlockCompression::Fast(1));

    copy(&mut encoded_reader, &mut encoded_data).unwrap();
    copy(&mut decoded_reader, &mut encoder).unwrap();

    decoded_data = encoder.finish().unwrap();

    assert_eq!(encoded_data.into_inner(), decoded_data.into_inner());
}

// This is here so we can use it if needed
#[allow(unused)]
fn encode_to_file(encoded_file: &str, decoded_file: &str, block_size: usize) {
    let encoded_writer = File::create(encoded_file).unwrap();
    let mut decoded_reader = File::open(decoded_file).unwrap();


    let mut encoder = LZ4BlockEncoder::new(encoded_writer, block_size, LZ4BlockCompression::Fast(1));

    copy(&mut decoded_reader, &mut encoder).unwrap();

    encoder.finish().unwrap();
}