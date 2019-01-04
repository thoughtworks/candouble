extern crate candouble;

use candouble::stub::Stub;
use candouble::can::{CANMessage, CANAdaptor};
use candouble::imposter::Imposter;


struct MockAdaptor {
    recorded_message: Option<CANMessage>
}

impl CANAdaptor for MockAdaptor {
    fn receive(&self) -> Result<CANMessage, &'static str> {
        unimplemented!()
    }

    fn send(&mut self, message: &CANMessage) -> Result<(), &'static str> {
        self.recorded_message = Some(message.clone());
        Ok(())
    }
}


#[test]
fn it_stub_matches_when_all_predicates_are_true() {
    let mut imposter = Imposter::from_file("tests/it_imposter.json");
    let mut adaptor = MockAdaptor { recorded_message: None };
    let message = CANMessage::with_content(0x0101, 0, &[0xCA, 0xFE]);

    imposter.handle_message(&mut adaptor, &message);

    assert_eq!(true, adaptor.recorded_message.is_some());
    assert_eq!(0x102, adaptor.recorded_message.unwrap().id);
}

#[test]
fn it_stub_does_not_match_when_at_least_one_predicate_is_false() {
    let mut imposter = Imposter::from_file("tests/it_imposter.json");
    let mut adaptor = MockAdaptor { recorded_message: None };
    let message = CANMessage::with_content(0x0101, 0, &[0x00, 0x00]);

    imposter.handle_message(&mut adaptor, &message);

    assert_eq!(true, adaptor.recorded_message.is_none());
}
