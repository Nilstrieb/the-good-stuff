use uwu::scratch::{
    actual_scratch_read, actual_scratch_write, define_scratch, scratch_space, Scratch,
};

#[scratch_space]
fn has_scratch_space(mut scratch: Scratch<'_>) {
    scratch_write!(scratch, 10u32);
    let _: u32 = scratch_read!(scratch);
}

fn main() {
    define_scratch!(scratch, 10);
    has_scratch_space(scratch);
}
