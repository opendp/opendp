use crate::{error::Fallible, traits::samplers::Shuffle};

pub(super) fn select_contributions<T: Clone>(
    arg: &[T],
    limit: usize,
    ordered: bool,
) -> Fallible<Vec<T>> {
    if arg.len() <= limit || ordered || limit == 0 {
        return Ok(arg[..arg.len().min(limit)].to_vec());
    }
    // Couple samples by assigning the same random priorities to shared records.
    // An insertion changes the sample by at most one addition or replacement.
    let mut values = arg.to_vec();
    values.shuffle()?;
    values.truncate(limit);
    Ok(values)
}
