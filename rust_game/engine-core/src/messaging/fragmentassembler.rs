use crate::messaging::MessageFragment;
use commons::factory::FactoryTrait;
use commons::time::TimeValue;
use std::collections::HashMap;

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

    pub fn add_fragment(
        &mut self,
        factory: &impl FactoryTrait,
        fragment: MessageFragment,
    ) -> Option<Vec<u8>> {
        if fragment.get_count() == 1 {
            return Some(fragment.move_buf());
        }

        let id = fragment.get_id();

        let partial = match self.messages.get_mut(&id) {
            None => {
                while self.messages.len() >= self.max_messages {
                    let mut oldest_id: Option<u32> = None;

                    {
                        let mut oldest: Option<&PartiallyAssembledFragment> = None;

                        for (id, partial) in self.messages.iter() {
                            if let Some(current) = oldest {
                                if current
                                    .get_time_of_first_fragment()
                                    .is_after(&partial.get_time_of_first_fragment())
                                {
                                    oldest = Some(partial);
                                    oldest_id = Some(*id);
                                }
                            } else {
                                oldest = Some(partial);
                                oldest_id = Some(*id);
                            }
                        }
                    }
                    if let Some(id_to_remove) = oldest_id {
                        self.messages.remove(&id_to_remove);
                    }
                }

                self.messages
                    .insert(id, PartiallyAssembledFragment::new(factory, fragment));
                self.messages.get_mut(&id).unwrap()
            }
            Some(partial) => {
                partial.add_fragment(fragment);
                partial
            }
        };

        if partial.has_all_fragments() {
            return Some(self.messages.remove(&id).unwrap().get_full_message());
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
    time_of_first_fragment: TimeValue,
}

impl PartiallyAssembledFragment {
    pub fn new(factory: &impl FactoryTrait, fragment: MessageFragment) -> Self {
        let mut vec = Vec::with_capacity(fragment.get_count() as usize);

        for _i in 0..fragment.get_count() {
            vec.push(None);
        }

        let mut new = Self {
            id: fragment.get_id(),
            count: fragment.get_count(),
            outstanding_fragments: fragment.get_count(),
            fragments: vec,
            time_of_first_fragment: factory.now(),
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

    pub fn get_time_of_first_fragment(&self) -> TimeValue {
        return self.time_of_first_fragment;
    }
}
