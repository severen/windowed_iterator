// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This crate implements a sliding window iterator adapter.
//!
//! For an explanation of what a "sliding window" *is* and usage directions,
//! see the [`windows`](trait.WindowedIterator.html#method.windows) method on
//! [`WindowedIterator`](trait.WindowedIterator.html).

use std::collections::VecDeque;

/// A sliding window iterator adapter.
///
/// This `struct` is created by the
/// [`windows`](trait.WindowedIterator.html#method.windows) method on
/// [`WindowedIterator`](trait.WindowedIterator.html). See its documentation
/// for more information.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Windows<I: Iterator<Item = T>, T: Clone> {
  /// The adapted iterator.
  iter: I,
  /// The window.
  window: VecDeque<T>,
  /// The number of elements in the window.
  window_size: usize,
}

impl<I: Iterator<Item = T>, T: Clone> Windows<I, T> {
  /// Create a new instance of [`Windows`](struct.Windows.html) where the
  /// number of elements in the window is given with `window_size`.
  fn new(iter: I, window_size: usize) -> Self {
    Windows {
      iter,
      window: VecDeque::with_capacity(window_size),
      window_size,
    }
  }
}

impl<I: Iterator<Item = T>, T: Clone> Iterator for Windows<I, T> {
  type Item = VecDeque<T>;

  fn next(&mut self) -> Option<VecDeque<T>> {
    if self.window_size == 0 {
      return None;
    }

    if self.window.len() == self.window_size {
      self.window.pop_front();
    }

    while self.window.len() < self.window_size {
      match self.iter.next() {
        Some(elem) => self.window.push_back(elem),
        None => return None,
      }
    }

    // Unfortunately, cloning the window for each iteration makes things less
    // efficient when the elements are not cheap to clone, but is required to
    // avoid lifetime issues.
    Some(self.window.clone())
  }
}

/// An `Iterator` blanket implementation that provides
/// the [`windows`](trait.WindowedIterator.html#method.windows) method for
/// creating an instance of the [`Windows`](struct.Windows.html) iterator
/// adapter.
///
/// This trait is automatically implemented by anything that implements
/// `IntoIterator`, and therefore anything that implements `Iterator`.
pub trait WindowedIterator<I: Iterator<Item = T>, T: Clone> {
  /// Return a "sliding window" iterator over all contiguous windows of the
  /// size given by `window_size`. This can be viewed as a window into the
  /// collection's elements that slides from the start to the end, hence the
  /// name.
  ///
  /// # Behaviour to Note
  ///
  /// * The windows overlap.
  /// * The windows are a `VecDeque` in order to allow for a flexible window
  ///   size at runtime. This has the trade off of being less ergonomic than
  ///   using a tuple, so if the window sizes do not need to vary Itertool's
  ///   `tuple_windows` method may be more suitable.
  /// * A window size of 0 will yield an empty iterator.
  /// * A window size that is greater than the amount of elements in the
  ///   iterator will yield an empty iterator.
  /// * The elements inside the iterator are cloned so that they can be part of
  ///   successive windows, making this iterator most suited for iterators of
  ///   references and other values that are cheap to clone.
  ///
  /// # Examples
  ///
  /// Regular usage:
  ///
  /// ```
  /// # fn main() {
  /// # use windowed_iterator::WindowedIterator;
  /// let words = vec!["These", "are", "a", "bunch", "of", "words"];
  /// let mut iter = words.windows(3);
  ///
  /// assert_eq!(iter.next().unwrap(), ["These", "are", "a"]);
  /// assert_eq!(iter.next().unwrap(), ["are", "a", "bunch"]);
  /// assert_eq!(iter.next().unwrap(), ["a", "bunch", "of"]);
  /// assert_eq!(iter.next().unwrap(), ["bunch", "of", "words"]);
  /// assert!(iter.next().is_none());
  /// # }
  /// ```
  ///
  /// If the collection is smaller than `window_size`:
  ///
  /// ```
  /// # fn main() {
  /// # use windowed_iterator::WindowedIterator;
  /// let primes = vec![2, 3, 5, 7];
  /// let mut iter = primes.windows(10);
  ///
  /// assert!(iter.next().is_none());
  /// # }
  /// ```
  fn windows(self, window_size: usize) -> Windows<I, T>;
}

impl<I: IntoIterator<Item = T>, T: Clone> WindowedIterator<I::IntoIter, T> for I {
  fn windows(self, window_size: usize) -> Windows<I::IntoIter, T> {
    Windows::new(self.into_iter(), window_size)
  }
}

#[cfg(test)]
mod tests {
  use proptest::prelude::*;

  use super::*;

  proptest! {
    // The the window size is restricted to the range of a u16, in order to
    // keep the run time of this test reasonable.
    #[test]
    fn test_random_window_size(x: Vec<isize>, size: u16) {
      for window in x.windows(size as usize) {
        assert_ne!(window.len(), 0);
      }
    }

    #[test]
    fn test_empty_window(x: Vec<isize>) {
      assert!(x.iter().windows(0).next().is_none());
    }
  }
}
