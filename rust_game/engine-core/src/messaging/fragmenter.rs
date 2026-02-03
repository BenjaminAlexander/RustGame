use rmp_serde::encode::Error;
use serde::Serialize;

use crate::messaging::{MessageFragment, messagefragment::{FragmentHeader, IdType, IndexType, TotalLengthType}};

const IS_PACK: u8 = 0;
const IS_FRAGMENT: u8 = IS_PACK + 1;
const PACK_OR_FRAGMENT_HEADER_SIZE: usize = size_of::<u8>();

type PackSegmentLengthType = u16;

const LENGTH_OF_PACK_SEGMENT_SIZE: usize = size_of::<PackSegmentLengthType>();

const PACK_HEADER_SIZE: usize = PACK_OR_FRAGMENT_HEADER_SIZE + LENGTH_OF_PACK_SEGMENT_SIZE;
const MIN_SPACE_REQUIRED_TO_PACK: usize = LENGTH_OF_PACK_SEGMENT_SIZE + 1;

pub struct Fragmenter {
    next_fragment_id: IdType,
    max_datagram_size: usize,
    fragment_payload_size: usize,
    buf: Vec<u8>,
    index_of_last_segment: usize,
    fragment_buf: Vec<u8>
}

impl Fragmenter {
    pub fn new(max_datagram_size: usize) -> Self {
        let fragment_payload_size = MessageFragment::max_fragment_payload_size(max_datagram_size - PACK_OR_FRAGMENT_HEADER_SIZE);

        let mut rtn = Self {
            next_fragment_id: 0,
            max_datagram_size,
            fragment_payload_size,
            buf: Vec::with_capacity(max_datagram_size),
            index_of_last_segment: 1,
            fragment_buf: Vec::with_capacity(max_datagram_size)
        };

        rtn.clear();

        rtn
    }

    fn take_next_fragment_id(&mut self) -> IdType {
        let id = self.next_fragment_id;

        if self.next_fragment_id == IdType::max_value() {
            self.next_fragment_id = 0;
        } else {
            self.next_fragment_id = self.next_fragment_id + 1;
        }

        id
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.buf.push(IS_PACK);
        self.index_of_last_segment = 1;
    }

    pub fn make_fragments<T: Serialize>(&mut self, message: &T) -> Result<FragmentIterator<'_>, Error> {

        self.index_of_last_segment = self.buf.len();

        for _ in 0..LENGTH_OF_PACK_SEGMENT_SIZE {
            self.buf.push(0);
        }

        rmp_serde::encode::write(&mut self.buf, message)?;

        let buf_len = self.buf.len();
        let pack_segment_len = (buf_len - self.index_of_last_segment - LENGTH_OF_PACK_SEGMENT_SIZE) as u16;

        //TODO: don't unwrap here
        let pack_segment_len_dest = self.buf.get_mut(buf_len..(self.index_of_last_segment + LENGTH_OF_PACK_SEGMENT_SIZE)).unwrap();
        pack_segment_len_dest.copy_from_slice(&pack_segment_len.to_be_bytes());


        //Just added message to the pack
        //TODO: check if the pack is too big and if its time to flush

        let id = self.take_next_fragment_id();

        let number_of_fragments = FragmentHeader::number_of_fragments(self.fragment_payload_size, self.buf.len());

        let iterator = FragmentIterator {
            fragmenter: self,
            id,
            number_of_fragments,
            next_index: 0
        };
        
        return Ok(iterator);
    }

    pub fn flush(&mut self) -> 
}

pub struct FragmentIterator<'a> {
    fragmenter: &'a mut Fragmenter,
    id: u32,
    number_of_fragments: IndexType,
    next_index: IndexType
}

impl FragmentIterator<'_> {

    pub fn next(&mut self) -> Option<&[u8]> {
        if self.fragmenter.index_of_last_segment > self.fragmenter.max_datagram_size - MIN_SPACE_REQUIRED_TO_PACK {

        }



        if self.next_index >= self.number_of_fragments {
            return None;
        }

        let header = FragmentHeader::new(
            self.id, 
            self.next_index as IndexType, 
            self.fragmenter.buf.len() as TotalLengthType
        );

        let start = header.start_index(self.fragmenter.fragment_payload_size);
        let end = (start + self.fragmenter.fragment_payload_size).min(self.fragmenter.buf.len());

        let fragment_buf = &self.fragmenter.buf[start..end];

        let fragment = MessageFragment::new(
            &mut self.fragmenter.fragment_buf, 
            &header, 
            fragment_buf
        );

        #[cfg(debug_assertions)]
        if fragment.get_buffer().len() > self.fragmenter.max_datagram_size {
            panic!(
                "Datagram is larger than MAX_UDP_DATAGRAM_SIZE: {:?}",
                fragment.get_buffer().len()
            );
        }

        self.next_index += 1;

        Some(fragment.get_buffer())
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

        let first_fragment = fragments.next().unwrap().to_vec();
        let second_fragment = fragments.next().unwrap().to_vec();
        assert!(matches!(fragments.next(), None));

        // Receive fragments out-of-order
        let result: Option<Vec<u8>> = fragment_assembler.add_fragment(second_fragment.as_slice());
        assert!(matches!(result, None));

        let received_message: Vec<u8> = fragment_assembler.add_fragment(first_fragment.as_slice()).unwrap();

        assert_eq!(message, received_message);
    }

    #[test]
    fn assemble_fragments() {
        setup_test_logging();
        
        let mut fragmenter = Fragmenter::new(DATAGRAM_SIZE);
        let mut fragment_assembler = FragmentAssembler::new(DATAGRAM_SIZE, MAX_MESSAGES);

        make_and_assemble_fragments(&mut fragmenter, &mut fragment_assembler);
        make_and_assemble_fragments(&mut fragmenter, &mut fragment_assembler);
    }

    #[test]
    fn drop_oldest_fragment() {
        setup_test_logging();
        
        let mut fragmenter = Fragmenter::new(DATAGRAM_SIZE);
        let mut fragment_assembler = FragmentAssembler::new(DATAGRAM_SIZE, MAX_MESSAGES);

        let message = make_message(TEST_MESSAGE_VEC_LENGTH);
        let mut fragments = fragmenter.make_fragments(&message).unwrap();

        let first_fragment = fragments.next().unwrap().to_vec();
        let second_fragment = fragments.next().unwrap().to_vec();
        assert!(matches!(fragments.next(), None));

        let result: Option<Vec<u8>> = fragment_assembler.add_fragment(second_fragment.as_slice());
        assert!(matches!(result, None));

        // Fragment enough messages so that the first one is dropped.
        for _ in 0..MAX_MESSAGES {
            let message = make_message(TEST_MESSAGE_VEC_LENGTH);
            let mut fragments = fragmenter.make_fragments(&message).unwrap();

            // Only add one fragment from the message to fill up the assembler
            // and cause it to drop the first one
            let result: Option<Vec<u8>> = fragment_assembler.add_fragment(fragments.next().unwrap());
            assert!(matches!(result, None));
        }

        //Original fragment should be dropped, so this shouldn't return a message
        let result: Option<Vec<u8>> = fragment_assembler.add_fragment(first_fragment.as_slice());
        assert!(matches!(result, None));
    }
}