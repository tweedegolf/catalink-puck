use std::time::{Duration, Instant};

/// This type signifies that a push to the ringbuffer failed. This allows using the
/// more explicit Result<(), Error> instead of Option<()>,
#[derive(Debug)]
pub struct Error;

/// A type that acts as a 32bit index, but has an automatic conversion to usize.
#[derive(Eq, Ord, PartialOrd, PartialEq, Default, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Index(pub u32);

impl std::ops::Rem<usize> for Index {
    type Output = usize;
    fn rem(self, n: usize) -> usize {
        if usize::BITS >= u32::BITS {
            self.0 as usize % n
        } else {
            (self.0 % n as u32) as usize
        }
    }
}

impl std::ops::AddAssign<u32> for Index {
    fn add_assign(&mut self, n: u32) {
        self.0 += n
    }
}

#[derive(Debug)]
/// A ringbuffer with a capacity of `N` elements of type `T`, that maintains a
/// monotonically increasing count and a timestamp when each elements was last updated.
pub struct Buffer<T, const N: usize> {
    pub r: Index,
    pub w: Index,
    entry: [(T, Instant); N],
}

impl<const N: usize, T: Default> Buffer<T, N> {
    /// Create a new ringbuffer with initial "count" `start`.
    pub fn new(start: u32) -> Self {
        let now = Instant::now();
        Self {
            r: Index(start),
            w: Index(start),
            entry: std::array::from_fn(|_| (T::default(), now)),
        }
    }
}

impl<const N: usize, T> Buffer<T, N> {
    /// Return the amount of elements currently queued in the ring buffer.
    pub fn len(&self) -> usize {
        assert!(self.w >= self.r);

        (self.w.0 - self.r.0)
            .try_into()
            .expect("memory indexing error")
    }

    /// Returns true of the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_full(&self) -> bool {
        self.len() == N
    }

    /// Return the "count" of the next item that will be stored in the buffer.
    pub fn count(&self) -> u32 {
        self.w.0
    }

    /// Clear queued elements with a "count" value up to and including `ackd` from
    /// the ring buffer.
    pub fn mark_clear(&mut self, ackd: u32) {
        use std::cmp::{max, min};
        self.r = max(self.r, min(self.w, Index(ackd + 1)));
    }

    /// Update the time stamp of all elements currently queued in the ring buffer.
    pub fn mark_sent(&mut self) {
        let start = self.r % N;
        let stop = self.w % N;
        let now = Instant::now();
        for i in start..stop {
            let (_, updated) = &mut self.entry[i];
            *updated = now;
        }
    }

    /// Add a new element to the ring buffer. Returns an `Error` if the ring buffer
    /// doesn't have enough capacity.
    pub fn push(&mut self, buf: T) -> Result<(), Error> {
        if self.is_full() {
            return Err(Error);
        }

        let idx = self.w % N;
        self.entry[idx] = (buf, Instant::now());
        self.w += 1;

        Ok(())
    }

    /// Returns true if the oldest item in the ring buffer is older than `timeout`.
    pub fn is_stale(&self, timeout: Duration) -> bool {
        if self.is_empty() {
            return false;
        }

        let idx = self.r % N;

        let (_, updated) = &self.entry[idx];
        let age = Instant::now().duration_since(*updated);

        age > timeout
    }

    /// Without clearing the queued elements, change the "count" of the ring buffer.
    /// This can be used to synchronize counts with a remote ring buffer without losing
    /// packets.
    pub fn reposition(&mut self, count: u32) {
        let diff = count.wrapping_sub(self.w.0);
        self.w.0 = self.w.0.wrapping_add(diff);
        self.r.0 = self.r.0.wrapping_add(diff);
    }
}

impl<const N: usize, T: Clone> Buffer<T, N> {
    /// Provide an iterator that returns the elements currently queued in the ring buffer
    /// together with their "count".
    pub fn queue(&self) -> impl Iterator<Item = (u32, T)> {
        let start = self.r.0;
        let stop = self.w.0;

        (start..stop).map(|i| {
            let (buf, _) = &self.entry[Index(i) % N];
            (i, buf.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_and_drain() {
        let mut ringbuf = Buffer::<u8, 10>::new(0);
        for i in 1..=10 {
            ringbuf.push(i).unwrap();
        }
        assert!(ringbuf.push(11).is_err());
        ringbuf.mark_clear(0);
        assert!(ringbuf.push(11).is_ok());
        assert!(ringbuf.is_stale(std::time::Duration::from_millis(0)));
        assert!(!ringbuf.is_stale(std::time::Duration::from_millis(5000)));
        let buf = ringbuf.queue().map(|(_, x)| x).collect::<Vec<_>>();
        assert_eq!(&buf, &[2u8, 3, 4, 5, 6, 7, 8, 9, 10, 11][..]);
    }
}
