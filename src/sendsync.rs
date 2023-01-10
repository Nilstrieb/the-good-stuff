use std::{
    cell::{Cell, UnsafeCell},
    rc::Rc,
    sync::Arc,
};

// Today we want to design the safe `std::thread::spawn` function and the traits around that.

// First we have the following signature. <maybe start without 'static?>

pub fn spawn<F: FnOnce() + 'static>(f: F) {
    // SAFETY: Well, that's what we're here for today.
    unsafe { magic_unchecked_spawn_for_our_convenience(f) }
}

#[test]
fn send_over_integer() {
    // This is perfectly safe. No data is shared. Our function allows this, which is very nice.

    let x = 0;
    spawn(move || drop(dbg!(x)));
}

#[test]
fn rc_the_new_contender() {
    // Now, let's send over a more complex type like an Rc.
    let x = Rc::new(0);
    let x2 = x.clone();
    spawn(move || {
        x2.clone();
    });
    x.clone(); // DATA RACE
}

// Oh no, we have a data race. This is not exactly good, in fact it's really bad.
// So, how can we forbid Rc from being sent over?
// We need some kind of "this can be sent to other threads" trait. Let's call it "Send".
pub unsafe auto trait Send {}
// It's an auto trait because we really don't want everyone having to implement this manually.
// It's also unsafe because the safety of our spawn function relies on it.

// Why exactly was Rc able to trigger a data race here? The key lies in interior mutability.
// Interior mutability like Cells but also raw pointers should therefore be forbidden by default.
impl<T> !Send for *const T {}
impl<T> !Send for *mut T {}
impl<T> !Send for UnsafeCell<T> {}

// When we now add a F: Send bound to our spawn function, the Rc stops cinoukubgè

#[test]
fn but_arc_is_fine() {
    // Now, let's send over a more complex type like an Rc.
    let x = Arc::new(0);
    let x2 = x.clone();
    spawn(move || {
        x2.clone();
    });
    x.clone();
}

// Arc is fine here because it uses atomics internally. But it fails to compile! Here, Arc (or us in this case)
// needs to assert that it's fine:
unsafe impl<T> Send for Arc<T> {}

// So now, everything is good.

#[test]
fn an_arc_of_sadness() {
    let x = Arc::new(Cell::new(0));
    let x2 = x.clone();
    spawn(move || {
        x2.set(0);
    });
    x.set(1); // DATA RACE
}

// Oh, not quite. We have an issue. Luckily it's a simple one, we just forgot to put a `T: Send` bound
// on the impl.

// unsafe impl<T: Send> Send for Arc<T> {}

// After we fix this, it fails to compile as desired.

#[test]
fn i_am_just_sending_over_a_cell() {
    // We just send the Cell over and only ever use it from the other thread.
    // This is perfectly sound. We want to allow this.

    let x = Cell::new(0);
    spawn(move || {
        let x = x;
        x.set(1)
    });
}

// The example above fails to compile. But there is no unsoundness here, we want to allow this.
// But as we've seen above, we cannot make `Cell: Send`.

// Really, we have two concepts at play here
// - Something that we can send owned top a thread.
// - Something that we can send a reference of to another thread
// Rc can support neither of those, as its possibly unsoundness (clone) can be triggered just
// with a shared reference to it, but also with an owned Rc because two owned Rcs can point to the same memory.
// Cell is different. Having a &Cell across threads can lead to the data race. But having an owned Cell cannot
// trigger the unsoundness, as it will just mutate the local value.

// Let's add a new trait for types that support being shared behind a reference.

pub unsafe auto trait Sync {}

// UnsafeCell is the key here and will make sure that types like Cell are !Sync.
impl<T> !Sync for UnsafeCell<T> {}

// Also forbid pointers to make sure that unsafe datastructures have to manually assert syncness.
impl<T> !Sync for *const T {}
impl<T> !Sync for *mut T {}

// Now we can actually implement Send for UnsafeCell again. Sending a Cell-like type to another thread
// is not problematic, only sharing it is.
// -impl<T> !Send for UnsafeCell<T> {}

// Now we just need one last piece, the interactions of Send and Sync.
// Sync means that we can share a reference across a thread, so let's represent that in an impl.
unsafe impl<T: Sync> Send for &T {}

// The same "reference like behavior" applies to Arc. We are only allowed to Send an Arc to another thread
// if the thing it holds is Sync. Arc<Cell<u8>> is therefore not Send, as this type is not thread-safe.
// unsafe impl<T: Sync> Send for Arc<T> {}

// In general, anything that provides shared access to T needs a T: Sync bound on its Send impl.

// Bonus: The cursed impl of magic_unchecked_spawn_for_our_convenience.
pub unsafe fn magic_unchecked_spawn_for_our_convenience<F: FnOnce()>(f: F) {
    // Pretend that we're Send.
    struct AssertSend<T>(T);
    unsafe impl<T> std::marker::Send for AssertSend<T> {}

    // Get over the annoying 'static requirement by just sending over an erased pointer and reading from it.
    let s = Box::into_raw(Box::new(f));
    let p = AssertSend(s.cast::<()>());
    std::thread::spawn(|| {
        let p = unsafe { Box::from_raw({ p }.0 as *mut F) };
        (p)();
    });
}
