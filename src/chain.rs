use std::marker::PhantomData;
use std::ptr;

pub struct BufferChainer<'a, T: Sized> {
    buffers: Vec<&'a [u8]>,
    buffer_index: usize,
    byte_offset: usize,
    next_fn: fn(&T) -> usize,
    _phantom: PhantomData<T>
}

impl<'a, T: Sized> BufferChainer<'a, T> {
    pub fn new(buffers: Vec<&'a [u8]>, next_fn: fn(&T) -> usize) -> Self {
        println!("buffers len is {}", buffers.len());
        Self { buffers, buffer_index: 0, byte_offset: 0, next_fn, _phantom: PhantomData }
    }

    pub fn read_all(&self) -> Vec<u8> {
        self.buffers.concat()
    }
}

impl<'a, T: Copy> Iterator for BufferChainer<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(buf) = self.buffers.get(self.buffer_index) {
            let remaining = buf.len().saturating_sub(self.byte_offset);

            if remaining < size_of::<T>() {
                self.buffer_index += 1;
                self.byte_offset = 0;
                continue;
            }

            let ptr = unsafe { buf.as_ptr().add(self.byte_offset) as *const T };
            let value = unsafe { ptr::read_unaligned(ptr) };

            self.byte_offset += (self.next_fn)(&value);
            return Some(value);
        }
        None
    }
}
