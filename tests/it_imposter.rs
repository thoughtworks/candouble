extern crate candouble;

use candouble::can::{CANMessage, CANAdaptor};
use candouble::imposter::Imposter;
use candouble::imposter;
use candouble::controller::ImposterList;


struct MockAdaptor {
    incoming_message: Option<CANMessage>,
    sent_message: Option<CANMessage>
}

impl CANAdaptor for MockAdaptor {
    fn receive(&mut self) -> Result<CANMessage, &'static str> {
        if let Some(message) = self.incoming_message {
            self.incoming_message = None;
            return Ok(message);
        }
        Err("no more messages")
    }

    fn send(&mut self, message: &CANMessage) -> Result<(), &'static str> {
        self.sent_message = Some(message.clone());
        Ok(())
    }
}


#[test]
fn it_stub_matches_when_all_predicates_are_true() {
    let imposter = Imposter::from_file("tests/it_imposter.json");
    let list = ImposterList::new();
    list.upsert(imposter);

    let message = CANMessage::with_content(0x0101, 0, &[0xCA, 0xFE]);
    let mut adaptor = MockAdaptor { incoming_message: Some(message), sent_message: None };

    imposter::run_with_adaptor(123, list, &mut adaptor);

    assert_eq!(true, adaptor.sent_message.is_some());
    assert_eq!(0x102, adaptor.sent_message.unwrap().id);
}

#[test]
fn it_stub_does_not_match_when_at_least_one_predicate_is_false() {
    let imposter = Imposter::from_file("tests/it_imposter.json");
    let list = ImposterList::new();
    list.upsert(imposter);

    let message = CANMessage::with_content(0x0101, 0, &[0x00, 0x00]);
    let mut adaptor = MockAdaptor { incoming_message: Some(message), sent_message: None };

    imposter::run_with_adaptor(123, list, &mut adaptor);

    assert_eq!(true, adaptor.sent_message.is_none());
}
