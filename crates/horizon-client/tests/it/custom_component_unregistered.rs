extern crate horizon_client_lib as horizon_client;
use horizon_protocol::{
    buf::{Codec, Reader, Writer},
    item::Slot,
};

const FIRST_CUSTOM_ID: i32 = 122;

#[test]
#[ignore = "daily"]
fn unregistered_custom_component_id_is_unknown() {
    if crate::in_own_process(
        "custom_component_unregistered::unregistered_custom_component_id_is_unknown",
    ) {
        return;
    }
    let mut bytes = Writer::new();
    bytes.varint(1);
    bytes.varint(0);
    bytes.varint(1);
    bytes.varint(0);
    bytes.varint(FIRST_CUSTOM_ID);
    let error = Slot::read(&mut Reader::new(&bytes.data)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains(&format!("unsupported item component {FIRST_CUSTOM_ID}"))
    );
}
