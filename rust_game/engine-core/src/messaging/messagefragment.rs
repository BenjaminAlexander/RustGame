pub const FRAGMENT_HEADER_SIZE: usize = 8;
const ID_INDEX: usize = 0;
const INDEX_INDEX: usize = 4;
const COUNT_INDEX: usize = 6;

pub struct MessageFragment {
    buf: Vec<u8>,
}

impl MessageFragment {
    pub fn new(id: u32, index: u16, count: u16, mut buf: Vec<u8>) -> Self {
        let mut fragment: Vec<u8> = Vec::with_capacity(buf.len() + FRAGMENT_HEADER_SIZE);
        fragment.append(&mut id.to_be_bytes().to_vec());
        fragment.append(&mut index.to_be_bytes().to_vec());
        fragment.append(&mut count.to_be_bytes().to_vec());
        fragment.append(&mut buf);

        return Self { buf: fragment };
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        return Self { buf };
    }

    pub fn get_id(&self) -> u32 {
        let array: [u8; 4] = [
            self.buf[ID_INDEX],
            self.buf[ID_INDEX + 1],
            self.buf[ID_INDEX + 2],
            self.buf[ID_INDEX + 3],
        ];
        return u32::from_be_bytes(array);
    }

    pub fn get_count(&self) -> u16 {
        let array: [u8; 2] = [self.buf[COUNT_INDEX], self.buf[COUNT_INDEX + 1]];
        return u16::from_be_bytes(array);
    }

    pub fn get_index(&self) -> u16 {
        let array: [u8; 2] = [self.buf[INDEX_INDEX], self.buf[INDEX_INDEX + 1]];
        return u16::from_be_bytes(array);
    }

    pub fn get_fragment_length(&self) -> usize {
        return self.buf.len() - FRAGMENT_HEADER_SIZE;
    }

    pub fn move_buf(self) -> Vec<u8> {
        return self.buf[FRAGMENT_HEADER_SIZE..self.buf.len()].to_vec();
    }

    pub fn get_whole_buf(&self) -> &Vec<u8> {
        return &self.buf;
    }
}
