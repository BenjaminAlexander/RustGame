use {
    crate::messaging::MessageFragment,
    std::collections::HashMap,
};

pub struct FragmentAssembler {
    max_messages: usize,
    messages: HashMap<u32, PartiallyAssembledFragment>,
}

impl FragmentAssembler {
    pub fn new(max_messages: usize) -> Self {
        return Self {
            max_messages,
            messages: HashMap::new(),
        };
    }

    fn decrement_sequence_received(&mut self, removed_sequence: usize) {
        for (_, partial_message) in self.messages.iter_mut() {
            if partial_message.get_sequence_received() > removed_sequence {
                partial_message.decrement_sequence_received();
            }
        }
    }

    pub fn add_fragment(&mut self, fragment: MessageFragment) -> Option<Vec<u8>> {
        if fragment.get_count() == 1 {
            return Some(fragment.move_buf());
        }

        let id = fragment.get_id();

        // TODO: see if this can be done without time values.  This requires iterating over the whole map, so just use an number to order them
        let partial = match self.messages.get_mut(&id) {
            None => {
                while self.messages.len() >= self.max_messages {
                    let mut oldest_id_and_sequence: Option<(u32, usize)> = None;

                    {
                        for (id, partial) in self.messages.iter() {
                            if let Some((_, current_sequence)) = oldest_id_and_sequence {
                                if current_sequence > partial.get_sequence_received() {
                                    oldest_id_and_sequence = Some((*id, partial.get_sequence_received()));
                                }
                            } else {
                                oldest_id_and_sequence = Some((*id, partial.get_sequence_received()));
                            }
                        }
                    }

                    if let Some((id_to_remove, _)) = oldest_id_and_sequence {
                        let removed_partial = self.messages.remove(&id_to_remove).unwrap();
                        self.decrement_sequence_received(removed_partial.get_sequence_received());
                    }
                }

                let partial_message = PartiallyAssembledFragment::new(self.messages.len(), fragment);

                self.messages.insert(id, partial_message);
                self.messages.get_mut(&id).unwrap()
            }
            Some(partial) => {
                partial.add_fragment(fragment);
                partial
            }
        };

        if partial.has_all_fragments() {
            let completed_message = self.messages.remove(&id).unwrap();
            let sequence_received = completed_message.get_sequence_received();

            self.decrement_sequence_received(sequence_received);

            let message = completed_message.get_full_message();
            return Some(message);
        } else {
            return None;
        }
    }
}

struct PartiallyAssembledFragment {
    id: u32,
    count: u16,
    outstanding_fragments: u16,
    fragments: Vec<Option<MessageFragment>>,

    /// The sequence number, ordering the messages based on when the first
    /// fragment was received.  This is used to drop the oldest partial message
    /// when the assembler fills up to its max number of partial messages
    sequence_received: usize,
}

impl PartiallyAssembledFragment {
    pub fn new(sequence_received: usize, fragment: MessageFragment) -> Self {
        let mut vec = Vec::with_capacity(fragment.get_count() as usize);

        for _i in 0..fragment.get_count() {
            vec.push(None);
        }

        let mut new = Self {
            id: fragment.get_id(),
            count: fragment.get_count(),
            outstanding_fragments: fragment.get_count(),
            fragments: vec,
            sequence_received,
        };

        new.add_fragment(fragment);

        return new;
    }

    fn add_fragment(&mut self, fragment: MessageFragment) {
        if self.fragments[fragment.get_index() as usize].is_none() {
            self.outstanding_fragments = self.outstanding_fragments - 1;
        }

        let index = fragment.get_index() as usize;
        self.fragments[index] = Some(fragment);
    }

    fn has_all_fragments(&self) -> bool {
        return self.outstanding_fragments == 0;
    }

    fn get_full_message(self) -> Vec<u8> {
        let mut length = 0;

        for option in &self.fragments {
            length = length + option.as_ref().unwrap().get_fragment_length();
        }

        let mut buf = Vec::with_capacity(length);

        for option in self.fragments {
            let mut fragment_buf = option.unwrap().move_buf();
            buf.append(&mut fragment_buf);
        }

        return buf;
    }

    fn get_sequence_received(&self) -> usize {
        return self.sequence_received;
    }

    fn decrement_sequence_received(&mut self) {
        self.sequence_received -= 1;
    }

}
