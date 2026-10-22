use super::*;
use crate::{
    core::Domain,
    traits::{
        CastInternalRational, Float, InfCast,
        sum::{sum_error, sum_relaxation},
    },
};
use dashu::{integer::IBig, rational::RBig};
use num::{One, Signed};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

#[test]
fn integer_types_and_invariants() -> Fallible<()> {
    macro_rules! check { ($($t:ty)+) => {$ (
        let domain = VectorDomain::new(AtomDomain::new_closed((0 as $t, 10 as $t))?);
        let sum = make_sum::<_, $t>(domain.clone(), SymmetricDistance)?;
        assert_eq!(sum.invoke(&vec![1, 2, 3, 4, 5])?, 15);
        assert_eq!(sum.invoke(&vec![5, 4, 1, 3, 2])?, 15);
        assert_eq!(sum.map(&2)?, 20);
        let fixed = make_sum::<_, $t>(domain.with_size(5), InsertDeleteDistance)?;
        assert_eq!(fixed.map(&1)?, 0);
        assert_eq!(fixed.map(&2)?, 10);
        assert_eq!(fixed.invoke(&vec![1, 2, 3, 4, 5])?, 15);
    )+}; }
    check! {u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize}
    macro_rules! signed { ($($t:ty)+) => {$ (
        let domain = VectorDomain::new(AtomDomain::new_closed((<$t>::MIN, 0))?).with_size(1);
        let sum = make_sum::<_, $t>(domain, SymmetricDistance)?;
        assert_eq!(sum.invoke(&vec![<$t>::MIN])?, <$t>::MIN);
        assert_eq!(sum.map(&0)?, 0);
        assert!(sum.map(&2).is_err()); // public distance cannot represent |MIN|
        let domain = VectorDomain::new(AtomDomain::new_closed((-10 as $t, 10 as $t))?).with_size(4);
        let sum = make_sum::<_, $t>(domain, InsertDeleteDistance)?;
        assert_eq!(sum.invoke(&vec![-10, 9, -2, 8])?, 5);
        assert_eq!(sum.invoke(&vec![8, -2, 9, -10])?, 5);
    )+}; }
    signed! {i8 i16 i32 i64 i128 isize}
    // The final mathematical sum of [100, 100, -100] fits, but a prefix does not.
    let domain = VectorDomain::new(AtomDomain::new_closed((-100i8, 100))?);
    assert!(make_sum::<_, i8>(domain.clone().with_size(3), InsertDeleteDistance).is_err());
    assert!(make_sum::<_, i8>(domain.with_size(3), SymmetricDistance).is_err());
    let domain = VectorDomain::new(AtomDomain::new_closed((0u8, 128))?).with_size(2);
    assert!(make_sum::<_, u8>(domain, SymmetricDistance).is_err());
    // Length itself need not fit the output type.
    let domain = VectorDomain::new(AtomDomain::new_closed((0u8, 0))?);
    assert_eq!(
        make_sum::<_, u8>(domain, SymmetricDistance)?.invoke(&vec![])?,
        0
    );
    Ok(())
}

#[test]
fn integer_size_limits() -> Fallible<()> {
    for (bounds, expected) in [
        ((-1i32, 1), 2_147_483_647),
        ((-10, 10), 214_748_364),
        ((0, 100), 21_474_836),
        ((1, 100), 21_474_836),
        ((-100, -1), 21_474_836),
        ((-100, 1), 21_474_836),
        ((-1, 0), 2_147_483_648),
        ((i32::MIN, 0), 1),
        ((i32::MIN, i32::MIN), 1),
        ((0, 0), usize::MAX),
    ] {
        assert_eq!(int::size_limit(bounds), expected);
        assert!(!int::can_int_sum_overflow(expected, bounds));
        if expected < usize::MAX {
            assert!(int::can_int_sum_overflow(expected + 1, bounds));
        }
    }
    assert_eq!(int::size_limit((0i64, 0)), usize::MAX);
    assert_eq!(int::size_limit((0u128, 1)), usize::MAX);
    assert_eq!(int::size_limit((0u128, u128::MAX)), 1);
    assert_eq!(int::size_limit((i128::MIN, 0)), 1);
    assert_eq!(int::size_limit((-1i128, 0)), usize::MAX);
    // Validate exact sizes immediately below, at and above a small safe limit.
    let domain = VectorDomain::new(AtomDomain::new_closed((-10i8, 10))?);
    for n in [11, 12] {
        let sum = make_sum(domain.clone().with_size(n), SymmetricDistance)?;
        assert_eq!(sum.invoke(&vec![10; n])?, (10 * n) as i8);
        assert_eq!(sum.map(&2)?, 20);
    }
    assert!(make_sum(domain.with_size(13), SymmetricDistance).is_err());
    // All integer representations, including unsigned and signed extrema.
    macro_rules! check { ($($t:ty)+) => {$ (
        let bounds = (0 as $t, <$t>::MAX);
        assert_eq!(int::size_limit(bounds), 1);
        let domain = VectorDomain::new(AtomDomain::new_closed(bounds)?);
        for n in [0, 1] {
            let sum = make_sum(domain.clone().with_size(n), InsertDeleteDistance)?;
            assert_eq!(sum.invoke(&vec![<$t>::MAX; n])?, if n == 0 { 0 } else { <$t>::MAX });
        }
        assert!(make_sum(domain.clone().with_size(2), InsertDeleteDistance).is_err());
        assert_eq!(make_sum(domain, InsertDeleteDistance)?.invoke(&vec![<$t>::MAX; 2])?, <$t>::MAX);
    )+}; }
    check! {u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize}
    Ok(())
}

#[test]
fn integer_overflow_checks_match_products() {
    macro_rules! check { ($($t:ty)+) => {$ (
        let values = [<$t>::MIN, <$t>::MIN / 2, <$t>::MIN + 1, 0, 1, 10, <$t>::MAX / 2, <$t>::MAX];
        for lower in values {
            for upper in values.into_iter().filter(|upper| *upper >= lower) {
                let bounds = (lower, upper);
                let limit = int::size_limit(bounds);
                for size in [0, 1, 2, limit.saturating_sub(1), limit, limit.saturating_add(1), usize::MAX] {
                    let n = IBig::from(size);
                    let partial_min = (IBig::from(lower) * &n).min(IBig::ZERO);
                    let partial_max = (IBig::from(upper) * n).max(IBig::ZERO);
                    let product_check = partial_min < IBig::from(<$t>::MIN)
                        || partial_max > IBig::from(<$t>::MAX);
                    assert_eq!(int::can_int_sum_overflow(size, bounds), product_check,
                        "{} bounds={bounds:?}, size={size}", stringify!($t));
                }
            }
        }
    )+}; }
    check! {u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize}
}

#[test]
fn inferred_types_and_float_default_cap() -> Fallible<()> {
    let n = DEFAULT_SIZE_LIMIT;
    let values = vec![1i32; n + 1];
    let domain = VectorDomain::new(AtomDomain::new_closed((0i32, 1))?);
    let sum = make_sum(domain.clone(), InsertDeleteDistance)?;
    assert_eq!(sum.input_domain, domain);
    assert!(sum.input_domain.member(&values)?);
    assert_eq!(sum.invoke(&values)?, (n + 1) as i32); // no fixed integer cap
    macro_rules! check { ($($t:ty)+) => {$ (
        let domain = VectorDomain::new(AtomDomain::new_closed((0 as $t, 1 as $t))?);
        let sum = make_sum(domain.clone(), InsertDeleteDistance)?;
        let mut values = vec![1 as $t; n + 1];
        values[0] = 0 as $t;
        assert_eq!(sum.input_domain, domain);
        assert!(sum.input_domain.member(&values)?);
        assert_eq!(sum.invoke(&values)?, (n - 1) as $t); // drops the last record, not the first
        let fixed = make_sum(domain.with_size(n + 1), InsertDeleteDistance)?;
        assert_eq!(fixed.invoke(&vec![1 as $t; n + 1])?, (n + 1) as $t);
    )+}; }
    check! {f32 f64}
    Ok(())
}

#[test]
fn length_contract_and_prefix_eviction() -> Fallible<()> {
    let domain = VectorDomain::new(AtomDomain::new_closed((0i32, 1_073_741_824))?);
    let sum = make_sum(domain.clone(), InsertDeleteDistance)?;
    assert_eq!(sum.input_domain, domain);
    assert!(sum.input_domain.member(&vec![1_073_741_824; 2])?);
    assert_eq!(sum.invoke(&vec![1_073_741_824; 2])?, 1_073_741_824);
    assert_eq!(sum.invoke(&vec![1, 2, 3])?, 1);
    assert!(make_sum(domain.with_size(2), SymmetricDistance).is_err());

    let (lower, upper) = (-1_000_000_000i32, 1_100_000_000);
    let sum = make_sum(
        VectorDomain::new(AtomDomain::new_closed((lower, upper))?),
        InsertDeleteDistance,
    )?;
    assert_eq!(sum.invoke(&vec![upper])?, upper);
    assert_eq!(sum.invoke(&vec![lower, upper])?, lower);
    assert_eq!(sum.map(&1)?, upper - lower); // insertion evicts: range, not magnitude
    Ok(())
}

#[test]
fn symmetric_uniform_sampling() -> Fallible<()> {
    let sum = make_sum(
        VectorDomain::new(AtomDomain::new_closed((0i8, 63))?), // safe retained length 2
        SymmetricDistance,
    )?;
    let mut counts = [0usize; 3];
    for _ in 0..900 {
        let output = sum.invoke(&vec![1, 2, 4])?;
        counts[match output {
            3 => 0,
            5 => 1,
            6 => 2,
            _ => panic!("not a two-record subset sum"),
        }] += 1;
    }
    // Extremely wide acceptance interval: catches a deterministic prefix or
    // sampling with replacement, without relying on exact random frequencies.
    assert!(
        counts.into_iter().all(|n| (150..450).contains(&n)),
        "{counts:?}"
    );
    Ok(())
}

fn float_suite<T>() -> Fallible<()>
where
    T: Float + MakeSum<SymmetricDistance> + MakeSum<InsertDeleteDistance>,
{
    let cast = |x| T::exact_int_cast(x).unwrap();
    let domain = VectorDomain::new(AtomDomain::new_closed((cast(-10), cast(10)))?);
    let sum = make_sum(domain.clone(), InsertDeleteDistance)?;
    assert_eq!(sum.invoke(&vec![])?, T::zero());
    assert_eq!(
        sum.invoke(&vec![cast(1), cast(2), cast(-3), cast(4)])?,
        cast(4)
    );
    assert_eq!(
        sum.invoke(&vec![cast(1), cast(2), cast(-3), cast(4), cast(10)])?,
        cast(14)
    );
    let fixed = make_sum(domain.with_size(4), SymmetricDistance)?;
    assert!(sum.map(&1)? > fixed.map(&1)?);
    assert!(fixed.map(&0)? > T::zero());
    let q = T::from_bits(T::Bits::one());
    let tiny = make_sum(
        VectorDomain::new(AtomDomain::new_closed((-q, q))?).with_size(4),
        SymmetricDistance,
    )?;
    assert_eq!(tiny.invoke(&vec![q, q, -q, q])?, q + q);
    // An exact rational positive error smaller than q must round UP, not to zero.
    assert!(tiny.map(&0)? >= q);
    let max = T::MAX_FINITE;
    let singleton = make_sum(
        VectorDomain::new(AtomDomain::new_closed((-max, max))?).with_size(1),
        SymmetricDistance,
    )?;
    assert_eq!(singleton.invoke(&vec![max])?, max);
    assert_eq!(singleton.invoke(&vec![-max])?, -max);
    assert_eq!(singleton.map(&0)?, T::zero());
    assert!(
        make_sum(
            VectorDomain::new(AtomDomain::new_closed((-max, max))?).with_size(2),
            SymmetricDistance
        )
        .is_err()
    );
    let near = max / cast(8);
    let near_sum = make_sum(
        VectorDomain::new(AtomDomain::new_closed((-near, near))?).with_size(4),
        SymmetricDistance,
    )?;
    assert!(near_sum.invoke(&vec![near; 4])?.is_finite());
    assert!(
        make_sum(
            VectorDomain::new(AtomDomain::new_closed((T::zero(), T::infinity()))?),
            SymmetricDistance
        )
        .is_err()
    );
    let nan_domain = AtomDomain::new(
        Some(crate::domains::Bounds::new_closed((cast(-1), cast(1)))?),
        Some(crate::domains::NaN::new()),
    );
    assert!(make_sum(VectorDomain::new(nan_domain), SymmetricDistance).is_err());
    let empty = make_sum(
        VectorDomain::new(AtomDomain::new_closed((-max, max))?).with_size(0),
        SymmetricDistance,
    )?;
    assert_eq!(empty.invoke(&vec![])?, T::zero());
    assert_eq!(empty.map(&0)?, T::zero());
    // Rounding-sensitive equal multiset: permutations have symmetric distance 0.
    let a = T::one() / (T::epsilon() / cast(2));
    let domain = VectorDomain::new(AtomDomain::new_closed((-a, a))?).with_size(4);
    let sum = make_sum(domain, SymmetricDistance)?;
    let x = sum.invoke(&vec![a, T::one(), -a, T::zero()])?;
    let y = sum.invoke(&vec![a, -a, T::one(), T::zero()])?;
    assert_ne!(x, y);
    assert!((x.into_rational()? - y.into_rational()?).abs() <= sum.map(&0)?.into_rational()?);
    Ok(())
}

#[test]
fn float_formats() -> Fallible<()> {
    float_suite::<f32>()?;
    float_suite::<f64>()?;
    Ok(())
}

fn float_properties<T, S>() -> Fallible<()>
where
    T: Float + MakeSum<SymmetricDistance> + MakeSum<InsertDeleteDistance>,
    S: UncheckedSum<Item = T>,
{
    let a = T::one() / (T::epsilon() / T::exact_int_cast(2)?);
    let choices = [
        -a,
        -T::one(),
        T::zero(),
        T::one(),
        a,
        T::from_bits(T::Bits::one()),
    ];
    let mut rng = StdRng::seed_from_u64(7429);
    for n in 2..32 {
        let domain = VectorDomain::new(AtomDomain::new_closed((-a, a))?);
        let fixed = make_sum(domain.clone().with_size(n), SymmetricDistance)?;
        let variable = make_sum(domain, InsertDeleteDistance)?;
        for _ in 0..20 {
            let mut x: Vec<T> = (0..n)
                .map(|_| choices[rng.gen_range(0, choices.len())])
                .collect();
            let out = fixed.invoke(&x)?;
            let exact = x
                .iter()
                .try_fold(RBig::ZERO, |s, v| v.into_rational().map(|v| s + v))?;
            assert!(
                (S::unchecked_sum(&x).into_rational()? - exact).abs()
                    <= sum_error(n, S::depth(n), a)?
            );
            x.shuffle(&mut rng);
            let permuted = fixed.invoke(&x)?;
            assert!(
                (out.into_rational()? - permuted.into_rational()?).abs()
                    <= fixed.map(&0)?.into_rational()?
            );
            let before = variable.invoke(&x)?;
            x.pop();
            let after = variable.invoke(&x)?;
            assert!(
                (before.into_rational()? - after.into_rational()?).abs()
                    <= variable.map(&1)?.into_rational()?
            );
            x.push(choices[rng.gen_range(0, choices.len())]);
            let replacement = fixed.invoke(&x)?;
            assert!(
                (permuted.into_rational()? - replacement.into_rational()?).abs()
                    <= fixed.map(&2)?.into_rational()?
            );
        }
    }
    Ok(())
}

#[test]
fn exact_rational_property_checks() -> Fallible<()> {
    float_properties::<f32, Sequential<f32>>()?;
    float_properties::<f32, Pairwise<f32>>()?;
    float_properties::<f64, Sequential<f64>>()?;
    float_properties::<f64, Pairwise<f64>>()?;
    Ok(())
}

#[test]
fn integer_occurrence_priority_coupling() -> Fallible<()> {
    // Exhaust all occurrence-priority orders on this insertion edge. Repeated
    // values are distinct occurrences; both marginal subsets are uniform.
    fn permutations(xs: &mut [usize], i: usize, visit: &mut impl FnMut(&[usize])) {
        if i == xs.len() {
            visit(xs);
            return;
        }
        for j in i..xs.len() {
            xs.swap(i, j);
            permutations(xs, i + 1, visit);
            xs.swap(i, j);
        }
    }
    let values = [-2i8, -2, 50, 50];
    let sum = make_sum(
        VectorDomain::new(AtomDomain::new_closed((-2i8, 50))?), // safe retained length 2
        SymmetricDistance,
    )?;
    let one_edit_bound = IBig::from(sum.map(&1)?);
    let two_edit_bound = IBig::from(sum.map(&2)?);
    assert_eq!(one_edit_bound, IBig::from(52));
    assert_eq!(two_edit_bound, IBig::from(104));
    let selected_sum = |ids: &[usize]| ids.iter().map(|i| IBig::from(values[*i])).sum::<IBig>();
    let mut max_one_edit = IBig::ZERO;
    let mut max_two_edits = IBig::ZERO;
    let mut orders = [0, 1, 2, 3];
    let mut before_counts = std::collections::BTreeMap::new();
    let mut after_counts = std::collections::BTreeMap::new();
    permutations(&mut orders, 0, &mut |order| {
        // Use this same occurrence-priority order for all three inputs.
        // Sum the selected records directly, without invoking random selection.
        let select = |size| {
            order
                .iter()
                .copied()
                .filter(|i| *i < size)
                .take(2)
                .collect::<Vec<_>>()
        };
        let initial_ids = select(2);
        let mut before_ids = select(3);
        let mut after_ids = select(4);
        let initial = selected_sum(&initial_ids);
        let before = selected_sum(&before_ids);
        let after = selected_sum(&after_ids);
        let one_edit = (before - &after).abs();
        let two_edits = (initial - after).abs();
        assert!(one_edit <= one_edit_bound);
        assert!(two_edits <= two_edit_bound);
        max_one_edit = max_one_edit.clone().max(one_edit);
        max_two_edits = max_two_edits.clone().max(two_edits);
        before_ids.sort_unstable();
        after_ids.sort_unstable();
        *before_counts.entry(before_ids).or_insert(0) += 1;
        *after_counts.entry(after_ids).or_insert(0) += 1;
    });
    // Endpoint-valued records attain both bounds, so neither check is vacuous.
    assert_eq!(max_one_edit, one_edit_bound);
    assert_eq!(max_two_edits, two_edit_bound);
    assert_eq!(before_counts.len(), 3);
    assert!(before_counts.values().all(|count| *count == 8));
    assert_eq!(after_counts.len(), 6);
    assert!(after_counts.values().all(|count| *count == 4));
    Ok(())
}

#[test]
fn upward_rounding_and_depth_boundaries() -> Fallible<()> {
    use crate::traits::sum::round_up;
    macro_rules! check {
        ($t:ty) => {{
            let q = <$t>::from_bits(1);
            assert_eq!(round_up::<$t>(q.into_rational()? / RBig::from(2))?, q);
            let above_one = RBig::ONE + <$t>::EPSILON.into_rational()? / RBig::from(4);
            assert_eq!(round_up::<$t>(above_one)?, 1.0 + <$t>::EPSILON);
            let max = <$t>::MAX.into_rational()?;
            assert_eq!(round_up::<$t>(max.clone())?, <$t>::MAX);
            let above_max = max + q.into_rational()?;
            assert!(<$t>::inf_cast(above_max.clone())?.is_infinite());
            assert!(round_up::<$t>(above_max).is_err());
        }};
    }
    check!(f32);
    check!(f64);
    for (n, depth) in [
        (0, 0),
        (1, 0),
        (2, 1),
        (3, 2),
        (4, 2),
        (5, 3),
        (8, 3),
        (9, 4),
    ] {
        assert_eq!(Pairwise::<f64>::depth(n), depth);
        assert_eq!(Sequential::<f64>::depth(n), n.saturating_sub(1));
    }
    assert_eq!(Pairwise::<f64>::depth(usize::MAX), usize::BITS as usize);
    let large_n = (1 << 24) + 1;
    assert!(sum_error(large_n, large_n - 1, 1f32).is_err());
    // The same N is analyzable when the actual reduction has smaller depth.
    assert!(round_up::<f32>(sum_error(large_n, 25, 1f32)?)?.is_finite());
    assert!(sum_relaxation(large_n, 25, 1f32).is_ok());
    assert_eq!(sum_error(usize::MAX, usize::MAX - 1, 0f32)?, RBig::ZERO);
    assert!(sum_error(2, 1, -1f64).is_err());
    assert!(sum_error(2, 1, f64::INFINITY).is_err());
    // Exact error calculation is separate from finite conversion and safety.
    let large_error = sum_error(10_000, 9_999, f32::MAX)?;
    assert!(large_error > f32::MAX.into_rational()?);
    assert!(round_up::<f32>(large_error).is_err());
    assert!(sum_relaxation(10_000, 9_999, f32::MAX).is_err());
    Ok(())
}

#[test]
fn depth_parameterized_analysis() -> Fallible<()> {
    assert!(
        sum_error(16, Sequential::<f64>::depth(16), 1.)?
            > sum_error(16, Pairwise::<f64>::depth(16), 1.)?
    );
    let (n, h, magnitude) = (17usize, 5usize, 3.5f64);
    let hu = RBig::from(IBig::from(h)) * f64::EPSILON.into_rational()? / RBig::from(2);
    let error = &hu / (RBig::ONE - &hu) * RBig::from(IBig::from(n)) * magnitude.into_rational()?;
    assert_eq!(sum_error(n, h, magnitude)?, error.clone());
    assert_eq!(sum_relaxation(n, h, magnitude)?, error * RBig::from(2));
    assert!(sum_error(n, h, magnitude)? < sum_error(n, n - 1, magnitude)?);
    assert!(sum_relaxation(2, 1, f64::MAX).is_err());
    Ok(())
}

#[test]
fn contribution_selection_is_independent_of_arithmetic() -> Fallible<()> {
    use super::selection::select_contributions;
    let values = ["a", "b", "c"];
    assert_eq!(select_contributions(&values, 3, false)?, values);
    assert_eq!(select_contributions(&values, usize::MAX, false)?, values);
    assert_eq!(select_contributions(&values, 2, true)?, ["a", "b"]);
    assert!(select_contributions(&values, 0, false)?.is_empty());
    assert!(select_contributions(&values, 0, true)?.is_empty());
    assert!(select_contributions::<&str>(&[], 0, false)?.is_empty());
    assert!(select_contributions::<&str>(&[], 3, true)?.is_empty());
    let selected = select_contributions(&values, 2, false)?;
    assert_eq!(selected.len(), 2);
    assert_ne!(selected[0], selected[1]);
    assert!(selected.iter().all(|value| values.contains(value)));
    Ok(())
}

#[test]
fn default_pairwise_and_discrete_gaussian() -> Fallible<()> {
    let domain = VectorDomain::new(AtomDomain::new_closed((-1., 1.))?).with_size(16);
    let default = make_sum(domain, SymmetricDistance)?;
    let error = sum_relaxation(16, Pairwise::<f64>::depth(16), 1.)?;
    assert_eq!(
        default.map(&0)?,
        crate::traits::sum::round_up::<f64>(error)?
    );
    let a = 2f64.powi(53);
    let values = vec![a, 1., 1., -a];
    let default = make_sum(
        VectorDomain::new(AtomDomain::new_closed((-a, a))?).with_size(4),
        SymmetricDistance,
    )?;
    assert_eq!(
        default.invoke(&values)?,
        Pairwise::<f64>::unchecked_sum(&values)
    );
    assert_ne!(
        default.invoke(&values)?,
        Sequential::<f64>::unchecked_sum(&values)
    );

    let sum = make_sum::<_, i32>(
        VectorDomain::new(AtomDomain::new_closed((0, 10))?).with_size(3),
        SymmetricDistance,
    )?;
    let noise = crate::measurements::make_gaussian(
        sum.output_domain.clone(),
        sum.output_metric.clone(),
        2.,
        None,
    )?;
    let measurement = (sum >> noise)?;
    let _: i32 = measurement.invoke(&vec![1, 2, 4])?;
    assert!(measurement.check(&3, &12.5)?);
    Ok(())
}

#[cfg(feature = "ffi")]
#[test]
fn sum_ffi() -> Fallible<()> {
    use super::ffi::opendp_transformations__make_sum;
    use crate::{
        core,
        ffi::any::{AnyDomain, AnyMetric, AnyObject, Downcast},
    };
    for size in [Some(3), None] {
        let mut domain = VectorDomain::new(AtomDomain::new_closed((0f64, 10.))?);
        domain.size = size;
        let transformation = Fallible::from(opendp_transformations__make_sum(
            AnyDomain::new_raw(domain),
            AnyMetric::new_raw(InsertDeleteDistance),
        ))?;
        let result = core::opendp_core__transformation_invoke(
            &transformation,
            AnyObject::new_raw(vec![1., 2., 3.]),
        );
        assert_eq!(Fallible::from(result)?.downcast::<f64>()?, 6.);
    }
    macro_rules! supported { ($($t:ty)+) => {$ (
        for metric in [AnyMetric::new(InsertDeleteDistance), AnyMetric::new(SymmetricDistance)] {
            let domain = AnyDomain::new(VectorDomain::new(
                AtomDomain::new_closed((0 as $t, 10 as $t))?
            ));
            let sum = Fallible::from(opendp_transformations__make_sum(&domain, &metric))?;
            let result = core::opendp_core__transformation_invoke(
                &sum, AnyObject::new_raw(vec![1 as $t, 2 as $t, 3 as $t])
            );
            assert_eq!(Fallible::from(result)?.downcast::<$t>()?, 6 as $t);
        }
    )+}; }
    supported! {u32 u64 u128 i32 i64 i128 f32 f64}
    macro_rules! rejected { ($($t:ty)+) => {$ (
        let domain = AnyDomain::new(VectorDomain::new(
            AtomDomain::new_closed((0 as $t, 1 as $t))?
        ));
        let metric = AnyMetric::new(SymmetricDistance);
        let error = Fallible::from(opendp_transformations__make_sum(&domain, &metric))
            .err().expect("narrow integer should not dispatch");
        assert_eq!(error.variant, crate::error::ErrorVariant::Type);
    )+}; }
    rejected! {u8 u16 i8 i16 usize isize}
    Ok(())
}
