pub trait Consumer<T>: Send + 'static {

    //TODO: should accept return a Result?
    //That would let consumers notify producers that they are dead
    fn accept(&self, t: T);
}

pub struct ConsumerList<T>
    where T: Clone + 'static {
    consumers: Vec<Box<dyn Consumer<T>>>
}

impl<T> ConsumerList<T>
    where T: Clone + 'static {

    pub fn new() -> Self {
        Self {consumers: Vec::new()}
    }

    pub fn add_consumer<U>(&mut self, u: U)
        where U: Consumer<T> {

        self.consumers.push(Box::new(u));
    }

    pub fn accept(&self, t: &T) {
        for consumer in &self.consumers {
            consumer.accept(t.clone());
        }
    }
}