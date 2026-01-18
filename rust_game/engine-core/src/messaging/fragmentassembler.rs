use {
    crate::messaging::MessageFragment, serde::{de::DeserializeOwned}, std::collections::HashMap
};

pub struct FragmentAssembler {
    max_messages: usize,
    messages: HashMap<u32, PartiallyAssembledFragment>,
    newest_and_oldest_id: Option<(u32,u32)>
}

//TODO: review buffer allocation in the fragment assembler
impl FragmentAssembler {
    pub fn new(max_messages: usize) -> Self {
        return Self {
            max_messages,
            messages: HashMap::new(),
            newest_and_oldest_id: None
        };
    }

    fn removed_partially_assembled_message(&mut self, id: &u32) -> PartiallyAssembledFragment {
        let partially_assembled_message = self.messages.remove(id).unwrap();

        if let Some(next_id) = partially_assembled_message.next_id {
            self.messages.get_mut(&next_id).unwrap().previous_id = partially_assembled_message.previous_id;
        }

        if let Some(previous_id) = partially_assembled_message.previous_id {
            self.messages.get_mut(&previous_id).unwrap().next_id = partially_assembled_message.next_id;
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

        if fragment.get_count() == 1 {
            //TODO: don't unwrap
            let message =  rmp_serde::from_slice(fragment.move_buf().as_slice()).unwrap();
            return Some(message);
        }

        let id = fragment.get_id();

        let partial = match self.messages.get_mut(&id) {
            None => {
                while self.messages.len() >= self.max_messages {
                    let (_, oldest_id) = self.newest_and_oldest_id.unwrap();
                    self.removed_partially_assembled_message(&oldest_id);
                }

                let next_id = match self.newest_and_oldest_id {
                    Some((newest_id, oldest_id)) => {
                        self.messages.get_mut(&newest_id).unwrap().previous_id = Some(id);
                        self.newest_and_oldest_id = Some((id, oldest_id));
                        Some(newest_id)
                    },
                    None => {
                        self.newest_and_oldest_id = Some((id, id));
                        None
                    },
                };

                let partial_message = PartiallyAssembledFragment::new(next_id, fragment);

                self.messages.insert(id, partial_message);
                self.messages.get_mut(&id).unwrap()
            }
            Some(partial) => {
                partial.add_fragment(fragment);
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
    id: u32,
    count: u16,
    outstanding_fragments: u16,

    //TODO: replace this with a buffer
    fragments: Vec<Option<MessageFragment>>,
    
    /// Points to an older partially assembled message
    next_id: Option<u32>,

    /// Points to a newer partially assembled message
    previous_id: Option<u32>,

    //TODO: allocate this to the right size at the begining
    buf: Vec<u8>
}

impl PartiallyAssembledFragment {
    pub fn new(next_id: Option<u32>, fragment: MessageFragment) -> Self {
        let mut vec = Vec::with_capacity(fragment.get_count() as usize);

        for _i in 0..fragment.get_count() {
            vec.push(None);
        }

        let mut new = Self {
            id: fragment.get_id(),
            count: fragment.get_count(),
            outstanding_fragments: fragment.get_count(),
            fragments: vec,
            next_id,
            previous_id: None,
            buf: Vec::new()
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

    fn get_full_message(&mut self) -> &[u8] {
        let mut length = 0;

        for option in &self.fragments {
            length = length + option.as_ref().unwrap().get_fragment_length();
        }

        // TODO: re-use buffers from partially assembled fragments
        self.buf = Vec::with_capacity(length);

        for option in &self.fragments {
            let fragment_buf = option.as_ref().unwrap().payload();
            self.buf.extend_from_slice(fragment_buf);
        }

        return self.buf.as_slice();
    }
}
