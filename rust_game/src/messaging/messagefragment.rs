use serde::{Deserialize, Serialize};
use log::info;

const FRAGMENT_HEADER_SIZE: usize = 13;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageFragment {
    id: u32,
    index: u16,
    count: u16,
    buf: Vec<u8>
}

impl MessageFragment {

    pub fn make_fragments(id: u32, buf: Vec<u8>, max_datagram_size: usize) -> Vec<Self> {
        let fragment_payload_size = max_datagram_size - FRAGMENT_HEADER_SIZE;
        let number_of_fragments = buf.len() / fragment_payload_size;

        let mut fragments = Vec::new();

        for i in 0..number_of_fragments {
            let start = i * fragment_payload_size;
            let end = (start + fragment_payload_size).min(buf.len());

            let fragment = Self::new(
                id,
                i as u16,
                number_of_fragments as u16,
                buf[start..end].to_vec()
            );

            fragments.push(fragment);
        }

        return fragments;
    }

    pub fn new (id: u32, index: u16, count: u16, buf: Vec<u8>) -> Self {

        return Self {
            id,
            index,
            count,
            buf
        };
    }
}