use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{
    Arc,
    Condvar,
    Mutex,
};

use log::warn;

type AsyncExpectObject = Arc<Mutex<dyn AsyncExpectTrait>>;

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
            is_expectation_met: false,
        };

        let async_expect_object = Arc::new(Mutex::new(async_expect_internal));

        guard.push(async_expect_object.clone());

        warn!("guard.len(): {:?}", guard.len());

        return AsyncExpect {
            async_expects: self.clone(),
            async_expect_object,
        };
    }

    pub fn wait_for_all(&self) {
        let (unmet_expectations, signal) = self.unmet_expectations.deref();

        let mut unmet_expectations_guard = unmet_expectations.lock().unwrap();

        warn!(
            "unmet_expectations_guard.len(): {:?}",
            unmet_expectations_guard.len()
        );

        while unmet_expectations_guard.len() > 0 {
            unmet_expectations_guard = signal.wait(unmet_expectations_guard).unwrap();

            warn!(
                "unmet_expectations_guard.len(): {:?}",
                unmet_expectations_guard.len()
            );
        }
    }
}

trait AsyncExpectTrait: Send {
    fn is_expectation_met(&self) -> bool;
}

struct AsyncExpectInternal<T: Debug + Eq + 'static> {
    description: String,
    expected_value: T,
    is_expectation_met: bool,
}

impl<T: Debug + Eq + Send + 'static> AsyncExpectTrait for AsyncExpectInternal<T> {
    fn is_expectation_met(&self) -> bool {
        return self.is_expectation_met;
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

                if self_guard.is_expectation_met {
                    panic!(
                        "Expectation has already been met: {:?}",
                        self_guard.description
                    );
                }

                if self_guard.expected_value != actual_value {
                    panic!(
                        "Expectation failed: {:?}\nExpected: {:?}\nActual: {:?}",
                        self_guard.description, self_guard.expected_value, actual_value
                    );
                }

                self_guard.is_expectation_met = true;
            }

            let mut i = 0;
            while i < unmet_expectations_guard.len() {
                let is_expectation_met = unmet_expectations_guard[i]
                    .lock()
                    .unwrap()
                    .is_expectation_met();

                if is_expectation_met {
                    unmet_expectations_guard.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        signal.notify_all();
    }

    pub fn wait_for(&self) {
        let (unmet_expectations, signal) = self.async_expects.unmet_expectations.deref();

        let mut unmet_expectations_guard = unmet_expectations.lock().unwrap();

        while !self.is_met() {
            unmet_expectations_guard = signal.wait(unmet_expectations_guard).unwrap();
        }
    }

    fn is_met(&self) -> bool {
        return self.async_expect_object.lock().unwrap().is_expectation_met;
    }
}
