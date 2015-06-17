use std::cell::UnsafeCell;
use std::fmt;
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

    /// Returns a reference to the underlying `UnsafeCell`.
    ///
    /// ## Unsafety
    ///
    /// This method is unsafe because `UnsafeCell`'s field is public.
    #[inline]
    pub unsafe fn as_unsafe_cell(&self) -> &UnsafeCell<T> {
        &self.0
    }
}

impl<T: Default> Default for MoveCell<T> {
    #[inline]
    fn default() -> MoveCell<T> {
        MoveCell::new(T::default())
    }
}

/// Convenience methods for when there is a default value.
impl<T: Default> MoveCell<T> {
    /// Return the inner value after replacing it with the default value.
    #[inline]
    pub fn take(&self) -> T {
        self.replace(T::default())
    }

    /// Apply a function to a reference to the inner value.
    /// The cell’s contents are temporarily set to the default value during the call.
    #[inline]
    pub fn peek<U, F>(&self, f: F) -> U where F: FnOnce(&T) -> U {
        let option = self.take();
        let result = f(&option);
        self.replace(option);
        result
    }

    /// Return a clone of the inner optional value.
    /// The cell’s contents are temporarily set to the default value during the clone.
    #[inline]
    pub fn clone_inner(&self) -> T where T: Clone {
        self.peek(Clone::clone)
    }
}

/// The cell’s contents are temporarily set to the default value during the clone.
impl<T: Default + Clone> Clone for MoveCell<T> {
    #[inline]
    fn clone(&self) -> MoveCell<T> {
        MoveCell::new(self.clone_inner())
    }
}

/// The cell’s contents are temporarily set to the default value during the formatting.
impl<T: Default + fmt::Debug> fmt::Debug for MoveCell<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str("MoveCell("));
        try!(self.peek(|value| value.fmt(f)));
        try!(f.write_str(")"));
        Ok(())
    }
}

/// The cell’s contents are temporarily set to the default value during the comparaison.
impl<T: Default + Eq> Eq for MoveCell<T> {}

/// The cell’s contents are temporarily set to the default value during the comparaison.
impl<T: Default + PartialEq> PartialEq for MoveCell<T> {
    #[inline]
    fn eq(&self, other: &MoveCell<T>) -> bool {
        self.peek(|a| other.peek(|b| a == b))
    }

    #[inline]
    fn ne(&self, other: &MoveCell<T>) -> bool {
        self.peek(|a| other.peek(|b| a != b))
    }
}

/// Convenience methods for when the value happens to be an `Option`.
impl<T> MoveCell<Option<T>> {
    /// Apply a function to a reference to the inner value if it is `Some(_)`.
    /// The cell’s contents are temporarily set to `None` during the call.
    #[inline]
    pub fn map_inner<U, F>(&self, f: F) -> Option<U> where F: FnOnce(&T) -> U {
        self.peek(|option| option.as_ref().map(f))
    }

    /// Return whether the inner optional value is `Some(_)`.
    /// The cell’s contents are temporarily set to `None` during the call.
    #[inline]
    pub fn is_some(&self) -> bool {
        self.peek(|option| option.is_some())
    }

    /// Return whether the inner optional value is `None(_)`.
    /// The cell’s contents are temporarily set to `None` during the call.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.peek(|option| option.is_none())
    }
}


#[test]
fn it_works() {
    let x = MoveCell::new("first".to_owned());
    assert_eq!(x.replace("second".to_owned()), "first");
    assert_eq!(x.replace("third".to_owned()), "second");

    let x = MoveCell::new(Some("fourth".to_owned()));
    assert_eq!(x.take(), Some("fourth".to_owned()));
    assert_eq!(x.take(), None);
    assert_eq!(x.replace(Some("fifth".to_owned())), None);
    x.peek(|o| assert_eq!(o, &Some("fifth".to_owned())));
    assert_eq!(x.map_inner(|s| s.len()), Some(5));
    assert_eq!(x.clone_inner(), Some("fifth".to_owned()));
    assert_eq!(x.is_some(), true);
    assert_eq!(x.is_none(), false);
    assert_eq!(x.clone(), x);
    assert_eq!(format!("{:?}", x), "MoveCell(Some(\"fifth\"))");
    assert_eq!(x.take(), Some("fifth".to_owned()));
    assert_eq!(x.is_some(), false);
    assert_eq!(x.is_none(), true);
    x.peek(|o| assert_eq!(o, &None));
    assert_eq!(x.clone(), x);
    assert_eq!(format!("{:?}", x), "MoveCell(None)");
}
