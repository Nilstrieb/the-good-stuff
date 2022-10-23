// TODO: This should probably be fallible instead of panic
// TODO: Needs more safety docs around alignment

use std::{marker::PhantomData, ptr::Pointee};

/// a replacement for Clone (ignoring the old methods)
pub trait NewClone {
    fn clone_unsized<P>(&self, place: ClonePlace<P, Self>) -> InitClonePlace<P, Self>;
}

/// a replacement for Copy
pub trait NewCopy: NewClone {}

/// A trait which denotes a pointer to a place
pub trait Pointer<T: ?Sized> {
    /// Create a pointer from a raw pointer
    /// # Safety
    /// The pointer needs to be valid to create a `Self`. This method can't really be called
    /// generically, but `ClonePlace` provides a safe interface over it.
    unsafe fn from_raw(ptr: *mut T) -> Self;
}

impl<T: ?Sized> Pointer<T> for Box<T> {
    unsafe fn from_raw(ptr: *mut T) -> Self {
        Self::from_raw(ptr)
    }
}

impl<T: ?Sized> Pointer<T> for &mut T {
    unsafe fn from_raw(ptr: *mut T) -> Self {
        &mut *ptr
    }
}

// more impls...

/// Denotes a place which something can be cloned into.
pub struct ClonePlace<P, T: ?Sized> {
    ptr: *mut u8,
    max_size: usize,
    _boo: PhantomData<(P, *const T)>,
}

/// Denotes a place where something has been cloned into successfully.
pub struct InitClonePlace<P, T: ?Sized> {
    ptr: *mut u8,
    metadata: <T as Pointee>::Metadata,
    _boo: PhantomData<P>,
}

impl<P, T: ?Sized> ClonePlace<P, T> {
    /// Get the raw pointer of the place to write things yourself
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get the maximum allocation size of the place
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Create a new ClonePlace from a pointer and maximum size
    /// # Safety
    /// `ptr` has to be valid for writes of size `max_size`
    unsafe fn from_raw(ptr: *mut u8, max_size: usize) -> Self {
        Self {
            ptr,
            max_size,
            _boo: PhantomData,
        }
    }

    /// Unsafely assert that the place has been initialized for as many bytes as covered
    /// by the metadata. This is done by using `as_ptr` and writing to it before
    /// # Safety
    /// `self.ptr` must be valid for reads of at least as much bytes as denoted by the `metadata`
    unsafe fn assert_init_with_meta(
        self,
        metadata: <T as Pointee>::Metadata,
    ) -> InitClonePlace<P, T> {
        InitClonePlace {
            ptr: self.ptr,
            metadata,
            _boo: PhantomData,
        }
    }
}

impl<P, T: ?Sized + NewCopy> ClonePlace<P, T> {
    /// Safe convenience function for implementing Clone via Copy
    pub fn copy_trivially(self, data: &T) -> InitClonePlace<P, T> {
        let size = std::mem::size_of_val(data);
        assert!(self.max_size() >= size);
        // SAFETY: `data` is valid for reads of `sizeof(data)`
        //         `self.ptr` must be writable for at least as many bytes as `self.max_size`, which we just asserted
        //         We have initialized `self.ptr` by `sizeof(data)` bytes, meaning it's fine to assert it as init
        unsafe {
            std::ptr::copy_nonoverlapping(data as *const T as *const u8, self.ptr, size);
            ClonePlace::assert_init_with_meta(self, std::ptr::metadata(data))
        }
    }
}

impl<P: Pointer<T>, T: ?Sized> InitClonePlace<P, T> {
    /// Turn the initialized place into the safe pointer type
    pub fn into_init_value(self) -> P {
        // SAFETY: Our pointer must point to valid initialized data
        //         The way it has been created initially asserts that it's valid for the pointer type or something like that i guess
        unsafe { P::from_raw(std::ptr::from_raw_parts_mut(self.ptr.cast(), self.metadata)) }
    }
}

// convenience function
impl<T: ?Sized> ClonePlace<Box<T>, T> {
    /// Creates a new boxed ClonePlace and allocates as many bytes as required for `value`
    pub fn boxed(value: &T) -> Self {
        // SAFETY: We checked the pointer for null meaning it's valid for `laoyut.size()` bytes
        //         That's the safety requirement for creating a box basically so we're fine
        unsafe {
            let layout = std::alloc::Layout::for_value(value);
            let allocated = std::alloc::alloc(layout);
            if allocated.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Self::from_raw(allocated, layout.size())
        }
    }
}

impl NewClone for str {
    fn clone_unsized<P>(&self, place: ClonePlace<P, Self>) -> InitClonePlace<P, Self> {
        place.copy_trivially(self)
    }
}

impl NewCopy for str {}

#[test]
fn boxit() {
    let str = "aaaa";
    let place = ClonePlace::boxed(str);
    let init_place = str.clone_unsized(place);
    let the_box = init_place.into_init_value();
    assert_eq!(&*the_box, "aaaa");
}

#[test]
fn on_the_stack() {
    let mut storage = [std::mem::MaybeUninit::<u8>::uninit(); 10];
    let str = "aaaa";

    // SAFETY: `storage` is valid for writes of 10 bytes.
    let place: ClonePlace<&mut str, _> =
        unsafe { ClonePlace::from_raw(storage.as_mut_ptr().cast::<u8>(), 10) };

    let init_place = str.clone_unsized(place);
    let the_box = init_place.into_init_value();
    assert_eq!(&*the_box, "aaaa");
}
