use core::cmp::Ordering;

/// Packs objects on a 1D line as tightly as possible for the provided iteration order.
///
/// Pairs that are `None` (incomparable) are placed without overlap.
/// Comparable pairs (`Some(_)`) are allowed to overlap.
pub fn pack_partial_order<F>(sizes: &[u64], mut partial_order: F) -> Vec<u64>
where
    F: FnMut(usize, usize) -> Option<Ordering>,
{
    let mut positions = vec![0; sizes.len()];

    for i in 0..sizes.len() {
        let mut position = 0;

        for j in 0..i {
            if partial_order(i, j).is_none() {
                position = position.max(positions[j] + sizes[j]);
            }
        }

        positions[i] = position;
    }

    positions
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering::{Greater, Less};

    use super::pack_partial_order;

    #[test]
    fn chain_can_fully_overlap() {
        let sizes = [5, 3, 7];
        let positions = pack_partial_order(&sizes, |a, b| match a.cmp(&b) {
            Less => Some(Less),
            Greater => Some(Greater),
            _ => None,
        });

        assert_eq!(positions, vec![0, 0, 0]);
    }

    #[test]
    fn incomparable_items_are_separated() {
        let sizes = [2, 4, 3];
        let positions = pack_partial_order(&sizes, |_, _| None);

        assert_eq!(positions, vec![0, 2, 6]);
    }

    #[test]
    fn mixed_partial_order_respects_constraints() {
        let sizes = [5, 4, 3];
        // 0 < 1 and 0 < 2, while 1 and 2 are incomparable.
        let positions = pack_partial_order(&sizes, |a, b| match (a, b) {
            (0, 1) | (0, 2) => Some(Less),
            (1, 0) | (2, 0) => Some(Greater),
            _ => None,
        });

        assert_eq!(positions, vec![0, 0, 4]);
    }
}
