use serde::{Deserialize, Serialize};
use log::info;

pub const FRAGMENT_HEADER_SIZE: usize = 13;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageFragment {
    id: u32,
    index: u16,
    count: u16,
    buf: Vec<u8>
}

impl MessageFragment {

    pub fn new (id: u32, index: u16, count: u16, buf: Vec<u8>) -> Self {

        return Self {
            id,
            index,
            count,
            buf
        };
    }

    pub fn get_id(&self) -> u32 {
        return self.id;
    }

    pub fn get_count(&self) -> u16 {
        return self.count;
    }

    pub fn get_index(&self) -> u16 {
        return self.index;
    }

    pub fn get_buf(&self) -> &Vec<u8> {
        return &self.buf;
    }

    pub fn move_buf(self) -> Vec<u8> {
        return self.buf;
    }
}