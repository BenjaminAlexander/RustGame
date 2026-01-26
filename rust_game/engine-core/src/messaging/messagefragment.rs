pub type IdType = u32;
pub type IndexType = u16;
pub type TotalLengthType = u64;

const ID_INDEX: usize = 0; 
const ID_LENGTH: usize = size_of::<IdType>();

const INDEX_INDEX: usize = ID_INDEX + ID_LENGTH; 
const INDEX_LENGTH: usize = size_of::<IndexType>();

const TOTAL_LENGTH_INDEX: usize = INDEX_INDEX + INDEX_LENGTH; 
const TOTAL_LENGTH_LENGTH: usize = size_of::<TotalLengthType>(); 

pub const FRAGMENT_HEADER_SIZE: usize = TOTAL_LENGTH_INDEX + TOTAL_LENGTH_LENGTH; 

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

    pub fn write(&mut self, id: IdType, index: IndexType, total_length: TotalLengthType, buf: &[u8]) {
        self.buf.clear();
        self.buf.extend_from_slice(&id.to_be_bytes());
        self.buf.extend_from_slice(&index.to_be_bytes());
        self.buf.extend_from_slice(&total_length.to_be_bytes());
        self.buf.extend_from_slice(buf);
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        //TODO: check if this buf is valid
        return Self { buf };
    }

    pub fn get_id(&self) -> IdType {
        let mut array =  [0; ID_LENGTH];
        array.copy_from_slice(&self.buf[ID_INDEX..(ID_INDEX + ID_LENGTH)]);
        return IdType::from_be_bytes(array);
    }

    pub fn get_index(&self) -> IndexType {
        let mut array =  [0; INDEX_LENGTH];
        array.copy_from_slice(&self.buf[INDEX_INDEX..(INDEX_INDEX + INDEX_LENGTH)]);
        return IndexType::from_be_bytes(array);
    }

    pub fn get_total_length(&self) -> TotalLengthType {
        let mut array =  [0; TOTAL_LENGTH_LENGTH];
        array.copy_from_slice(&self.buf[TOTAL_LENGTH_INDEX..(TOTAL_LENGTH_INDEX + TOTAL_LENGTH_LENGTH)]);
        return TotalLengthType::from_be_bytes(array);
    }

    pub fn get_count(&self, fragment_payload_size: usize) -> IndexType {
        Self::number_of_fragments(fragment_payload_size, self.get_total_length() as usize)
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

    pub fn payload(&self) -> &[u8] {
        &self.buf[FRAGMENT_HEADER_SIZE..self.buf.len()]
    }

    pub const fn fragment_payload_size(max_datagram_size: usize) -> usize {
        max_datagram_size - FRAGMENT_HEADER_SIZE
    }

    pub const fn number_of_fragments(fragment_payload_size: usize, buffer_length: usize) -> IndexType {
        let mut number_of_fragments = buffer_length / fragment_payload_size;

        if buffer_length % fragment_payload_size != 0 {
            number_of_fragments = number_of_fragments + 1;
        }

        number_of_fragments as IndexType
    }

    pub const fn start_index(fragment_payload_size: usize, index: IndexType) -> usize {
        index as usize * fragment_payload_size
    }
}
