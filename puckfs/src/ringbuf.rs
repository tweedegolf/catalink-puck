use super::UdpBuffer;

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Error;

pub struct Buffer<const N: usize> {
    r: usize,
    w: usize,
    entry: [(UdpBuffer, Instant); N],
}

impl<const N: usize> Buffer<N> {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            r: 0,
            w: 0,
            entry: std::array::from_fn(|_| (UdpBuffer::new(), now)),
        }
    }

    pub fn len(&self) -> usize {
        self.w - self.r
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_full(&self) -> bool {
        self.len() == N
    }

    pub fn count(&self) -> u32 {
        self.w as u32
    }

    pub fn push(&mut self, buf: UdpBuffer) -> Result<(), Error> {
        if self.is_full() {
            return Err(Error);
        }

        let idx = self.w % N;
        self.entry[idx] = (buf, Instant::now());
        self.w += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Result<UdpBuffer, Error> {
        if self.is_empty() {
            return Err(Error);
        }

        let idx = self.r % N;
        let buf = std::mem::take(&mut self.entry[idx].0);
        self.r += 1;

        Ok(buf)
    }

    pub fn staleness(&mut self) -> Result<Duration, Error> {
        if self.is_empty() {
            return Err(Error);
        }

        let idx = self.r % N;
        let age = Instant::now().duration_since(self.entry[idx].1);

        Ok(age)
    }

    pub fn queued(&mut self) -> impl Iterator<Item = UdpBuffer> {
        BufferIterator(self)
    }
}

struct BufferIterator<'a, const N: usize>(&'a mut Buffer<N>);

impl<'a, const N: usize> Iterator for BufferIterator<'a, N> {
    type Item = UdpBuffer;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_and_drain() {
        let mut ringbuf = Buffer::<10>::new();
        for i in 1..=10 {
            let buf = [i].into_iter().collect();
            ringbuf.push(buf).unwrap();
        }
        let eleven = [11].into_iter().collect();
        assert!(ringbuf.push(eleven).is_err());
        assert_eq!(ringbuf.pop().unwrap()[0], 1);
        let eleven = [11].into_iter().collect();
        assert!(ringbuf.push(eleven).is_ok());
        assert!(ringbuf.staleness().unwrap() >= std::time::Duration::from_millis(0));
        assert!(ringbuf.staleness().unwrap() < std::time::Duration::from_millis(5000));
        let mut buf = vec![];
        for item in ringbuf.queued() {
            buf.extend_from_slice(&item);
        }
        assert_eq!(&[2u8, 3, 4, 5, 6, 7, 8, 9, 10, 11][..], &buf);
    }
}
