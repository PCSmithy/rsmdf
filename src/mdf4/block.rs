pub trait Block {
    fn read(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String>
    where
        Self: Sized;

    fn read_at(bytes: &[u8], pos: usize, little_endian: bool) -> Result<(usize, Self), String>
    where
        Self: Sized,
    {
        Self::read(bytes, pos, little_endian)
    }

    fn byte_len(&self) -> usize;
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
