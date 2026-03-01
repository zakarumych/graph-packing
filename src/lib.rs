use core::cmp::Ordering;

/// Packs objects on a 1D line while respecting incomparability constraints.
///
/// Pairs that are `None` (incomparable) are placed without overlap.
/// Comparable pairs (`Some(_)`) are allowed to overlap.
///
/// This implementation is exact: it explores all left-tight placements and
/// places each item (in input order) at the earliest valid start position.
///
/// Complexity (`n = sizes.len()`):
/// - Time: `O(n^2 log n)` (`O(n^2)` incomparability preprocessing plus, for each
///   item, sorting already-placed incomparable intervals to find the first gap).
/// - Space: `O(n^2)` for the incomparability matrix and `O(n)` temporary interval
///   storage.
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

    let mut positions = vec![0_u64; n];
    let mut intervals = Vec::new();

    for i in 0..n {
        intervals.clear();

        for j in 0..i {
            if incomparable[i][j] {
                let Some(end) = positions[j].checked_add(sizes[j]) else {
                    continue;
                };
                intervals.push((positions[j], end));
            }
        }

        intervals.sort_unstable_by_key(|&(start, _)| start);

        let mut start = 0_u64;
        for (other_start, other_end) in intervals.iter().copied() {
            let Some(end) = start.checked_add(sizes[i]) else {
                break;
            };
            if end <= other_start {
                break;
            }
            if start < other_end {
                start = other_end;
            }
        }

        positions[i] = start;
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
    fn total_order_30_elements_all_overlap() {
        let sizes = vec![1_u64; 30];
        let positions = pack_partial_order(&sizes, |a, b| match a.cmp(&b) {
            Less => Some(Less),
            Greater => Some(Greater),
            _ => None,
        });

        assert_eq!(positions, vec![0; sizes.len()]);
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

    #[test]
    #[ignore = "performance measurement; run with --ignored --nocapture"]
    fn pseudorandom_100_elements_reports_execution_time() {
        const DETERMINISTIC_SEED: u64 = 0x1234_5678_9ABC_DEF0;
        const MIN_SIZE: u64 = 10;
        const SIZE_RANGE: u64 = 991; // inclusive 10..=1000

        let mut seed = DETERMINISTIC_SEED;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            seed
        };
        let sizes = (0..100)
            .map(|_| MIN_SIZE + (next() % SIZE_RANGE))
            .collect::<Vec<_>>();
        assert!(sizes.windows(2).any(|w| w[0] != w[1]));
        let ranks = (0..100).map(|_| next()).collect::<Vec<_>>();
        let relation = |a: usize, b: usize| {
            if a == b {
                return None;
            }
            match ranks[a].cmp(&ranks[b]) {
                Less => Some(Less),
                Greater => Some(Greater),
                _ => None,
            }
        };
        for i in 0..ranks.len() {
            for j in (i + 1)..ranks.len() {
                assert!(relation(i, j).is_some());
            }
        }
        let started = std::time::Instant::now();
        let positions = pack_partial_order(&sizes, relation);
        let elapsed = started.elapsed();
        eprintln!("pseudorandom_100_elements_reports_execution_time: {elapsed:?}");

        assert_eq!(positions, vec![0; sizes.len()]);
    }
}
