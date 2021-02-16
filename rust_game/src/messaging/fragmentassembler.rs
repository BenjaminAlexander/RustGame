use crate::messaging::MessageFragment;
use std::collections::HashMap;

pub struct FragmentAssembler {
    //TODO: implement max size
    max_messages: usize,
    messages: HashMap<u32, PartiallyAssembledFragment>
}

impl FragmentAssembler {
    pub fn new(max_messages: usize) -> Self {
        return Self{
            max_messages,
            messages: HashMap::new()
        };
    }

    pub fn add_fragment(&mut self, fragment: MessageFragment) -> Option<Vec<u8>> {

        //TODO:short circuit 1 of 1

        let id = fragment.get_id();

        let partial = match self.messages.get_mut(&id) {
            None => {
                self.messages.insert(id, PartiallyAssembledFragment::new(fragment));
                self.messages.get_mut(&id).unwrap()
            },
            Some(partial) => {
                partial.add_fragment(fragment);
                partial
            },
        };

        if partial.has_all_fragments() {
            return Some(self.messages.remove(&id).unwrap().get_full_message());
        } else {
            return None;
        }

    }

}

pub struct PartiallyAssembledFragment {
    id: u32,
    count: u16,
    outstanding_fragments: u16,
    fragments: Vec<Option<MessageFragment>>
}

impl PartiallyAssembledFragment {

    pub fn new(fragment: MessageFragment) -> Self {
        let mut vec = Vec::with_capacity(fragment.get_count() as usize);

        for i in 0..fragment.get_count() {
            vec.push(None);
        }

        let mut new = Self {
            id: fragment.get_id(),
            count: fragment.get_count(),
            outstanding_fragments: fragment.get_count(),
            fragments: vec
        };

        new.add_fragment(fragment);

        return new;
    }

    pub fn add_fragment(&mut self, fragment: MessageFragment) {
        if self.fragments[fragment.get_index() as usize].is_none() {
            self.outstanding_fragments = self.outstanding_fragments - 1;
        }

        let index = fragment.get_index() as usize;
        self.fragments[index] = Some(fragment);
    }

    pub fn has_all_fragments(&self) -> bool {
        return self.outstanding_fragments == 0;
    }

    pub fn get_full_message(self) -> Vec<u8> {

        let mut length = 0;

        for option in &self.fragments {
            length = length + option.as_ref().unwrap().get_buf().len();
        }

        let mut buf = Vec::with_capacity(length);

        for option in self.fragments {
            let mut fragment_buf = option.unwrap().move_buf();
            buf.append(&mut fragment_buf);
        };

        return buf;
    }
}