use commons::threading::AsyncJoinCallBackTrait;
use log::{
    error,
    info,
};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::addr_eq;
use std::sync::{
    Arc,
    Condvar,
    Mutex,
};

//TODO: Add a wait_or_timeout variant to panic if expects are not met by a timeout
//TODO: add useful logging or panic message about outstanding expects when a timeout occurs

type AsyncExpectObject = Arc<Mutex<dyn AsyncExpectTrait>>;

#[derive(PartialEq, Eq, Clone, Debug)]
enum AsyncExpectState {
    WaitingForActualValue,
    ActualMatchesExpected,
    Failed(String),
}

impl AsyncExpectState {
    fn panic_if_failed(&self) {
        if let AsyncExpectState::Failed(message) = self {
            panic!("{message}");
        }
    }
}

#[derive(Clone)]
pub struct AsyncExpects {
    internal: Arc<AsyncExpectsInternal>,
}

#[derive(Default)]
struct AsyncExpectsInternal {
    signal: Condvar,
    collection: Mutex<AsyncExpectsCollection>,
}

#[derive(Default)]
struct AsyncExpectsCollection {
    unmet_expectations: Vec<AsyncExpectObject>,
    failed_expectations: Vec<AsyncExpectObject>,
}

impl AsyncExpects {
    pub fn new() -> Self {
        return AsyncExpects {
            internal: Arc::default(),
        };
    }

    pub fn new_async_expect<T: Debug + Eq + Send + 'static>(
        &self,
        description: &str,
        expected_value: T,
    ) -> AsyncExpect<T> {
        let internal = self.internal.deref();

        let mut guard = internal.collection.lock().unwrap();

        let async_expect_internal = AsyncExpectInternal {
            description: description.to_string(),
            expected_value,
            state: AsyncExpectState::WaitingForActualValue,
        };

        let async_expect_object = Arc::new(Mutex::new(async_expect_internal));

        guard.unmet_expectations.push(async_expect_object.clone());

        return AsyncExpect {
            async_expects: self.clone(),
            async_expect_object,
        };
    }

    pub fn new_expect_async_join<T: Send + 'static>(
        &self,
        description: &str,
    ) -> ExpectAsyncJoin<T> {
        return ExpectAsyncJoin {
            async_expect: self.new_async_expect(description, ()),
            phantom: PhantomData,
        };
    }

    pub fn wait_for_all(&self) {
        let internal = self.internal.deref();

        let mut collection_guard = internal.collection.lock().unwrap();

        while collection_guard.unmet_expectations.len() > 0
            && collection_guard.failed_expectations.len() == 0
        {
            collection_guard = internal.signal.wait(collection_guard).unwrap();
        }

        if collection_guard.failed_expectations.len() != 0 {
            collection_guard
                .failed_expectations
                .get(0)
                .unwrap()
                .lock()
                .unwrap()
                .get_state()
                .panic_if_failed();
        }
    }
}

trait AsyncExpectTrait: Send {
    fn get_state(&self) -> &AsyncExpectState;
}

struct AsyncExpectInternal<T: Debug + Eq + 'static> {
    description: String,
    expected_value: T,
    state: AsyncExpectState,
}

impl<T: Debug + Eq + Send + 'static> AsyncExpectTrait for AsyncExpectInternal<T> {
    fn get_state(&self) -> &AsyncExpectState {
        return &self.state;
    }
}

#[derive(Clone)]
pub struct AsyncExpect<T: Debug + Eq + Send + 'static> {
    async_expects: AsyncExpects,
    async_expect_object: Arc<Mutex<AsyncExpectInternal<T>>>,
}

impl<T: Debug + Eq + Send + 'static> AsyncExpect<T> {
    pub fn set_actual(&self, actual_value: T) {
        let state_clone;

        let internal = self.async_expects.internal.deref();

        {
            let mut collection_guard = internal.collection.lock().unwrap();

            {
                let mut self_guard = self.async_expect_object.lock().unwrap();

                match self_guard.state {
                    AsyncExpectState::WaitingForActualValue => {
                        if self_guard.expected_value == actual_value {
                            self_guard.state = AsyncExpectState::ActualMatchesExpected;
                        } else {
                            let message = format!(
                                "Expected Value does not match actual value: {:?}\nExpected: {:?}\nActual: {:?}",
                                self_guard.description, self_guard.expected_value, actual_value
                            );

                            error!("{message}");

                            self_guard.state = AsyncExpectState::Failed(message);
                        }
                    }
                    AsyncExpectState::ActualMatchesExpected => {
                        let message = format!(
                            "Expectation has already been met: {:?}",
                            self_guard.description
                        );

                        error!("{message}");

                        self_guard.state = AsyncExpectState::Failed(message);
                    }
                    AsyncExpectState::Failed(_) => {
                        return;
                    }
                }

                state_clone = self_guard.get_state().clone();
            }

            let mut i = 0;
            while i < collection_guard.unmet_expectations.len() {
                if addr_eq(
                    Arc::as_ptr(&collection_guard.unmet_expectations[i]),
                    Arc::as_ptr(&self.async_expect_object),
                ) {
                    collection_guard.unmet_expectations.remove(i);
                    break;
                } else {
                    i += 1
                }
            }

            if let AsyncExpectState::Failed(_) = state_clone {
                collection_guard
                    .failed_expectations
                    .push(self.async_expect_object.clone());
            }
        }

        internal.signal.notify_all();

        state_clone.panic_if_failed();
    }

    pub fn wait_for(&self) {
        let internal = self.async_expects.internal.deref();

        let mut collection_guard = internal.collection.lock().unwrap();

        let mut state = self.async_expect_object.lock().unwrap().get_state().clone();

        while state == AsyncExpectState::WaitingForActualValue {
            collection_guard = internal.signal.wait(collection_guard).unwrap();
            state = self.async_expect_object.lock().unwrap().get_state().clone();
        }

        state.panic_if_failed();
    }
}

pub struct ExpectAsyncJoin<T: Send + 'static> {
    async_expect: AsyncExpect<()>,
    phantom: PhantomData<T>,
}

impl<T: Send> AsyncJoinCallBackTrait<T> for ExpectAsyncJoin<T> {
    fn join(self, async_join: commons::threading::AsyncJoin<T>) {
        info!(
            "Thread Name: {:?} joined as expected.",
            async_join.get_thread_name()
        );
        self.async_expect.set_actual(());
    }
}
