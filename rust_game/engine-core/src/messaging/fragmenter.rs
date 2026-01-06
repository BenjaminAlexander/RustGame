use crate::messaging::{
    MessageFragment,
    messagefragment::FRAGMENT_HEADER_SIZE,
};

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

    pub fn make_fragments(&mut self, buf: &Vec<u8>) -> Vec<MessageFragment> {
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

        #[cfg(debug_assertions)]
        for fragment in &fragments {
            
            if fragment.get_whole_buf().len() > self.max_datagram_size {
                panic!(
                    "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                    fragment.get_whole_buf().len()
                );
            }
        }

        return fragments;
    }
}

#[cfg(test)]
mod tests {

    use crate::messaging::{FragmentAssembler, Fragmenter, messagefragment::FRAGMENT_HEADER_SIZE};

    const DATAGRAM_SIZE: usize = 20;

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
        let message = make_message(fragment_payload_size(fragmenter.max_datagram_size) * 2);
        let mut fragments = fragmenter.make_fragments(&message);

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
        let mut fragmenter = Fragmenter::new(DATAGRAM_SIZE);
        let mut fragment_assembler = FragmentAssembler::new(2);

        make_and_assemble_fragments(&mut fragmenter, &mut fragment_assembler);
        make_and_assemble_fragments(&mut fragmenter, &mut fragment_assembler);
    }

    #[test]
    fn drop_oldest_fragment() {
        
        const MAX_MESSAGES: usize = 2;

        let mut fragmenter = Fragmenter::new(DATAGRAM_SIZE);
        let mut fragment_assembler = FragmentAssembler::new(MAX_MESSAGES);

        let message = make_message(fragment_payload_size(fragmenter.max_datagram_size) * 2);
        let mut fragments = fragmenter.make_fragments(&message);

        assert_eq!(2, fragments.len());
        let first_fragment = fragments.remove(0);
        let second_fragment = fragments.remove(0);

        let result = fragment_assembler.add_fragment(second_fragment);
        assert!(matches!(result, None));

        // Fragment enough messages so that the first one is dropped.
        for _ in 0..MAX_MESSAGES {
            let message = make_message(fragment_payload_size(fragmenter.max_datagram_size) * 2);
            let mut fragments = fragmenter.make_fragments(&message);

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