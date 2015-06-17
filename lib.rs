use std::cell::UnsafeCell;
use std::mem;

/// A container similar to [`std::cell::Cell`](http://doc.rust-lang.org/std/cell/struct.Cell.html),
/// but that also supports not-implicitly-copyable types.
pub struct MoveCell<T>(UnsafeCell<T>);


impl<T> MoveCell<T> {
    /// Create a new `MoveCell` containing the given value.
    #[inline]
    pub fn new(value: T) -> MoveCell<T> {
        MoveCell(UnsafeCell::new(value))
    }

    /// Return the inner value after replacing it with the given value.
    #[inline]
    pub fn replace(&self, new_value: T) -> T {
        unsafe {
            mem::replace(&mut *self.0.get(), new_value)
        }
    }
}


#[test]
fn it_works() {
    let x = MoveCell::new("first".to_owned());
    assert_eq!(x.replace("second".to_owned()), "first");
    assert_eq!(x.replace("third".to_owned()), "second");
}
