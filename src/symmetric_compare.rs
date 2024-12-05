use std::rc::Rc;

pub trait SymmetricCompare {
    fn symmetric_compare(&self, other: &Self) -> bool;
    fn inverse_compare(&self, other: &Self) -> bool;
}

impl SymmetricCompare for (usize, usize) {
    fn symmetric_compare(&self, other: &(usize, usize)) -> bool {
        self == other || self.inverse_compare(other)
    }

    fn inverse_compare(&self, other: &(usize, usize)) -> bool {
        self == &other.flipped()
    }
}

impl<T> SymmetricCompare for (Rc<T>, Rc<T>) {
    fn symmetric_compare(&self, other: &(Rc<T>, Rc<T>)) -> bool {
        Rc::ptr_eq(&self.0, &other.0) && Rc::ptr_eq(&self.1, &other.1)
            || self.inverse_compare(other)
    }

    fn inverse_compare(&self, other: &(Rc<T>, Rc<T>)) -> bool {
        Rc::ptr_eq(&self.0, &other.1) && Rc::ptr_eq(&self.1, &other.0)
    }
}

pub trait Flipped {
    fn flipped(&self) -> (usize, usize);
}

impl Flipped for (usize, usize) {
    fn flipped(&self) -> (usize, usize) {
        (self.1, self.0)
    }
}

pub trait TupleOrdered {
    fn ordered(&self) -> (usize, usize);
}

impl TupleOrdered for (usize, usize) {
    fn ordered(&self) -> (usize, usize) {
        if self.0 < self.1 {
            *self
        } else {
            (self.1, self.0)
        }
    }
}
