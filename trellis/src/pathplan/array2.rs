#[derive(Clone, Debug, Default)]
pub struct Array2<T> {
    pub vec: Vec<T>,
    pub rows: usize,
    pub cols: usize,
    pub extra: usize,
    pub row_size: usize,
}

impl<T> Array2<T> {
    pub fn new(rows: usize, cols: usize, extra: usize) -> Self
    where
        T: Default + Copy,
    {
        let row_size = cols + extra;
        Self {
            vec: vec![<T as Default>::default(); row_size * rows],
            rows,
            cols,
            row_size,
            extra,
        }
    }
}

// (row, column)
impl<T> core::ops::Index<(usize, usize)> for Array2<T> {
    type Output = T;
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.vec[row * self.row_size + col]
    }
}

// (row, column)
impl<T> core::ops::IndexMut<(usize, usize)> for Array2<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.vec[row * self.row_size + col]
    }
}

/* allocArray:
 * Allocate a VxV array of COORD values.
 * (array2 is a pointer to an array of pointers; the array is
 * accessed in row-major order.)
 * The values in the array are initialized to 0.
 * Add extra rows.
 */
pub fn allocArray<T: Default + Copy>(V: usize, extra: usize) -> Array2<T> {
    Array2::new(V, V, extra)
}
