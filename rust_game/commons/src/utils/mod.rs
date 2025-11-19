pub fn simplify_result<T, U>(result: Result<T, U>) -> Result<T, ()> {
    return match result {
        Ok(t) => Ok(t),
        Err(_) => Err(()),
    };
}
