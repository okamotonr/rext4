use std::marker::PhantomData;

pub trait ChainItem: Sized {
    fn from_bytes(input: &[u8]) -> (usize, Self);
}

impl ChainItem for u8 {
    fn from_bytes(input: &[u8]) -> (usize, Self) {
        (1, input[0])
    }
}

pub struct BufferChainer<'a, T: ChainItem> {
    buffers: Vec<&'a [u8]>,
    buffer_index: usize,
    byte_offset: usize,
    _phantom: PhantomData<T>
}

impl<'a, T: ChainItem> BufferChainer<'a, T> {
    pub fn new(buffers: Vec<&'a [u8]>) -> Self {
        println!("buffers len is {}", buffers.len());
        Self { buffers, buffer_index: 0, byte_offset: 0, _phantom: PhantomData }
    }

    pub fn read_all(&self) -> Vec<u8> {
        self.buffers.concat()
    }
}

impl<'a, T: ChainItem> Iterator for BufferChainer<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(buf) = self.buffers.get(self.buffer_index) {
            println!("{}, {}, {}", buf.len(), self.byte_offset, self.buffer_index);
            let remaining = buf.len().saturating_sub(self.byte_offset);
            if remaining < size_of::<T>() {
                self.buffer_index += 1;
                self.byte_offset = 0;
                continue;
            }

            let (offset, value) = T::from_bytes(&buf[self.byte_offset..]);

            self.byte_offset += offset;
            println!("{}, {}, {}", buf.len(), self.byte_offset, self.buffer_index);
            return Some(value);
        }
        None
    }
}
