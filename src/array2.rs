use ndarray::Array2;

pub fn collect_array2<T: Default>(iter: impl Iterator<Item = Vec<T>>) -> Option<Array2<T>> {
    let rows: Vec<_> = iter.collect();
    let ncols = rows.iter().map(|row| row.len()).max()?;
    if ncols == 0 {
        return None;
    }
    let nrows = rows.len();
    let mut array = Array2::default((ncols, nrows));
    for (row, vec) in rows.into_iter().enumerate() {
        for (col, value) in vec.into_iter().enumerate() {
            array[(col, row)] = value;
        }
    }
    Some(array)
}
