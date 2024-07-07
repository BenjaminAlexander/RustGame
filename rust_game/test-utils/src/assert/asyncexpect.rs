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
    unmet_expectations: Arc<(Mutex<Vec<AsyncExpectObject>>, Condvar)>,
}

impl AsyncExpects {
    pub fn new() -> Self {
        return AsyncExpects {
            unmet_expectations: Arc::new((Mutex::new(Vec::new()), Condvar::new())),
        };
    }

    pub fn new_async_expect<T: Debug + Eq + Send + 'static>(
        &self,
        description: &str,
        expected_value: T,
    ) -> AsyncExpect<T> {
        let (unmet_expectations, _signal) = self.unmet_expectations.deref();

        //let unmet_expectations = UNMET_EXPECTATIONS;
        let mut guard = unmet_expectations.lock().unwrap();

        let async_expect_internal = AsyncExpectInternal {
            description: description.to_string(),
            expected_value,
            state: AsyncExpectState::WaitingForActualValue,
        };

        let async_expect_object = Arc::new(Mutex::new(async_expect_internal));

        guard.push(async_expect_object.clone());

        return AsyncExpect {
            async_expects: self.clone(),
            async_expect_object,
        };
    }

    pub fn wait_for_all(&self) {
        let (unmet_expectations, signal) = self.unmet_expectations.deref();

        let mut unmet_expectations_guard = unmet_expectations.lock().unwrap();

        while unmet_expectations_guard.len() > 0 {
            unmet_expectations_guard = signal.wait(unmet_expectations_guard).unwrap();

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
        let (unmet_expectations, signal) = self.async_expects.unmet_expectations.deref();

        {
            let mut unmet_expectations_guard = unmet_expectations.lock().unwrap();

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
            while i < unmet_expectations_guard.len() {
                let state = unmet_expectations_guard[i]
                    .lock()
                    .unwrap()
                    .get_state();

                if state == AsyncExpectTraitState::ActualMatchesExpected {
                    unmet_expectations_guard.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        signal.notify_all();

        //TODO: panic after notify all
    }

    pub fn wait_for(&self) {
        let (unmet_expectations, signal) = self.async_expects.unmet_expectations.deref();

        let mut unmet_expectations_guard = unmet_expectations.lock().unwrap();

        while !self.is_met() {
            unmet_expectations_guard = signal.wait(unmet_expectations_guard).unwrap();
        }
    }

    fn is_met(&self) -> bool {
        return self.async_expect_object.lock().unwrap().state == AsyncExpectState::ActualMatchesExpected;
    }
}
