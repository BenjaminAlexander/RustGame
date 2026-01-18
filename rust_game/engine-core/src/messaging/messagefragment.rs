const ID_INDEX: usize = 0;
const INDEX_INDEX: usize = ID_INDEX + size_of::<u32>();
const COUNT_INDEX: usize = INDEX_INDEX + size_of::<u16>();
pub const FRAGMENT_HEADER_SIZE: usize = COUNT_INDEX + size_of::<u16>();

#[derive(Clone)]
pub struct MessageFragment {

    //TODO: make this borrowed
    buf: Vec<u8>,
}

impl MessageFragment {
    pub fn new(max_datagram_size: usize) -> Self {
        Self {
            buf: Vec::with_capacity(max_datagram_size)
        }
    }

    pub fn write(&mut self, id: u32, index: u16, count: u16, buf: &[u8]) {
        self.buf.clear();
        self.buf.extend_from_slice(&id.to_be_bytes());
        self.buf.extend_from_slice(&index.to_be_bytes());
        self.buf.extend_from_slice(&count.to_be_bytes());
        self.buf.extend_from_slice(buf);
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
