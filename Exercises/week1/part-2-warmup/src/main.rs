/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

/// A simple add_n function for vector
/// # Arguments
///
/// * `v` - A vector of numbers
/// * `n` - Some numbers
///
/// # Return
/// A new vector whose elements are the numbers in the original vector v with n added to each number
fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    let mut res = vec![];
    for element in v.iter() {
        res.push(element + n);
    }
    res
}

/// A simple add_n function for vector
/// # Arguments
///
/// * `v` - A vector of numbers
/// * `n` - Some numbers
///
/// # Return
/// Do the same thing as `add_n`, but modifies v directly (in place) and does not return anything
fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    for element in v.iter_mut() {
        // we will modify the `element`, so we need a mutable reference
        // type(element): &mut i32, so we use * to deref
        *element = *element + n;
    }
}

fn dedup(v: &mut Vec<i32>) {
    let mut hashset_helper = HashSet::new();
    v.retain(|e| hashset_helper.insert(*e));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
