pub trait SymmetricCompare {
    fn symmetric_compare(&self, other: &Self) -> bool;
    fn inverse_compare(&self, other: &Self) -> bool;
}

impl SymmetricCompare for (usize, usize) {
    fn symmetric_compare(&self, other: &(usize, usize)) -> bool {
        self == other || self.inverse_compare(other)
    }

    fn inverse_compare(&self, other: &(usize, usize)) -> bool {
        self == &(other.1, other.0)
    }
}
