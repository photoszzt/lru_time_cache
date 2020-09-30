#[cfg(feature = "heapsize")]
extern crate heapsize;

use std::borrow::Borrow;

/// A trait for measuring the size of a cache entry.
///
/// If you implement this trait, you should use `usize` as the `Measure` type, otherwise you will
/// also have to implement [`CountableMeter`][countablemeter].
///
/// [countablemeter]: trait.Meter.html
pub trait Meter<K, V> {
    /// The type used to store measurements.
    type Measure: Default + Copy;
    /// Calculate the size of `key` and `value`.
    fn measure<Q: ?Sized>(&self, key: &Q, value: &V) -> Self::Measure
    where
        K: Borrow<Q>;
}

/// Size limit based on a simple count of cache items.
pub struct Count;

impl<K, V> Meter<K, V> for Count {
    /// Don't store anything, the measurement can be derived from the map.
    type Measure = ();

    /// Don't actually count anything either.
    fn measure<Q: ?Sized>(&self, _: &Q, _: &V)
    where
        K: Borrow<Q>,
    {
    }
}

/// A trait to allow the default `Count` measurement to not store an
/// extraneous counter.
pub trait CountableMeter<K, V>: Meter<K, V> {
    /// Add `amount` to `current` and return the sum.
    fn add(&self, current: Self::Measure, amount: Self::Measure) -> Self::Measure;
    /// Subtract `amount` from `current` and return the difference.
    fn sub(&self, current: Self::Measure, amount: Self::Measure) -> Self::Measure;
    /// Return `current` as a `usize` if possible, otherwise return `None`.
    ///
    /// If this method returns `None` the cache will use the number of cache entries as
    /// its size.
    fn size(&self, current: Self::Measure) -> Option<u64>;
}

/// `Count` is all no-ops, the number of entries in the map is the size.
impl<K, V, T: Meter<K, V>> CountableMeter<K, V> for T
where
    T: CountableMeterWithMeasure<K, V, <T as Meter<K, V>>::Measure>,
{
    fn add(&self, current: Self::Measure, amount: Self::Measure) -> Self::Measure {
        CountableMeterWithMeasure::meter_add(self, current, amount)
    }
    fn sub(&self, current: Self::Measure, amount: Self::Measure) -> Self::Measure {
        CountableMeterWithMeasure::meter_sub(self, current, amount)
    }
    fn size(&self, current: Self::Measure) -> Option<u64> {
        CountableMeterWithMeasure::meter_size(self, current)
    }
}

pub trait CountableMeterWithMeasure<K, V, M> {
    /// Add `amount` to `current` and return the sum.
    fn meter_add(&self, current: M, amount: M) -> M;
    /// Subtract `amount` from `current` and return the difference.
    fn meter_sub(&self, current: M, amount: M) -> M;
    /// Return `current` as a `usize` if possible, otherwise return `None`.
    ///
    /// If this method returns `None` the cache will use the number of cache entries as
    /// its size.
    fn meter_size(&self, current: M) -> Option<u64>;
}

/// For any other `Meter` with `Measure=usize`, just do the simple math.
impl<K, V, T> CountableMeterWithMeasure<K, V, usize> for T
where
    T: Meter<K, V>,
{
    fn meter_add(&self, current: usize, amount: usize) -> usize {
        current + amount
    }
    fn meter_sub(&self, current: usize, amount: usize) -> usize {
        current - amount
    }
    fn meter_size(&self, current: usize) -> Option<u64> {
        Some(current as u64)
    }
}

impl<K, V> CountableMeterWithMeasure<K, V, ()> for Count {
    fn meter_add(&self, _current: (), _amount: ()) {}
    fn meter_sub(&self, _current: (), _amount: ()) {}
    fn meter_size(&self, _current: ()) -> Option<u64> {
        None
    }
}

#[cfg(feature = "heapsize")]
mod heap_meter {
    use heapsize::HeapSizeOf;
    use std::borrow::Borrow;

    /// Size limit based on the heap size of each cache item.
    ///
    /// Requires cache entries that implement [`HeapSizeOf`][1].
    ///
    /// [1]: https://doc.servo.org/heapsize/trait.HeapSizeOf.html
    pub struct HeapSize;

    impl<K, V: HeapSizeOf> super::Meter<K, V> for HeapSize {
        type Measure = usize;

        fn measure<Q: ?Sized>(&self, _: &Q, item: &V) -> usize
        where
            K: Borrow<Q>,
        {
            item.heap_size_of_children() + ::std::mem::size_of::<V>()
        }
    }
}

#[cfg(feature = "heapsize")]
pub use heap_meter::*;
