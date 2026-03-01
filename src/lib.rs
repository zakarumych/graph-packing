use core::cmp::Ordering;

/// Packs objects on a 1D line with minimal possible bin length.
///
/// Pairs that are `None` (incomparable) are placed without overlap.
/// Comparable pairs (`Some(_)`) are allowed to overlap.
///
/// This implementation is exact: it explores all left-tight placements and
/// returns one with the smallest maximal end position.
pub fn pack_partial_order<F>(sizes: &[u64], mut partial_order: F) -> Vec<u64>
where
    F: FnMut(usize, usize) -> Option<Ordering>,
{
    let n = sizes.len();
    let mut incomparable = vec![vec![false; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i != j {
                incomparable[i][j] = partial_order(i, j).is_none();
            }
        }
    }

    let mut best_positions = vec![0; n];
    let mut best_span = u64::MAX;
    let mut current_positions = vec![0; n];
    let mut placed = vec![false; n];

    fn interval_end(start: u64, size: u64) -> Option<u64> {
        start.checked_add(size)
    }

    fn search(
        sizes: &[u64],
        incomparable: &[Vec<bool>],
        placed: &mut [bool],
        current_positions: &mut [u64],
        placed_count: usize,
        current_span: u64,
        best_span: &mut u64,
        best_positions: &mut Vec<u64>,
    ) {
        if placed_count == sizes.len() {
            if current_span < *best_span {
                *best_span = current_span;
                *best_positions = current_positions.to_vec();
            }
            return;
        }

        if current_span >= *best_span {
            return;
        }

        for i in 0..sizes.len() {
            if placed[i] {
                continue;
            }

            let mut candidates = vec![0_u64];
            for j in 0..sizes.len() {
                if placed[j] && incomparable[i][j] {
                    if let Some(end) = interval_end(current_positions[j], sizes[j]) {
                        candidates.push(end);
                    }
                }
            }
            candidates.sort_unstable();
            candidates.dedup();

            for &start in &candidates {
                let Some(end) = interval_end(start, sizes[i]) else {
                    continue;
                };
                if end >= *best_span {
                    continue;
                }

                let mut valid = true;
                for j in 0..sizes.len() {
                    if placed[j] && incomparable[i][j] {
                        let other_start = current_positions[j];
                        let Some(other_end) = interval_end(other_start, sizes[j]) else {
                            valid = false;
                            break;
                        };
                        if start < other_end && other_start < end {
                            valid = false;
                            break;
                        }
                    }
                }

                if !valid {
                    continue;
                }

                placed[i] = true;
                current_positions[i] = start;
                search(
                    sizes,
                    incomparable,
                    placed,
                    current_positions,
                    placed_count + 1,
                    current_span.max(end),
                    best_span,
                    best_positions,
                );
                placed[i] = false;
            }
        }
    }

    search(
        sizes,
        &incomparable,
        &mut placed,
        &mut current_positions,
        0,
        0,
        &mut best_span,
        &mut best_positions,
    );

    best_positions
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

    #[test]
    fn finds_global_optimum_not_input_order_greedy() {
        let sizes = [5, 10, 10, 5];
        // 0 < 1, 0 < 2, 3 < 1, with incomparable pairs: (0,3), (1,2), (2,3).
        let positions = pack_partial_order(&sizes, |a, b| match (a, b) {
            (0, 1) | (0, 2) | (3, 1) => Some(Less),
            (1, 0) | (2, 0) | (1, 3) => Some(Greater),
            _ => None,
        });

        let span = positions
            .iter()
            .zip(sizes)
            .map(|(start, size)| start.checked_add(size).unwrap_or(u64::MAX))
            .max()
            .unwrap_or(0);

        assert_eq!(span, 20);
        assert_eq!(positions[2], 10);
        assert_eq!(positions[3], 5);
    }
}
