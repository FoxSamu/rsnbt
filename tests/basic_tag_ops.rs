use rsnbt::tag::*;


#[test] 
fn byte_from_numeral() {
    use rsnbt::tag::TagList;
    
    let mut empty = tag_list![ints:];
    let mut not_empty = TagList::ints([1, 2, 3]);
    
    empty.clear();
    assert_eq!(TagList::Empty, empty);
    
    not_empty.clear();
    assert_eq!(TagList::Empty, not_empty);
}