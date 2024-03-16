use std::{
    fmt::{Debug, Display},
    num::NonZeroUsize,
    ptr::{self, NonNull},
};

/// thin.
/// ```text
/// 000000 ... 000000 0000000
///                         ^-always 1 for niche
///                        ^- tag, 1 for inline u62, 0 for box
/// ```
pub struct ThinU128(NonNull<u128>);

enum Repr {
    Inline(u128),
    Boxed(NonNull<u128>),
}

const USIZE_TWO_BIT_LESS_MAX: u128 = (usize::MAX as u128) >> 2;

const ALWAYS_ONE_NICHE: usize = 0b1;
const TAG_MASK: usize = 0b10;

impl ThinU128 {
    pub fn new(int: u128) -> Self {
        if int > USIZE_TWO_BIT_LESS_MAX {
            let ptr = Box::into_raw(Box::new(int));
            let repr = ptr.map_addr(|addr| addr | ALWAYS_ONE_NICHE);
            unsafe { Self(NonNull::new_unchecked(repr)) }
        } else {
            let value = (int as usize) << 2;
            let repr = value | TAG_MASK | ALWAYS_ONE_NICHE;
            Self(NonNull::new(ptr::without_provenance_mut(repr)).unwrap())
        }
    }

    fn is_inline(&self) -> bool {
        (self.addr() & TAG_MASK) != 0
    }

    fn addr(&self) -> usize {
        self.0.addr().get()
    }

    fn repr(&self) -> Repr {
        if self.is_inline() {
            let value = self.addr() >> 2;
            Repr::Inline(value as u128)
        } else {
            let ptr = self.0.map_addr(|addr| unsafe {
                NonZeroUsize::new_unchecked(addr.get() & !ALWAYS_ONE_NICHE)
            });
            Repr::Boxed(ptr)
        }
    }

    pub fn value(&self) -> u128 {
        match self.repr() {
            Repr::Inline(value) => value,
            Repr::Boxed(ptr) => unsafe { ptr.as_ptr().read() },
        }
    }
}

impl Drop for ThinU128 {
    fn drop(&mut self) {
        if let Repr::Boxed(ptr) = self.repr() {
            unsafe {
                drop(Box::from_raw(ptr.as_ptr()));
            }
        }
    }
}

impl Debug for ThinU128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.value(), f)
    }
}

impl Display for ThinU128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value(), f)
    }
}

impl PartialEq for ThinU128 {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(&other.value())
    }
}

impl Eq for ThinU128 {}

impl PartialOrd for ThinU128 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value().partial_cmp(&other.value())
    }
}

impl Ord for ThinU128 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl Clone for ThinU128 {
    fn clone(&self) -> Self {
        Self::new(self.value())
    }
}

unsafe impl Send for ThinU128 {}
unsafe impl Sync for ThinU128 {}

#[cfg(test)]
mod tests {
    use super::ThinU128;

    fn roundtrip(a: u128) {
        let thin = ThinU128::new(a);
        assert_eq!(thin.value(), a);

        let other = ThinU128::new(a);
        assert_eq!(thin, other);
        let dbg_a = format!("{a:?}{a}");
        let dbg_thin = format!("{thin:?}{thin}");
        assert_eq!(dbg_a, dbg_thin);
    }

    #[test]
    fn small() {
        roundtrip(0);
        roundtrip(1);
        roundtrip(100);
        roundtrip((usize::MAX >> 2) as u128);
    }

    #[test]
    fn big() {
        roundtrip(((usize::MAX >> 2) as u128) + 1);
        roundtrip(usize::MAX as u128);
        roundtrip(u128::MAX);
    }
}
