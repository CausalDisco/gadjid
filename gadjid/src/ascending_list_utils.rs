// SPDX-License-Identifier: MPL-2.0

use std::cmp::Ordering;

/// Returns the first element that is in both lists, or None if there is no such element.
pub fn ascending_lists_first_shared_element(a: &[usize], b: &[usize]) -> Option<usize> {
    let (mut i, mut j) = (0, 0);

    loop {
        // if we reached the end of one of the two lists without returning yet
        if a.len() == i || b.len() == j {
            // return None as there is then no shared element
            return None;
        } else if a[i] == b[j] {
            // found the first element that is in both lists, return that
            return Some(a[i]);
        } else {
            // if they are not equal, we increment the index of the smaller one
            if a[i] < b[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
}

/// symmetric difference between two sorted vectors (ascending), with duplicates removed. Result also sorted.
pub fn ascending_lists_set_symmetric_difference<T>(
    mut v1: impl Iterator<Item = T>,
    mut v2: impl Iterator<Item = T>,
) -> Vec<T>
where
    T: Ord + Eq + Copy,
{
    let mut diff = Vec::new();

    // while both lists are not yet exhausted
    // take first element, compare.
    let mut e1: Option<T> = v1.next();
    let mut e2: Option<T> = v2.next();
    while let (Some(e1_val), Some(e2_val)) = (e1, e2) {
        let last_added = diff.last();
        let to_add;

        // if they are the same, advance both
        match e1_val.cmp(&e2_val) {
            Ordering::Equal => {
                e1 = v1.next();
                e2 = v2.next();
                continue;
            }
            Ordering::Less => {
                to_add = e1_val;
                e1 = v1.next();
            }
            Ordering::Greater => {
                to_add = e2_val;
                e2 = v2.next();
            }
        }
        // otherwise we take the smallest and advance that one
        if let Some(n) = last_added {
            assert!(to_add >= *n);
            // but we don't add it if it's the same as the last one we added
            if to_add != *n {
                diff.push(to_add);
            }
        } else {
            diff.push(to_add);
        }
    }

    // add any remaining elements after one list is exhausted
    while let Some(e1_val) = e1 {
        let last_added = diff.last();
        e1 = v1.next();
        if let Some(n) = last_added {
            if e1_val != *n {
                diff.push(e1_val);
            }
        } else {
            diff.push(e1_val);
        }
    }
    while let Some(e2_val) = e2 {
        let last_added = diff.last();
        e2 = v2.next();
        if let Some(n) = last_added {
            if e2_val != *n {
                diff.push(e2_val);
            }
        } else {
            diff.push(e2_val);
        }
    }

    diff
}

/// union of two sorted vectors (ascending), with duplicates removed. Result also sorted.
pub fn ascending_lists_set_union<T>(
    mut v1: impl Iterator<Item = T>,
    mut v2: impl Iterator<Item = T>,
) -> Vec<T>
where
    T: Ord + Eq + Copy,
{
    let mut union = Vec::new();

    // while both lists are not yet exhausted
    // take first element, compare.
    let mut e1_next: Option<T> = v1.next();
    let mut e2_next: Option<T> = v2.next();
    while let (Some(e1), Some(e2)) = (e1_next, e2_next) {
        let last_added = union.last();
        let to_add;
        match e1_next.cmp(&e2_next) {
            Ordering::Equal => {
                to_add = e1;
                e1_next = v1.next();
                e2_next = v2.next();
            }
            Ordering::Less => {
                to_add = e1;
                e1_next = v1.next();
            }
            Ordering::Greater => {
                to_add = e2;
                e2_next = v2.next();
            }
        }
        if let Some(n) = last_added {
            assert!(to_add >= *n);

            if to_add != *n {
                union.push(to_add);
            }
        } else {
            union.push(to_add);
        }
    }

    while let Some(e1) = e1_next {
        let last_added = union.last();
        e1_next = v1.next();
        if let Some(n) = last_added {
            if e1 != *n {
                union.push(e1);
            }
        } else {
            union.push(e1);
        }
    }
    while let Some(e2) = e2_next {
        let last_added = union.last();
        e2_next = v2.next();
        if let Some(n) = last_added {
            if e2 != *n {
                union.push(e2);
            }
        } else {
            union.push(e2);
        }
    }

    union
}

#[cfg(test)]
mod test {
    use crate::ascending_list_utils::{
        ascending_lists_first_shared_element, ascending_lists_set_union,
    };

    use super::ascending_lists_set_symmetric_difference;

    #[test]
    fn first_shared() {
        assert!(ascending_lists_first_shared_element(&[], &[]).is_none());
        assert!(ascending_lists_first_shared_element(&[1], &[]).is_none());
        assert!(ascending_lists_first_shared_element(&[1, 2], &[]).is_none());
        assert!(ascending_lists_first_shared_element(&[1, 2], &[3, 4]).is_none());
        assert!(ascending_lists_first_shared_element(&[1, 2, 5], &[3, 4, 5]) == Some(5));
        assert!(ascending_lists_first_shared_element(&[1, 2, 5], &[3, 4, 5, 10]) == Some(5));
    }

    #[test]
    fn symmetric_difference_test() {
        let res = ascending_lists_set_symmetric_difference(
            [1, 2, 4, 6].into_iter(),
            [0, 0, 1, 2, 3, 5, 6, 7].into_iter(),
        );
        assert_eq!([0, 3, 4, 5, 7], res.as_slice());
    }

    #[test]
    fn set_union_test() {
        let res =
            ascending_lists_set_union([1, 2, 4, 6].into_iter(), [0, 1, 2, 3, 5, 6, 7].into_iter());
        assert_eq!([0, 1, 2, 3, 4, 5, 6, 7], res.as_slice());
    }
}
