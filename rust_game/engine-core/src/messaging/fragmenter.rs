use log::warn;
use rmp_serde::encode::Error;
use serde::Serialize;

use crate::messaging::{
    MessageFragment,
    messagefragment::FRAGMENT_HEADER_SIZE,
};

pub struct Fragmenter {
    next_id: u32,
    max_datagram_size: usize,
    buf: Vec<u8>,
    //fragment_bufs: Vec<Vec<u8>>
}

impl Fragmenter {
    pub fn new(max_datagram_size: usize) -> Self {
        return Self {
            next_id: 0,
            max_datagram_size,
            buf: Vec::with_capacity(max_datagram_size),
            //fragment_bufs: Vec::new()
        };
    }

    pub fn make_fragments<T: Serialize>(&mut self, message: &T) -> Result<Vec<MessageFragment>, Error> {
        self.buf.clear();

        warn!("empty len: {}", self.buf.len());

        rmp_serde::encode::write(&mut self.buf, message)?;

        warn!("write len: {}", self.buf.len());

        let id = self.next_id;

        if self.next_id == u32::max_value() {
            self.next_id = 0;
        } else {
            self.next_id = self.next_id + 1;
        }

        let fragment_payload_size = self.max_datagram_size - FRAGMENT_HEADER_SIZE;
        let mut number_of_fragments = self.buf.len() / fragment_payload_size;

        if self.buf.len() % fragment_payload_size != 0 {
            number_of_fragments = number_of_fragments + 1;
        }

        warn!("len: {}, Num Frags: {}", self.buf.len(), number_of_fragments);

        /* 
        while self.fragment_bufs.len() < number_of_fragments {
            let index = self.fragment_bufs.len();

            self.fragment_bufs.push(Vec::with_capacity(capacity));
        }
        */
        
        let mut fragments = Vec::new();

        for i in 0..number_of_fragments {
            let start = i * fragment_payload_size;
            let end = (start + fragment_payload_size).min(self.buf.len());

            let fragment_buf = self.buf[start..end].to_vec().clone();

            let fragment =
                MessageFragment::new(id, i as u16, number_of_fragments as u16, fragment_buf);

            fragments.push(fragment);
        }

        #[cfg(debug_assertions)]
        for fragment in &fragments {
            
            if fragment.get_whole_buf().len() > self.max_datagram_size {
                panic!(
                    "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                    fragment.get_whole_buf().len()
                );
            }
        }

        return Ok(fragments);
    }
}

#[cfg(test)]
mod tests {

    use commons::logging::setup_test_logging;

    use crate::messaging::{FragmentAssembler, Fragmenter, messagefragment::FRAGMENT_HEADER_SIZE};

    const MAX_MESSAGES: usize = 2;
    const DATAGRAM_SIZE: usize = 20;
    const FRAGMENT_PAYLOAD_SIZE: usize = fragment_payload_size(DATAGRAM_SIZE);
    const TEST_MESSAGE_VEC_LENGTH: usize = FRAGMENT_PAYLOAD_SIZE * 2 - 3;

    const fn fragment_payload_size(max_datagram_size: usize) -> usize {
        max_datagram_size - FRAGMENT_HEADER_SIZE
    }

    fn make_message(length: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(length);
        for i in 0..length {
            buf.insert(i, i.try_into().unwrap());
        }
        buf
    }

    fn make_and_assemble_fragments(fragmenter: &mut Fragmenter, fragment_assembler: &mut FragmentAssembler) {
        let message = make_message(TEST_MESSAGE_VEC_LENGTH);
        let mut fragments = fragmenter.make_fragments(&message).unwrap();

        assert_eq!(2, fragments.len());

        let first_fragment = fragments.remove(0);
        let second_fragment = fragments.remove(0);

        // Receive fragments out-of-order
        let result = fragment_assembler.add_fragment(second_fragment);
        assert!(matches!(result, None));

        let received_message = fragment_assembler.add_fragment(first_fragment).unwrap();

        assert_eq!(message, received_message);
    }

    #[test]
    fn assemble_fragments() {
        setup_test_logging();
        
        let mut fragmenter = Fragmenter::new(DATAGRAM_SIZE);
        let mut fragment_assembler = FragmentAssembler::new(MAX_MESSAGES);

        make_and_assemble_fragments(&mut fragmenter, &mut fragment_assembler);
        make_and_assemble_fragments(&mut fragmenter, &mut fragment_assembler);
    }

    #[test]
    fn drop_oldest_fragment() {
        setup_test_logging();
        
        let mut fragmenter = Fragmenter::new(DATAGRAM_SIZE);
        let mut fragment_assembler = FragmentAssembler::new(MAX_MESSAGES);

        let message = make_message(TEST_MESSAGE_VEC_LENGTH);
        let mut fragments = fragmenter.make_fragments(&message).unwrap();

        assert_eq!(2, fragments.len());
        let first_fragment = fragments.remove(0);
        let second_fragment = fragments.remove(0);

        let result = fragment_assembler.add_fragment(second_fragment);
        assert!(matches!(result, None));

        // Fragment enough messages so that the first one is dropped.
        for _ in 0..MAX_MESSAGES {
            let message = make_message(TEST_MESSAGE_VEC_LENGTH);
            let mut fragments = fragmenter.make_fragments(&message).unwrap();

            // Only add one fragment from the message to fill up the assembler
            // and cause it to drop the first one
            let result = fragment_assembler.add_fragment(fragments.pop().unwrap());
            assert!(matches!(result, None));
        }

        //Original fragment should be dropped, so this shouldn't return a message
        let result = fragment_assembler.add_fragment(first_fragment);
        assert!(matches!(result, None));
    }
}