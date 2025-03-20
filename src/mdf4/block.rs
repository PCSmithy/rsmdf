pub trait Block {
    fn new() -> Self;
    fn default() -> Self;
    fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String>
    where
        Self: Sized;
    fn read_at(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String>
    where
        Self: Sized;
    fn byte_len(&self) -> usize;
    //fn is_empty(&self) -> bool;
}

pub trait LinkedBlock {
    fn next(&self, stream: &[u8], little_endian: bool) -> Option<Self>
    where
        Self: std::marker::Sized;
    fn list(&self, stream: &[u8], little_endian: bool) -> Vec<Self>
    where
        Self: std::marker::Sized;
}

pub trait DataBlock {
    fn data_array(&self, stream: &[u8], little_endian: bool) -> Vec<u8>;
}
