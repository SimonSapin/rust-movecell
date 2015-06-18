use std::cell::UnsafeCell;
use std::fmt;
use std::mem;
use std::ops;
use std::ptr;

/// A container similar to [`std::cell::Cell`](http://doc.rust-lang.org/std/cell/struct.Cell.html),
/// but that also supports not-implicitly-copyable types.
pub struct MoveCell<T>(UnsafeCell<T>);


impl<T> MoveCell<T> {
    /// Create a new `MoveCell` containing the given value.
    #[inline]
    pub fn new(value: T) -> MoveCell<T> {
        MoveCell(UnsafeCell::new(value))
    }

    /// Consume the `MoveCell` and return the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        unsafe { self.0.into_inner() }
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

    /// Take the value, and return it in a `Borrow` guard that will return it when dropped.
    /// The cell’s contents are set to the default value until the guard is dropped.
    #[inline]
    pub fn borrow(&self) -> Borrow<T> {
        Borrow {
            _cell: self,
            _value: self.take()
        }
    }
}

/// The cell’s contents are temporarily set to the default value during the clone.
impl<T: Default + Clone> Clone for MoveCell<T> {
    #[inline]
    fn clone(&self) -> MoveCell<T> {
        MoveCell::new(self.borrow().clone())
    }
}

/// The cell’s contents are temporarily set to the default value during the formatting.
impl<T: Default + fmt::Debug> fmt::Debug for MoveCell<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MoveCell({:?})", *self.borrow())
    }
}

/// The cell’s contents are temporarily set to the default value during the comparaison.
impl<T: Default + Eq> Eq for MoveCell<T> {}

/// The cell’s contents are temporarily set to the default value during the comparaison.
impl<T: Default + PartialEq> PartialEq for MoveCell<T> {
    #[inline]
    fn eq(&self, other: &MoveCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }

    #[inline]
    fn ne(&self, other: &MoveCell<T>) -> bool {
        *self.borrow() != *other.borrow()
    }
}

/// A wrapper for a value "borrowed" from a `MoveCell`.
/// When the wrapper is dropped, the value is returned to the cell automatically.
pub struct Borrow<'a, T: 'a> {
    _cell: &'a MoveCell<T>,
    _value: T,
}

// Borrow intentionally does *not* implement Clone
// so that borrow.clone() clones the inner value through auto-deref.

impl<'a, T> Borrow<'a, T> {
    /// Consume the `Borrow` guard and return the value.
    pub fn into_inner(self) -> T {
        let value = unsafe { ptr::read(&self._value) };
        mem::forget(self);
        value
    }
}

impl<'a, T> Drop for Borrow<'a, T> {
    fn drop(&mut self) {
        // FIXME: make self._value a `ManuallyDrop` when that exists.
        // https://github.com/rust-lang/rfcs/pull/197
        mem::swap(&mut self._value, unsafe { &mut *self._cell.as_unsafe_cell().get() })
    }
}

impl<'a, T> ops::Deref for Borrow<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T { &self._value }
}
impl<'a, T> ops::DerefMut for Borrow<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T { &mut self._value }
}

impl<'a, T: fmt::Debug> fmt::Debug for Borrow<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "movecell::Borrow({:?})", self._value)
    }
}


#[test]
fn it_works() {
    let x = MoveCell::new("first".to_owned());
    assert_eq!(x.replace("second".to_owned()), "first");
    assert_eq!(x.replace("third".to_owned()), "second");
    assert_eq!(x.into_inner(), "third");

    let x = MoveCell::new(Some("fourth".to_owned()));
    assert_eq!(x.take(), Some("fourth".to_owned()));
    assert_eq!(x.take(), None);
    assert_eq!(x.replace(Some("fifth".to_owned())), None);
    assert_eq!(&*x.borrow(), &Some("fifth".to_owned()));
    assert_eq!(x.borrow().as_ref().map(|s| s.len()), Some(5));
    assert_eq!(x.borrow().clone(), Some("fifth".to_owned()));
    assert_eq!(x.borrow().is_some(), true);
    assert_eq!(x.borrow().is_none(), false);
    assert_eq!(x.clone(), x);
    assert_eq!(format!("{:?}", x), "MoveCell(Some(\"fifth\"))");
    assert_eq!(format!("{:?}", x.borrow()), "movecell::Borrow(Some(\"fifth\"))");
    assert_eq!(x.take(), Some("fifth".to_owned()));
    assert_eq!(x.borrow().is_some(), false);
    assert_eq!(x.borrow().is_none(), true);
    assert_eq!(&*x.borrow(), &None);
    assert_eq!(x.clone(), x);
    assert_eq!(format!("{:?}", x), "MoveCell(None)");
}
