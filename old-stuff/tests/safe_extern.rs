use uwu::safe_extern::safe_extern;

#[safe_extern]
extern "Rust" {
    fn add(a: u8, b: u8) -> u8;
}

mod _impl {
    #[no_mangle]
    pub(super) fn add(a: u8, b: u8) -> u8 {
        a + b
    }
}

#[test]
fn adding() {
    assert_eq!(add(1, 2), 3);
}
