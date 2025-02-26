use super::{Bucket, Entries, IndexSet, Slice};

use allocator_api2::alloc::{Allocator, Global};
use allocator_api2::vec::{self, Vec};
use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::iter::{Chain, FusedIterator};
use core::ops::RangeBounds;
use core::slice::Iter as SliceIter;

impl<'a, T, S, A: Allocator> IntoIterator for &'a IndexSet<T, S, A> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, A: Allocator> IntoIterator for IndexSet<T, S, A> {
    type Item = T;
    type IntoIter = IntoIter<T, A>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.into_entries())
    }
}

/// An iterator over the items of an [`IndexSet`].
///
/// This `struct` is created by the [`IndexSet::iter`] method.
/// See its documentation for more.
pub struct Iter<'a, T> {
    iter: SliceIter<'a, Bucket<T>>,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(entries: &'a [Bucket<T>]) -> Self {
        Self {
            iter: entries.iter(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &'a Slice<T> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    iterator_methods!(Bucket::key_ref);
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    double_ended_iterator_methods!(Bucket::key_ref);
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        Self { iter: [].iter() }
    }
}

/// An owning iterator over the items of an [`IndexSet`].
///
/// This `struct` is created by the [`IndexSet::into_iter`] method
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
#[derive(Clone)]
pub struct IntoIter<T, A: Allocator> {
    iter: vec::IntoIter<Bucket<T>, A>,
}

impl<T, A: Allocator> IntoIter<T, A> {
    pub(super) fn new(entries: Vec<Bucket<T>, A>) -> Self {
        Self {
            iter: entries.into_iter(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<T> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<T, A: Allocator> Iterator for IntoIter<T, A> {
    type Item = T;

    iterator_methods!(Bucket::key);
}

impl<T, A: Allocator> DoubleEndedIterator for IntoIter<T, A> {
    double_ended_iterator_methods!(Bucket::key);
}

impl<T, A: Allocator> ExactSizeIterator for IntoIter<T, A> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, A: Allocator> FusedIterator for IntoIter<T, A> {}

impl<T: fmt::Debug, A: Allocator> fmt::Debug for IntoIter<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
        f.debug_list().entries(iter).finish()
    }
}

impl<T> Default for IntoIter<T, Global> {
    fn default() -> Self {
        Self {
            iter: Vec::new().into_iter(),
        }
    }
}

/// A draining iterator over the items of an [`IndexSet`].
///
/// This `struct` is created by the [`IndexSet::drain`] method.
/// See its documentation for more.
pub struct Drain<'a, T, A: Allocator> {
    iter: vec::Drain<'a, Bucket<T>, A>,
}

impl<'a, T, A: Allocator> Drain<'a, T, A> {
    pub(super) fn new(iter: vec::Drain<'a, Bucket<T>, A>) -> Self {
        Self { iter }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<T> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<T, A: Allocator> Iterator for Drain<'_, T, A> {
    type Item = T;

    iterator_methods!(Bucket::key);
}

impl<T, A: Allocator> DoubleEndedIterator for Drain<'_, T, A> {
    double_ended_iterator_methods!(Bucket::key);
}

impl<T, A: Allocator> ExactSizeIterator for Drain<'_, T, A> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, A: Allocator> FusedIterator for Drain<'_, T, A> {}

impl<T: fmt::Debug, A: Allocator> fmt::Debug for Drain<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
        f.debug_list().entries(iter).finish()
    }
}

/// A lazy iterator producing elements in the difference of [`IndexSet`]s.
///
/// This `struct` is created by the [`IndexSet::difference`] method.
/// See its documentation for more.
pub struct Difference<'a, T, S, A: Allocator> {
    iter: Iter<'a, T>,
    other: &'a IndexSet<T, S, A>,
}

impl<'a, T, S, A: Allocator> Difference<'a, T, S, A> {
    pub(super) fn new<S1, A1: Allocator>(
        set: &'a IndexSet<T, S1, A1>,
        other: &'a IndexSet<T, S, A>,
    ) -> Self {
        Self {
            iter: set.iter(),
            other,
        }
    }
}

impl<'a, T, S, A> Iterator for Difference<'a, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            if !self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<T, S, A> DoubleEndedIterator for Difference<'_, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next_back() {
            if !self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }
}

impl<T, S, A> FusedIterator for Difference<'_, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
}

impl<T, S, A: Allocator> Clone for Difference<'_, T, S, A> {
    fn clone(&self) -> Self {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<T, S, A> fmt::Debug for Difference<'_, T, S, A>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the intersection of [`IndexSet`]s.
///
/// This `struct` is created by the [`IndexSet::intersection`] method.
/// See its documentation for more.
pub struct Intersection<'a, T, S, A: Allocator> {
    iter: Iter<'a, T>,
    other: &'a IndexSet<T, S, A>,
}

impl<'a, T, S, A: Allocator> Intersection<'a, T, S, A> {
    pub(super) fn new<S1, A1: Allocator>(
        set: &'a IndexSet<T, S1, A1>,
        other: &'a IndexSet<T, S, A>,
    ) -> Self {
        Self {
            iter: set.iter(),
            other,
        }
    }
}

impl<'a, T, S, A> Iterator for Intersection<'a, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            if self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<T, S, A> DoubleEndedIterator for Intersection<'_, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next_back() {
            if self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }
}

impl<T, S, A> FusedIterator for Intersection<'_, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
}

impl<T, S, A: Allocator> Clone for Intersection<'_, T, S, A> {
    fn clone(&self) -> Self {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<T, S, A> fmt::Debug for Intersection<'_, T, S, A>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the symmetric difference of [`IndexSet`]s.
///
/// This `struct` is created by the [`IndexSet::symmetric_difference`] method.
/// See its documentation for more.
pub struct SymmetricDifference<'a, T, S1, S2, A1: Allocator, A2: Allocator> {
    iter: Chain<Difference<'a, T, S2, A2>, Difference<'a, T, S1, A1>>,
}

impl<'a, T, S1, S2, A1, A2> SymmetricDifference<'a, T, S1, S2, A1, A2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
    A1: Allocator,
    A2: Allocator,
{
    pub(super) fn new(set1: &'a IndexSet<T, S1, A1>, set2: &'a IndexSet<T, S2, A2>) -> Self {
        let diff1 = set1.difference(set2);
        let diff2 = set2.difference(set1);
        Self {
            iter: diff1.chain(diff2),
        }
    }
}

impl<'a, T, S1, S2, A1, A2> Iterator for SymmetricDifference<'a, T, S1, S2, A1, A2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
    A1: Allocator,
    A2: Allocator,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl<T, S1, S2, A1, A2> DoubleEndedIterator for SymmetricDifference<'_, T, S1, S2, A1, A2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
    A1: Allocator,
    A2: Allocator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<T, S1, S2, A1, A2> FusedIterator for SymmetricDifference<'_, T, S1, S2, A1, A2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
    A1: Allocator,
    A2: Allocator,
{
}

impl<T, S1, S2, A1: Allocator, A2: Allocator> Clone for SymmetricDifference<'_, T, S1, S2, A1, A2> {
    fn clone(&self) -> Self {
        SymmetricDifference {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S1, S2, A1, A2> fmt::Debug for SymmetricDifference<'_, T, S1, S2, A1, A2>
where
    T: fmt::Debug + Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
    A1: Allocator,
    A2: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the union of [`IndexSet`]s.
///
/// This `struct` is created by the [`IndexSet::union`] method.
/// See its documentation for more.
pub struct Union<'a, T, S, A: Allocator> {
    iter: Chain<Iter<'a, T>, Difference<'a, T, S, A>>,
}

impl<'a, T, S, A> Union<'a, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    pub(super) fn new<S2, A2>(set1: &'a IndexSet<T, S, A>, set2: &'a IndexSet<T, S2, A2>) -> Self
    where
        S2: BuildHasher,
        A2: Allocator,
    {
        Self {
            iter: set1.iter().chain(set2.difference(set1)),
        }
    }
}

impl<'a, T, S, A> Iterator for Union<'a, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl<T, S, A> DoubleEndedIterator for Union<'_, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<T, S, A> FusedIterator for Union<'_, T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
}

impl<T, S, A: Allocator> Clone for Union<'_, T, S, A> {
    fn clone(&self) -> Self {
        Union {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S, A> fmt::Debug for Union<'_, T, S, A>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A splicing iterator for `IndexSet`.
///
/// This `struct` is created by [`IndexSet::splice()`].
/// See its documentation for more.
pub struct Splice<'a, I, T, S, A>
where
    I: Iterator<Item = T>,
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    iter: crate::map::Splice<'a, UnitValue<I>, T, (), S, A>,
}

impl<'a, I, T, S, A> Splice<'a, I, T, S, A>
where
    I: Iterator<Item = T>,
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator + Clone,
{
    #[track_caller]
    pub(super) fn new<R>(set: &'a mut IndexSet<T, S, A>, range: R, replace_with: I) -> Self
    where
        R: RangeBounds<usize>,
    {
        Self {
            iter: set.map.splice(range, UnitValue(replace_with)),
        }
    }
}

impl<I, T, S, A> Iterator for Splice<'_, I, T, S, A>
where
    I: Iterator<Item = T>,
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, T, S, A> DoubleEndedIterator for Splice<'_, I, T, S, A>
where
    I: Iterator<Item = T>,
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.iter.next_back()?.0)
    }
}

impl<I, T, S, A> ExactSizeIterator for Splice<'_, I, T, S, A>
where
    I: Iterator<Item = T>,
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, T, S, A> FusedIterator for Splice<'_, I, T, S, A>
where
    I: Iterator<Item = T>,
    T: Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
}

struct UnitValue<I>(I);

impl<I: Iterator> Iterator for UnitValue<I> {
    type Item = (I::Item, ());

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|x| (x, ()))
    }
}

impl<'a, I, T, S, A> fmt::Debug for Splice<'a, I, T, S, A>
where
    I: fmt::Debug + Iterator<Item = T>,
    T: fmt::Debug + Hash + Eq,
    S: BuildHasher,
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.iter, f)
    }
}

impl<I: fmt::Debug> fmt::Debug for UnitValue<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
