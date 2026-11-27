use crate::error::Fallible;

/// A heuristic score together with the actual source bound at its argument.
#[derive(Clone, Copy)]
pub(super) struct Candidate {
    pub arg: f64,
    pub bound: f64,
    pub score: f64,
}

/// Refine local maxima for many objectives, sharing the source evaluation at
/// each round. The brackets and scores are heuristic; callers certify their
/// chosen candidates using the associated source bounds.
pub(super) fn refine_maxima(
    mut brackets: Vec<(usize, f64, f64)>,
    best: &mut [Candidate],
    evaluate: impl Fn(&[f64]) -> Fallible<Vec<f64>>,
    score: impl Fn(usize, f64, f64) -> Fallible<f64>,
) -> Fallible<()> {
    let probes = |lo: f64, hi: f64| {
        let left = lo + (hi - lo) / 3.0;
        let right = hi - (hi - lo) / 3.0;
        (lo < left && left < right && right < hi).then_some((left, right))
    };

    // A finite refinement budget also terminates searches on flat objectives
    // with a bracket touching zero.
    for _ in 0..80 {
        let mut points = brackets
            .iter()
            .filter_map(|&(_, lo, hi)| probes(lo, hi))
            .flat_map(|(left, right)| [left, right])
            .collect::<Vec<_>>();
        if points.is_empty() {
            break;
        }
        points.sort_by(f64::total_cmp);
        points.dedup();
        let bounds = evaluate(&points)?;
        for (query, lo, hi) in &mut brackets {
            let Some((left, right)) = probes(*lo, *hi) else {
                continue;
            };
            let candidate = |arg: f64| -> Fallible<Candidate> {
                let bound = bounds[points.binary_search_by(|p| p.total_cmp(&arg)).unwrap()];
                Ok(Candidate {
                    arg,
                    bound,
                    score: score(*query, arg, bound)?,
                })
            };
            let left = candidate(left)?;
            let right = candidate(right)?;
            for candidate in [left, right] {
                if candidate.score > best[*query].score {
                    best[*query] = candidate;
                }
            }
            if left.score < right.score {
                *lo = left.arg;
            } else {
                *hi = right.arg;
            }
        }
    }
    Ok(())
}
