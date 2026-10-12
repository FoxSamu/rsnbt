use crate::snbt::ParseConfig;
use crate::snbt::parse_chars;

#[test]
fn parse_1() {
    let tag = r##"
        {
            This: ["is", "SNBT"],
            Foo: Bar,
            Baz: 2ub,
            Gus: {
                Lorem: true,
                Ipsum: 255,
            },
            Hi: ["this", "is", {"FOOOOO": 3}]
        }
    "##;

    let config = ParseConfig::minecraft();
    let result = parse_chars(tag, &config).unwrap();
    dbg!(result);
}
