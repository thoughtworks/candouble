extern crate speedboat;

use speedboat::stub::Stub;
use speedboat::can::CANMessage;

#[test]
fn it_stub_matches_when_all_predicates_are_true() {
    let stub = Stub::from_file("tests/it_stubs.json").expect("Failed to parse JSON");
    let message = CANMessage::with_content(0x0101, 0, &[0xCA, 0xFE]);
    assert_eq!(true, stub.matches_message(&message));
}

#[test]
fn it_stub_matches_when_at_least_one_predicate_is_false() {
    let stub = Stub::from_file("tests/it_stubs.json").expect("Failed to parse JSON");
    let message = CANMessage::with_content(0x0102, 0, &[0xCA, 0xFE]);
    assert_eq!(false, stub.matches_message(&message));
}
