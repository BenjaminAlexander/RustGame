use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{
    Arc,
    Condvar,
    Mutex,
};

use log::error;

//TODO: Add a wait_or_timeout variant to panic if expects are not met by a timeout
//TODO: add useful logging or panic message about outstanding expects when a timeout occurs

type AsyncExpectObject = Arc<Mutex<dyn AsyncExpectTrait>>;

#[derive(PartialEq, Eq)]
enum AsyncExpectTraitState {
    WaitingForActualValue,
    ActualMatchesExpected,
    ActualDoesNotMatchExpected,
    FailedSinceAlreadyMet    
}

#[derive(Clone)]
pub struct AsyncExpects {
    internal: Arc<AsyncExpectsInternal>,
}

#[derive(Default)]
struct AsyncExpectsInternal {
    signal: Condvar,
    collection: Mutex<AsyncExpectsCollection>
}

#[derive(Default)]
struct AsyncExpectsCollection {
    unmet_expectations: Vec<AsyncExpectObject>,
    failed_expectations: Vec<AsyncExpectObject>
}

impl AsyncExpects {
    pub fn new() -> Self {
        return AsyncExpects {
            internal: Arc::default()
        };
    }

    pub fn new_async_expect<T: Debug + Eq + Send + 'static>(
        &self,
        description: &str,
        expected_value: T,
    ) -> AsyncExpect<T> {
        let internal = self.internal.deref();

        //let unmet_expectations = UNMET_EXPECTATIONS;
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

    pub fn wait_for_all(&self) {
        let internal = self.internal.deref();

        let mut collection_guard = internal.collection.lock().unwrap();

        while collection_guard.unmet_expectations.len() > 0 {
            collection_guard = internal.signal.wait(collection_guard).unwrap();

        }
    }
}

trait AsyncExpectTrait: Send {
    fn get_state(&self) -> AsyncExpectTraitState;
}

#[derive(PartialEq, Eq)]
enum AsyncExpectState<T: Debug + Eq + 'static> {
    WaitingForActualValue,
    ActualMatchesExpected,
    ActualDoesNotMatchExpected(T),
    FailedSinceAlreadyMet    
}
struct AsyncExpectInternal<T: Debug + Eq  + 'static> {
    description: String,
    expected_value: T,
    state: AsyncExpectState<T>,
}

impl<T: Debug + Eq + Send + 'static> AsyncExpectTrait for AsyncExpectInternal<T> {
    fn get_state(&self) -> AsyncExpectTraitState {
        return match self.state {
            AsyncExpectState::WaitingForActualValue => AsyncExpectTraitState::WaitingForActualValue,
            AsyncExpectState::ActualMatchesExpected => AsyncExpectTraitState::ActualMatchesExpected,
            AsyncExpectState::ActualDoesNotMatchExpected(_) => AsyncExpectTraitState::ActualDoesNotMatchExpected,
            AsyncExpectState::FailedSinceAlreadyMet => AsyncExpectTraitState::FailedSinceAlreadyMet,
        };
    }
}

#[derive(Clone)]
pub struct AsyncExpect<T: Debug + Eq + Send + 'static> {
    async_expects: AsyncExpects,
    async_expect_object: Arc<Mutex<AsyncExpectInternal<T>>>,
}

impl<T: Debug + Eq + Send + 'static> AsyncExpect<T> {
    pub fn set_actual(&self, actual_value: T) {
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
                            self_guard.state = AsyncExpectState::ActualDoesNotMatchExpected(actual_value);

                            //TODO: don't panic
                            panic!();
                        }
                    },
                    AsyncExpectState::ActualMatchesExpected => {
                        self_guard.state = AsyncExpectState::FailedSinceAlreadyMet;
                        //TODO: don't panic
                        panic!();
                    },
                    AsyncExpectState::ActualDoesNotMatchExpected(_) => {
                        return;
                    },
                    AsyncExpectState::FailedSinceAlreadyMet => {
                        return;
                    },
                }

                /*
                if self_guard.is_expectation_met {

                    panic!(
                        "Expectation has already been met: {:?}",
                        self_guard.description
                    );

                }

                format!()

                if self_guard.expected_value != actual_value {

                    error!(
                        "Expectation failed: {:?}\nExpected: {:?}\nActual: {:?}",
                        self_guard.description, self_guard.expected_value, actual_value
                    );

                    panic!(
                        "Expectation failed: {:?}\nExpected: {:?}\nActual: {:?}",
                        self_guard.description, self_guard.expected_value, actual_value
                    );
                }
                */
                
            }

            let mut i = 0;
            while i < collection_guard.unmet_expectations.len() {
                let state = collection_guard.unmet_expectations[i]
                    .lock()
                    .unwrap()
                    .get_state();

                if state == AsyncExpectTraitState::ActualMatchesExpected {
                    collection_guard.unmet_expectations.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        internal.signal.notify_all();

        //TODO: panic after notify all
    }

    pub fn wait_for(&self) {
        let internal = self.async_expects.internal.deref();

        let mut collection_guard = internal.collection.lock().unwrap();

        while !self.is_met() {
            collection_guard = internal.signal.wait(collection_guard).unwrap();
        }
    }

    fn is_met(&self) -> bool {
        return self.async_expect_object.lock().unwrap().state == AsyncExpectState::ActualMatchesExpected;
    }
}
