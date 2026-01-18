use rmp_serde::encode::Error;
use serde::Serialize;

use crate::messaging::{
    MessageFragment,
    messagefragment::FRAGMENT_HEADER_SIZE,
};

pub struct Fragmenter {
    next_id: u32,
    max_datagram_size: usize,
    fragment_payload_size: usize,
    buf: Vec<u8>,
    fragment: MessageFragment
}

impl Fragmenter {
    pub fn new(max_datagram_size: usize) -> Self {
        return Self {
            next_id: 0,
            max_datagram_size,
            fragment_payload_size: max_datagram_size - FRAGMENT_HEADER_SIZE,
            buf: Vec::with_capacity(max_datagram_size),
            fragment: MessageFragment::new(max_datagram_size)
        };
    }

    pub fn make_fragments<T: Serialize>(&mut self, message: &T) -> Result<FragmentIterator<'_>, Error> {
        self.buf.clear();
        rmp_serde::encode::write(&mut self.buf, message)?;

        let id = self.next_id;

        if self.next_id == u32::max_value() {
            self.next_id = 0;
        } else {
            self.next_id = self.next_id + 1;
        }

        let mut number_of_fragments = self.buf.len() / self.fragment_payload_size;

        if self.buf.len() % self.fragment_payload_size != 0 {
            number_of_fragments = number_of_fragments + 1;
        }

        let iterator = FragmentIterator {
            fragmenter: self,
            id,
            number_of_fragments,
            next_index: 0
        };
        
        return Ok(iterator);
    }
}

pub struct FragmentIterator<'a> {
    fragmenter: &'a mut Fragmenter,
    id: u32,
    number_of_fragments: usize,
    next_index: usize
}

impl FragmentIterator<'_> {

    pub fn next(&mut self) -> Option<&MessageFragment> {
        if self.next_index >= self.number_of_fragments {
            return None;
        }

        let start = self.next_index * self.fragmenter.fragment_payload_size;
        let end = (start + self.fragmenter.fragment_payload_size).min(self.fragmenter.buf.len());

        let fragment_buf = &self.fragmenter.buf[start..end];

        self.fragmenter.fragment.write(self.id, self.next_index as u16, self.number_of_fragments as u16, fragment_buf);


        #[cfg(debug_assertions)]
        if self.fragmenter.fragment.get_whole_buf().len() > self.fragmenter.max_datagram_size {
            panic!(
                "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                self.fragmenter.fragment.get_whole_buf().len()
            );
        }

        self.next_index += 1;

        Some(&self.fragmenter.fragment)
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

        let first_fragment = fragments.next().unwrap().clone();
        let second_fragment = fragments.next().unwrap().clone();
        assert!(matches!(fragments.next(), None));

        // Receive fragments out-of-order
        let result: Option<Vec<u8>> = fragment_assembler.add_fragment(second_fragment.get_whole_buf());
        assert!(matches!(result, None));

        let received_message: Vec<u8> = fragment_assembler.add_fragment(first_fragment.get_whole_buf()).unwrap();

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

        let first_fragment = fragments.next().unwrap().clone();
        let second_fragment = fragments.next().unwrap().clone();
        assert!(matches!(fragments.next(), None));

        let result: Option<Vec<u8>> = fragment_assembler.add_fragment(second_fragment.get_whole_buf());
        assert!(matches!(result, None));

        // Fragment enough messages so that the first one is dropped.
        for _ in 0..MAX_MESSAGES {
            let message = make_message(TEST_MESSAGE_VEC_LENGTH);
            let mut fragments = fragmenter.make_fragments(&message).unwrap();

            // Only add one fragment from the message to fill up the assembler
            // and cause it to drop the first one
            let result: Option<Vec<u8>> = fragment_assembler.add_fragment(fragments.next().unwrap().get_whole_buf());
            assert!(matches!(result, None));
        }

        //Original fragment should be dropped, so this shouldn't return a message
        let result: Option<Vec<u8>> = fragment_assembler.add_fragment(first_fragment.get_whole_buf());
        assert!(matches!(result, None));
    }
}