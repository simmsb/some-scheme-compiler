use std::rc::Rc;

pub fn clone_rc<T: Clone>(r: Rc<T>) -> T {
    Rc::try_unwrap(r).unwrap_or_else(|t| t.as_ref().clone())
}
