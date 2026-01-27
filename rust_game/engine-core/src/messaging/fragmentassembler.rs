use {
    crate::messaging::{MessageFragment, messagefragment::{IdType, IndexType}}, log::warn, serde::de::DeserializeOwned, std::{cell::Cell, collections::{HashMap, VecDeque}}
};

#[derive(PartialEq)]
enum IdRange {
    First,
    Second,
    Third,
    Fourth
}

impl From<IdType> for IdRange {
    fn from(value: IdType) -> Self {
        const SHIFT: usize = size_of::<IdType>() - 2;
        match value >> SHIFT {
            0 => IdRange::First,
            1 => IdRange::Second,
            2 => IdRange::Third,
            _ => IdRange::Fourth
        }
    }
}

pub struct FragmentAssembler {
    max_datagram_size: usize,
    fragment_payload_size: usize,
    max_messages: usize,
    current_id_range: IdRange,
    available_messages: Vec<PartiallyAssembledFragment>,
    partial_messages: HashMap<IdType, PartiallyAssembledFragment>,
    partial_message_queue: VecDeque<IdType>,
    newest_and_oldest_id: Option<(IdType,IdType)>
}

//TODO: review buffer allocation in the fragment assembler
impl FragmentAssembler {
    pub fn new(max_datagram_size: usize, max_messages: usize) -> Self {

        let fragment_payload_size = MessageFragment::fragment_payload_size(max_datagram_size);

        let mut available_messages = Vec::with_capacity(max_messages);
        available_messages.resize_with(max_messages, ||{
            PartiallyAssembledFragment::new(fragment_payload_size)
        });

        return Self {
            current_id_range: IdRange::First,
            max_datagram_size,
            fragment_payload_size: MessageFragment::fragment_payload_size(max_datagram_size),
            max_messages,
            available_messages,
            partial_messages: HashMap::new(),
            newest_and_oldest_id: None
        };
    }

    fn removed_partially_assembled_message(
        newest_and_oldest_id: &mut Option<(IdType,IdType)>,
        partial_messages: &mut HashMap<IdType, PartiallyAssembledFragment>, 
        removed_message: &PartiallyAssembledFragment
    ) {
        let removed_id = removed_message.id;
        let removed_next_id = removed_message.next_id;
        let removed_previous_id = removed_message.previous_id;

        if let Some(next_id) = removed_next_id {
            let next_partial_message = partial_messages.get_mut(&next_id).unwrap();
            next_partial_message.previous_id = removed_previous_id;
        }

        if let Some(previous_id) = removed_previous_id {
            let previous_partial_message = partial_messages.get_mut(&previous_id).unwrap();
            previous_partial_message.next_id = removed_next_id;
        }

        let (newest_id, oldest_id) = newest_and_oldest_id.unwrap();

        if newest_id == removed_id {
            if let Some(next_id) = removed_next_id {
                *newest_and_oldest_id = Some((next_id, oldest_id));
            } else {
                *newest_and_oldest_id = None;
            }
        } else if oldest_id == removed_id {
            if let Some(previous_id) = removed_previous_id {
                *newest_and_oldest_id = Some((newest_id, previous_id));
            } else {
                *newest_and_oldest_id = None;
            }
        }
    }

    pub fn add_fragment<T: DeserializeOwned>(&mut self, buf: &[u8]) -> Option<T> {

        let fragment = MessageFragment::from_vec(buf.to_vec());

        // If this is a new range of IDs, clear out any residual fragments from 
        // the last time we saw this range of IDs
        let next_id_range = IdRange::from(fragment.get_id());
        if self.current_id_range != next_id_range {
            self.current_id_range = next_id_range;

            todo!();

            let x = self.partial_messages.extract_if(|_, partial_message|{
                let message_id_range = IdRange::from(partial_message.id);
                if message_id_range == self.current_id_range {
                    Self::removed_partially_assembled_message(&mut self.newest_and_oldest_id, &mut self.partial_messages, &partial_message);
                    true
                } else {
                    false
                }
            }).for_each(|(_, removed_message)|{
                self.available_messages.push(removed_message);partial_message
            });
        }

        let fragment_count = fragment.get_count(self.fragment_payload_size);

        if fragment_count == 1 {
            //TODO: don't unwrap
            //TODO: don't move the buffer here
            let message =  rmp_serde::from_slice(fragment.move_buf().as_slice()).unwrap();
            return Some(message);
        }

        let id = fragment.get_id();

        let partial_message = match self.partial_messages.get_mut(&id) {
            None => {
                while self.partial_messages.len() >= self.max_messages {
                    let (_, oldest_id) = self.newest_and_oldest_id.unwrap();
                    let removed_message = self.partial_messages.remove(&oldest_id).unwrap();
                    Self::removed_partially_assembled_message(&mut self.newest_and_oldest_id, &mut self.partial_messages, &removed_message);
                    self.available_messages.push(removed_message);
                }

                let next_id = match self.newest_and_oldest_id {
                    Some((newest_id, oldest_id)) => {
                        let newest_partial_message = self.partial_messages.get_mut(&newest_id).unwrap();
                        newest_partial_message.previous_id = Some(id);
                        self.newest_and_oldest_id = Some((id, oldest_id));
                        Some(newest_id)
                    },
                    None => {
                        self.newest_and_oldest_id = Some((id, id));
                        None
                    },
                };

                let mut partial_message = self.available_messages.pop().unwrap();
                partial_message.set_first_fragment(self.fragment_payload_size, fragment_count, next_id, &fragment);

                self.partial_messages.insert(id, partial_message);

                //TODO: just use the index here
                self.partial_messages.get(&id).unwrap()
            }
            Some(partial_message) => {
                partial_message.add_fragment(self.fragment_payload_size, &fragment);
                partial_message
            }
        };

        if partial_message.has_all_fragments() {
            let buf = partial_message.get_full_message();
            let message =  rmp_serde::from_slice(buf).unwrap();

            let removed_message = self.partial_messages.remove(&id).unwrap();
            Self::removed_partially_assembled_message(&mut self.newest_and_oldest_id, &mut self.partial_messages, &removed_message);
            self.available_messages.push(removed_message);
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

    fn get_full_message(&self) -> &[u8] {
        return self.buf.as_slice();
    }
}
