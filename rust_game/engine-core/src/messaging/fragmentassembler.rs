use {
    crate::messaging::{MessageFragment, messagefragment::{IdType, IndexType, TotalLengthType}}, log::warn, serde::de::DeserializeOwned, std::collections::HashMap
};

pub struct FragmentAssembler {
    max_datagram_size: usize,
    fragment_payload_size: usize,
    max_messages: usize,
    partial_messages: Vec<PartiallyAssembledFragment>,
    id_to_message_array_index: HashMap<IdType, usize>,
    newest_and_oldest_id: Option<(IdType,IdType)>
}

//TODO: review buffer allocation in the fragment assembler
impl FragmentAssembler {
    pub fn new(max_datagram_size: usize, max_messages: usize) -> Self {

        let fragment_payload_size = MessageFragment::fragment_payload_size(max_datagram_size);

        let mut partial_messages = Vec::new();
        partial_messages.resize_with(max_messages, ||{
            PartiallyAssembledFragment::new(fragment_payload_size)
        });

        return Self {
            max_datagram_size,
            fragment_payload_size: MessageFragment::fragment_payload_size(max_datagram_size),
            max_messages,
            partial_messages,
            id_to_message_array_index: HashMap::new(),
            newest_and_oldest_id: None
        };
    }

    fn removed_partially_assembled_message(&mut self, id: &u32) -> PartiallyAssembledFragment {
        let partially_assembled_message = self.id_to_message_array_index.remove(id).unwrap();

        if let Some(next_id) = partially_assembled_message.next_id {
            self.id_to_message_array_index.get_mut(&next_id).unwrap().previous_id = partially_assembled_message.previous_id;
        }

        if let Some(previous_id) = partially_assembled_message.previous_id {
            self.id_to_message_array_index.get_mut(&previous_id).unwrap().next_id = partially_assembled_message.next_id;
        }

        let (newest_id, oldest_id) = self.newest_and_oldest_id.unwrap();

        if newest_id == *id {
            if let Some(next_id) = partially_assembled_message.next_id {
                self.newest_and_oldest_id = Some((next_id, oldest_id));
            } else {
                self.newest_and_oldest_id = None;
            }
        } else if oldest_id == *id {
            if let Some(previous_id) = partially_assembled_message.previous_id {
                self.newest_and_oldest_id = Some((newest_id, previous_id));
            } else {
                self.newest_and_oldest_id = None;
            }
        }

        partially_assembled_message
    }

    pub fn add_fragment<T: DeserializeOwned>(&mut self, buf: &[u8]) -> Option<T> {

        let fragment = MessageFragment::from_vec(buf.to_vec());
        let fragment_count = fragment.get_count(self.fragment_payload_size);

        if fragment_count == 1 {
            //TODO: don't unwrap
            let message =  rmp_serde::from_slice(fragment.move_buf().as_slice()).unwrap();
            return Some(message);
        }

        let id = fragment.get_id();

        let partial = match self.id_to_message_array_index.get_mut(&id) {
            None => {
                while self.id_to_message_array_index.len() >= self.max_messages {
                    let (_, oldest_id) = self.newest_and_oldest_id.unwrap();
                    self.removed_partially_assembled_message(&oldest_id);
                }

                let next_id = match self.newest_and_oldest_id {
                    Some((newest_id, oldest_id)) => {
                        self.id_to_message_array_index.get_mut(&newest_id).unwrap().previous_id = Some(id);
                        self.newest_and_oldest_id = Some((id, oldest_id));
                        Some(newest_id)
                    },
                    None => {
                        self.newest_and_oldest_id = Some((id, id));
                        None
                    },
                };

                let partial_message = PartiallyAssembledFragment::new(self.fragment_payload_size, fragment_count, next_id, &fragment);

                self.id_to_message_array_index.insert(id, partial_message);
                self.id_to_message_array_index.get_mut(&id).unwrap()
            }
            Some(partial) => {
                partial.add_fragment(self.fragment_payload_size, &fragment);
                partial
            }
        };

        if partial.has_all_fragments() {
            let mut completed_message = self.removed_partially_assembled_message(&id);
            let buf = completed_message.get_full_message();
            let message =  rmp_serde::from_slice(buf).unwrap();
            return Some(message);
        } else {
            return None;
        }
    }
}

//TODO: rename to message
struct PartiallyAssembledFragment {
    id: IdType,

    total_fragment_count: IndexType,
    outstanding_fragment_count: IndexType,
    has_fragment: Vec<bool>,
    
    /// Points to an older partially assembled message
    next_id: Option<IdType>,

    /// Points to a newer partially assembled message
    previous_id: Option<IdType>,

    //TODO: allocate this to the right size at the begining
    buf: Vec<u8>
}

impl PartiallyAssembledFragment {

    fn new(fragment_payload_size: usize) -> Self {
        Self {
            id: 0,
            total_fragment_count: 0,
            outstanding_fragment_count: 0,
            has_fragment: Vec::with_capacity(fragment_payload_size * 2),
            next_id: None,
            previous_id: None,
            buf: Vec::with_capacity(fragment_payload_size * 2),
        }
    }

    pub fn set_first_fragment(&mut self, fragment_payload_size: usize, fragment_count: IndexType, next_id: Option<IdType>, fragment: &MessageFragment) {
        let total_length = fragment.get_total_length();
        self.buf.resize(total_length as usize, 0);


        self.has_fragment.clear();
        self.has_fragment.resize(fragment_count as usize, false);

        self.id = fragment.get_id();
        self.total_fragment_count = fragment_count;
        self.outstanding_fragment_count = fragment_count;
        self.next_id = next_id;
        self.previous_id = None;

        self.add_fragment(fragment_payload_size, fragment);
    }

    fn add_fragment(&mut self, fragment_payload_size: usize, fragment: &MessageFragment) {

        let index = fragment.get_index();

        //TODO: assert total length, if they don't match, stop trying to assemble this fragment

        if let Some(has_fragment) = self.has_fragment.get_mut(index as usize) {
            if *has_fragment {
                warn!("Received the same fragment more than once.  Subsequent fragment is being ignored.");
                return;
            } else {
                self.outstanding_fragment_count -= 1;
                *has_fragment = true;
            }
        } else {
            todo!("stop trying to assemble this fragment if the index is out of bounds")
        }

        let start_index = MessageFragment::start_index(fragment_payload_size, index);
        let expected_end_index = if index == self.total_fragment_count - 1 {
            self.buf.len()
        } else {
            start_index + fragment_payload_size 
        };

        //TODO: assert actual end index based of fragment buffer length, stop trying to assemble this fragment

        //TODO: don't unwrap, stop trying to assemble this fragment
        let dest = self.buf.get_mut(start_index..expected_end_index).unwrap();
        dest.copy_from_slice(&fragment.payload());
    }

    fn has_all_fragments(&self) -> bool {
        return self.outstanding_fragment_count == 0;
    }

    fn get_full_message(&mut self) -> &[u8] {
        return self.buf.as_slice();
    }
}
