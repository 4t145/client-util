pub fn ok<U, T, E>(map: impl FnOnce(U) -> Result<T, E>) -> impl FnOnce(U) -> Option<T> {
    move |u| match map(u) {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}
