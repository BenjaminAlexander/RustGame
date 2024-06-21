use crate::messaging::messagefragment::FRAGMENT_HEADER_SIZE;
use crate::messaging::MessageFragment;

pub struct Fragmenter {
    next_id: u32,
    max_datagram_size: usize,
}

impl Fragmenter {
    pub fn new(max_datagram_size: usize) -> Self {
        return Self {
            next_id: 0,
            max_datagram_size,
        };
    }

    pub fn make_fragments(&mut self, buf: Vec<u8>) -> Vec<MessageFragment> {
        let id = self.next_id;

        if self.next_id == u32::max_value() {
            self.next_id = 0;
        } else {
            self.next_id = self.next_id + 1;
        }

        let fragment_payload_size = self.max_datagram_size - FRAGMENT_HEADER_SIZE;
        let mut number_of_fragments = buf.len() / fragment_payload_size;

        if buf.len() % fragment_payload_size != 0 {
            number_of_fragments = number_of_fragments + 1;
        }

        let mut fragments = Vec::new();

        for i in 0..number_of_fragments {
            let start = i * fragment_payload_size;
            let end = (start + fragment_payload_size).min(buf.len());

            let fragment_buf = buf[start..end].to_vec().clone();

            let fragment =
                MessageFragment::new(id, i as u16, number_of_fragments as u16, fragment_buf);

            fragments.push(fragment);
        }

        return fragments;
    }
}
