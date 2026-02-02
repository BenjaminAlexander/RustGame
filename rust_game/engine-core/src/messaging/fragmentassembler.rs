use {
    crate::messaging::{MessageFragment, messagefragment::{FragmentHeader, IdType, IndexType}}, log::warn, serde::de::DeserializeOwned, std::collections::HashMap
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

//TODO: rename to MessageAssembler
pub struct FragmentAssembler {
    fragment_payload_size: usize,
    max_messages: usize,
    current_id_range: IdRange,
    available_messages: Vec<PartiallyAssembledMessage>,
    partial_messages: HashMap<IdType, PartiallyAssembledMessage>,
    newest_and_oldest_id: Option<(IdType,IdType)>
}

//TODO: review buffer allocation in the fragment assembler
impl FragmentAssembler {
    pub fn new(max_datagram_size: usize, max_messages: usize) -> Self {
        return Self {
            current_id_range: IdRange::First,
            fragment_payload_size: MessageFragment::max_fragment_payload_size(max_datagram_size),
            max_messages,
            available_messages: Vec::new(),
            partial_messages: HashMap::new(),
            newest_and_oldest_id: None
        };
    }

    /// Removes a partially assembled message from the map and maintains the link-list of partial messages
    fn removed_partially_assembled_message(&mut self, id: &IdType) {
        let removed_message = self.partial_messages.remove(id).unwrap();
        let removed_id = removed_message.id;
        let removed_next_id = removed_message.next_id;
        let removed_previous_id = removed_message.previous_id;
        self.available_messages.push(removed_message);

        if let Some(next_id) = removed_next_id {
            let next_partial_message = self.partial_messages.get_mut(&next_id).unwrap();
            next_partial_message.previous_id = removed_previous_id;
        }

        if let Some(previous_id) = removed_previous_id {
            let previous_partial_message = self.partial_messages.get_mut(&previous_id).unwrap();
            previous_partial_message.next_id = removed_next_id;
        }

        let (newest_id, oldest_id) = self.newest_and_oldest_id.unwrap();

        if newest_id == removed_id {
            if let Some(next_id) = removed_next_id {
                self.newest_and_oldest_id = Some((next_id, oldest_id));
            } else {
                self.newest_and_oldest_id = None;
            }
        } else if oldest_id == removed_id {
            if let Some(previous_id) = removed_previous_id {
                self.newest_and_oldest_id = Some((newest_id, previous_id));
            } else {
                self.newest_and_oldest_id = None;
            }
        }
    }

    /// If this is a new range of IDs, clear out any residual fragments from
    /// the last time we saw this range of IDs.  This is prevents a message that
    /// was never assembled, from causing a subsequent message with the same ID
    /// (this happens when the ID rolls over back to zero) from also failing to
    /// assemble.
    fn drop_partial_messages_in_new_id_range(&mut self, next_id_range: IdRange) {
        if self.current_id_range == next_id_range {
            return;
        }

        self.current_id_range = next_id_range;

        let (newest_id, _) = match self.newest_and_oldest_id {
            Some((newest_id, previous_id)) => (newest_id, previous_id),
            None => return,
        };

        let mut current_id = newest_id;

        loop {
            let next_id = self.partial_messages.get_mut(&current_id).unwrap().next_id;
            let range_of_current_id = IdRange::from(current_id);

            if self.current_id_range == range_of_current_id {
                self.removed_partially_assembled_message(&current_id);
            }

            match next_id {
                Some(next_id) => current_id = next_id,
                None => break,
            }
        }

    }

    //TODO: return errors
    pub fn add_fragment<T: DeserializeOwned>(&mut self, buf: &[u8]) -> Option<T> {

        let fragment = MessageFragment::from_slice(buf);
        let header = fragment.parse_header();

        let next_id_range = IdRange::from(header.get_id());
        self.drop_partial_messages_in_new_id_range(next_id_range);

        let fragment_count = header.get_count(self.fragment_payload_size);

        if fragment_count == 1 {
            //TODO: don't unwrap
            //TODO: mabye move serde outside fragmenter/assembler layer
            let message =  rmp_serde::from_slice(fragment.get_payload()).unwrap();
            return Some(message);
        }

        let id = header.get_id();

        match self.partial_messages.get_mut(&id) {
            None => {

                if self.partial_messages.len() < self.max_messages && self.available_messages.len() == 0 {
                    self.available_messages.push(PartiallyAssembledMessage::new(self.fragment_payload_size));
                }

                while self.partial_messages.len() >= self.max_messages {
                    let (_, oldest_id) = self.newest_and_oldest_id.unwrap();
                    self.removed_partially_assembled_message(&oldest_id);
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
                partial_message.set_first_fragment(self.fragment_payload_size, fragment_count, next_id, &header, &fragment);

                self.partial_messages.insert(id, partial_message);

                // Don't need to check if this message is assembled here, since this
                // is the first fragment from this messages.  Messages with only
                // one fragment are handled as a special case above.
                None
            }
            Some(partial_message) => {
                match partial_message.add_fragment(self.fragment_payload_size, &header, &fragment) {
                    Ok(()) => {
                        if partial_message.has_all_fragments() {
                            let buf = partial_message.get_full_message();
                            let message =  rmp_serde::from_slice(buf).unwrap();

                            self.removed_partially_assembled_message(&id);
                            Some(message)
                        } else {
                            None
                        }
                    },
                    Err(()) => {
                        self.removed_partially_assembled_message(&id);
                        None
                    },
                }
            }
        }
    }
}

struct PartiallyAssembledMessage {
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

impl PartiallyAssembledMessage {

    fn new(fragment_payload_size: usize) -> Self {
        Self {
            id: 0,
            total_fragment_count: 0,
            outstanding_fragment_count: 0,
            has_fragment: Vec::with_capacity(2),
            next_id: None,
            previous_id: None,
            buf: Vec::with_capacity(fragment_payload_size * 2),
        }
    }

    pub fn set_first_fragment(&mut self, fragment_payload_size: usize, fragment_count: IndexType, next_id: Option<IdType>, header: &FragmentHeader, fragment: &MessageFragment) {
        let total_length = header.get_total_length();
        self.buf.resize(total_length as usize, 0);

        self.has_fragment.clear();
        self.has_fragment.resize(fragment_count as usize, false);

        self.id = header.get_id();
        self.total_fragment_count = fragment_count;
        self.outstanding_fragment_count = fragment_count;
        self.next_id = next_id;
        self.previous_id = None;

        self.add_fragment(fragment_payload_size, header, fragment).unwrap();
    }

    fn add_fragment(&mut self, fragment_payload_size: usize, header: &FragmentHeader, fragment: &MessageFragment) -> Result<(), ()> {

        let index = header.get_index();

        if self.buf.len() != header.get_total_length() as usize {
            warn!("Received a fragment whose message length did not the expected value.");
            return Err(())
        }

        if let Some(has_fragment) = self.has_fragment.get_mut(index as usize) {
            if *has_fragment {
                warn!("Received the same fragment more than once.  Subsequent fragment is being ignored.");
                return Ok(());
            } else {
                self.outstanding_fragment_count -= 1;
                *has_fragment = true;
            }
        } else {
            todo!("stop trying to assemble this fragment if the index is out of bounds")
        }

        let start_index = header.start_index(fragment_payload_size);
        let expected_end_index = if index == self.total_fragment_count - 1 {
            self.buf.len()
        } else {
            start_index + fragment_payload_size 
        };

        let expected_length = expected_end_index - start_index;
        if fragment.get_payload().len() != expected_length {
            warn!("Received a fragment whose expected length did not match its actual length.  Expected: {}, Actual {}", expected_length, fragment.get_payload().len());
            return Err(());
        }

        let dest = match self.buf.get_mut(start_index..expected_end_index) {
            Some(dest) => dest,
            None => {
                warn!("Failed to get the destination slice for a fragment.");
                return Err(());
            },
        };

        dest.copy_from_slice(&fragment.get_payload());
        return Ok(());
    }

    fn has_all_fragments(&self) -> bool {
        return self.outstanding_fragment_count == 0;
    }

    fn get_full_message(&self) -> &[u8] {
        return self.buf.as_slice();
    }
}
