use std::mem::{self, MaybeUninit};

pub use pm::scratch_space;

pub struct Scratch<'a>(&'a mut [MaybeUninit<u8>]);

impl<'a> Scratch<'a> {
    pub fn new(buf: &'a mut [MaybeUninit<u8>]) -> Self {
        Self(buf)
    }

    pub fn write<T>(&mut self, _value: T) {
        let size = mem::size_of::<T>();
        assert!(size <= self.0.len());
    }

    pub fn read<T: Default>(&mut self) -> T {
        T::default()
    }
}

#[macro_export]
macro_rules! scratch_write {
    ($scratch:ident, $value:expr) => {
        /* transformed to a call to actual_scratch_write */
        compile_error!("Failed to transform macro invocation");
    };
}

#[macro_export]
macro_rules! scratch_read {
    ($scratch:ident) => {
        /* transformed to a call to actual_scratch_write */
        compile_error!("Failed to transform macro invocation");
    };
}

#[macro_export]
macro_rules! actual_scratch_write {
    ($scratch:ident, $value:expr ; $track_local:ident) => {
        $track_local = ();
        $scratch.write($value);
    };
}

#[macro_export]
macro_rules! actual_scratch_read {
    ($scratch:ident ; $track_local:ident) => {{
        let _read = $track_local;
        $scratch.read()
    }};
}

#[macro_export]
macro_rules! define_scratch {
    ($name:ident, $size:expr) => {
        let mut __buffer: [::core::mem::MaybeUninit<u8>; $size] =
            unsafe { ::core::mem::MaybeUninit::uninit().assume_init() };
        #[allow(unused_mut)]
        let mut $name = $crate::scratch::Scratch::new(&mut __buffer);
    };
}

pub use {actual_scratch_read, actual_scratch_write, define_scratch, scratch_read, scratch_write};

#[cfg(test)]
mod tests {
    use pm::scratch_space;

    use super::Scratch;

    #[scratch_space]
    fn has_scratch_space(mut scratch: Scratch<'_>) {
        scratch_write!(scratch, 10u32);
        let _: u32 = scratch_read!(scratch);
    }

    #[test]
    fn simple_scratch() {
        define_scratch!(scratch, 100);
        has_scratch_space(scratch);
    }
}
