use crate::messaging::fragmenter::IS_FRAGMENT;

pub type IdType = u32;
pub type IndexType = u16;
pub type TotalLengthType = u64;

const IS_FRAGMENT_INDEX: usize = 0;
const IS_FRAGMENT_SIZE: usize = size_of::<u8>();

//TODO: rename from length to size
const ID_INDEX: usize = IS_FRAGMENT_INDEX + IS_FRAGMENT_SIZE; 
const ID_LENGTH: usize = size_of::<IdType>();

const INDEX_INDEX: usize = ID_INDEX + ID_LENGTH; 
const INDEX_LENGTH: usize = size_of::<IndexType>();

const TOTAL_LENGTH_INDEX: usize = INDEX_INDEX + INDEX_LENGTH; 
const TOTAL_LENGTH_LENGTH: usize = size_of::<TotalLengthType>(); 

pub const FRAGMENT_HEADER_SIZE: usize = TOTAL_LENGTH_INDEX + TOTAL_LENGTH_LENGTH; 

pub struct FragmentHeader {
    id: IdType, 
    index: IndexType, 
    total_length: TotalLengthType
}

impl FragmentHeader {

    pub fn new(id: IdType, index: IndexType, total_length: TotalLengthType) -> Self {
        Self{
            id,
            index,
            total_length
        }
    }

    fn extend_buffer_with_header(&self, buf: &mut Vec<u8>) {
        buf.push(IS_FRAGMENT);
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&self.index.to_be_bytes());
        buf.extend_from_slice(&self.total_length.to_be_bytes());
    }

    fn from_slice(buf: &[u8]) -> Self {

        #[cfg(debug_assertions)]
        {
            let mut array =  [0; IS_FRAGMENT_SIZE];
            array.copy_from_slice(&buf[IS_FRAGMENT_INDEX..(IS_FRAGMENT_INDEX + IS_FRAGMENT_SIZE)]);
            let is_fragment = u8::from_be_bytes(array);
            if is_fragment != IS_FRAGMENT {
                panic!("Illegal value in header")
            }
        }

        let mut array =  [0; ID_LENGTH];
        array.copy_from_slice(&buf[ID_INDEX..(ID_INDEX + ID_LENGTH)]);
        let id = IdType::from_be_bytes(array);

        let mut array =  [0; INDEX_LENGTH];
        array.copy_from_slice(&buf[INDEX_INDEX..(INDEX_INDEX + INDEX_LENGTH)]);
        let index = IndexType::from_be_bytes(array);

        let mut array =  [0; TOTAL_LENGTH_LENGTH];
        array.copy_from_slice(&buf[TOTAL_LENGTH_INDEX..(TOTAL_LENGTH_INDEX + TOTAL_LENGTH_LENGTH)]);
        let total_length = TotalLengthType::from_be_bytes(array);

        Self{
            id,
            index,
            total_length
        }
    }

    pub fn get_id(&self) -> IdType {
        self.id
    }

    pub fn get_index(&self) -> IndexType {
        self.index
    }

    pub fn get_total_length(&self) -> TotalLengthType {
        self.total_length
    }

    pub fn get_count(&self, fragment_payload_size: usize) -> IndexType {
        Self::number_of_fragments(fragment_payload_size, self.total_length as usize)
    }


    pub const fn number_of_fragments(fragment_payload_size: usize, buffer_length: usize) -> IndexType {
        let mut number_of_fragments = buffer_length / fragment_payload_size;

        if buffer_length % fragment_payload_size != 0 {
            number_of_fragments = number_of_fragments + 1;
        }

        number_of_fragments as IndexType
    }

    /// Returns the index in the full message of where this fragment begins
    pub fn start_index(&self, fragment_payload_size: usize) -> usize {
        self.index as usize * fragment_payload_size
    }

}

pub struct MessageFragment<'a> {
    buf: &'a [u8],
}

impl<'a> MessageFragment<'a> {

    pub fn new(buf: &'a mut Vec<u8>, header: &FragmentHeader, fragment_buf: &[u8]) -> Self {
        buf.clear();
        header.extend_buffer_with_header(buf);
        buf.extend_from_slice(fragment_buf);

        Self {
            buf: buf.as_slice()
        }
    }

    pub fn from_slice(buf: &'a [u8]) -> Self {
        //TODO: check if this buf is valid, i.e. contains at least the header plus 1 byte, maybe this should be on the method to extract the header struct
        return Self { buf };
    }

    pub fn parse_header(&self) -> FragmentHeader {
        FragmentHeader::from_slice(self.buf)
    }

    pub fn get_payload(&self) -> &'a [u8] {
        &self.buf[FRAGMENT_HEADER_SIZE..self.buf.len()]
    }

    pub fn get_buffer(&self) -> &'a [u8] {
        &self.buf
    }

    pub const fn max_fragment_payload_size(max_datagram_size: usize) -> usize {
        max_datagram_size - FRAGMENT_HEADER_SIZE
    }
}
