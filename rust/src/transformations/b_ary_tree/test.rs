use crate::{
    measurements::then_laplace,
    metrics::{L1Distance, L2Distance},
};

use super::*;

fn log_b_ceil_float(x: usize, b: usize) -> usize {
    // naive implementation susceptible to float approximation errors
    // for example, log_5(125) == 3, but evaluates to 3.0000000000000004 -> 4
    (x as f64).log(b as f64).ceil() as usize
}

#[test]
fn test_log_b_ceil() {
    // tests pass for small values,
    //    but large values fail because log_b_ceil_float is unstable
    (1..100).for_each(move |x| {
        (2..100).for_each(|b| {
            assert_eq!(
                log_b_ceil(x, b),
                log_b_ceil_float(x, b),
                "log_{:?}({:?})",
                b,
                x
            )
        })
    });
}

#[test]
fn test_num_layers_from_num_leaves() {
    assert_eq!(num_layers_from_num_leaves(10, 2), 5);
    assert_eq!(num_layers_from_num_leaves(16, 2), 5);
    assert_eq!(num_layers_from_num_leaves(8, 2), 4);
    assert_eq!(num_layers_from_num_leaves(6, 2), 4);
    assert_eq!(num_layers_from_num_leaves(3, 2), 3);
}

#[test]
fn test_layers_from_num_nodes() {
    assert_eq!(num_layers_from_num_nodes(1, 2), 1);
    assert_eq!(num_layers_from_num_nodes(2, 2), 2);
    assert_eq!(num_layers_from_num_nodes(3, 2), 2);
    assert_eq!(num_layers_from_num_nodes(7, 2), 3);
    assert_eq!(num_layers_from_num_nodes(8, 2), 4);

    assert_eq!(num_layers_from_num_nodes(2, 3), 2);
    assert_eq!(num_layers_from_num_nodes(4, 3), 2);
    assert_eq!(num_layers_from_num_nodes(5, 4), 2);
    assert_eq!(num_layers_from_num_nodes(13, 3), 3);
    assert_eq!(num_layers_from_num_nodes(14, 3), 4);
}

#[test]
fn test_num_nodes_from_num_layers() {
    assert_eq!(num_nodes_from_num_layers(1, 2), 1);
    assert_eq!(num_nodes_from_num_layers(2, 2), 3);
    assert_eq!(num_nodes_from_num_layers(3, 2), 7);
    assert_eq!(num_nodes_from_num_layers(4, 2), 15);

    assert_eq!(num_nodes_from_num_layers(1, 3), 1);
    assert_eq!(num_nodes_from_num_layers(2, 3), 4);
    assert_eq!(num_nodes_from_num_layers(3, 3), 13);
    assert_eq!(num_nodes_from_num_layers(4, 3), 40);

    assert!(checked_num_nodes_from_num_layers(3, usize::MAX).is_err());

    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        checked_num_nodes_from_num_layers(3, 3_000_000).unwrap(),
        9_000_003_000_001
    );
}

#[test]
fn test_b_ary_tree_arithmetic_and_domain() -> Fallible<()> {
    use crate::core::Domain;

    let tree = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::new_closed((0, 1))?).with_size(2),
        L1Distance::<i32>::default(),
        L1Distance::<i32>::default(),
        2,
        2,
    )?;
    assert!(tree.output_domain.member(&vec![2, 1, 1])?);

    let tree = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::default()).with_size(2),
        L1Distance::<i32>::default(),
        L1Distance::<i32>::default(),
        2,
        2,
    )?;
    assert_eq!(
        tree.invoke(&vec![i32::MAX, 1])?,
        vec![i32::MAX, i32::MAX, 1]
    );
    assert_eq!(
        tree.invoke(&vec![i32::MIN, -1])?,
        vec![i32::MIN, i32::MIN, -1]
    );
    Ok(())
}

#[test]
fn test_consistent_b_ary_tree_checked_arithmetic() -> Fallible<()> {
    assert!(make_consistent_b_ary_tree::<i32, f64>(0).is_err());
    assert!(make_consistent_b_ary_tree::<i32, f64>(1).is_err());

    let post = make_consistent_b_ary_tree::<i32, f64>(2)?;
    assert!(post.eval(&vec![]).is_err());
    assert_eq!(post.eval(&vec![7])?, vec![7.0]);

    // A complete tree can fit in usize even when b^layers does not.
    assert_eq!(
        num_nodes_from_num_layers(usize::BITS as usize, 2),
        usize::MAX
    );
    assert_eq!(
        num_layers_from_num_nodes(usize::MAX, 2),
        usize::BITS as usize
    );
    assert!(checked_num_nodes_from_num_layers(usize::BITS as usize + 1, 2).is_err());
    Ok(())
}

#[test]
fn test_choose_branching_factor_small() {
    assert_eq!(choose_branching_factor(0), 2);
    assert_eq!(choose_branching_factor(1), 2);
    assert_eq!(choose_branching_factor(2), 2);
    assert!((2..=256).contains(&choose_branching_factor(u32::MAX)));
}

#[test]
fn test_make_b_ary_tree() -> Fallible<()> {
    let trans = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::default()),
        L1Distance::<i32>::default(),
        L1Distance::<i32>::default(),
        10,
        2,
    )?;
    let actual = trans.invoke(&vec![1; 10])?;
    let expect = vec![
        vec![10],
        vec![8, 2],
        vec![4, 4, 2, 0],
        vec![2, 2, 2, 2, 2, 0, 0, 0],
        vec![1; 10],
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<i32>>();
    assert_eq!(expect, actual);

    Ok(())
}

#[test]
fn test_b_ary_tree_stability() -> Fallible<()> {
    let domain = VectorDomain::new(AtomDomain::<i32>::default()).with_size(4);
    let l1 = make_b_ary_tree(
        domain.clone(),
        L1Distance::<i32>::default(),
        L1Distance::<i32>::default(),
        4,
        4,
    )?;
    let l2 = make_b_ary_tree(
        domain,
        L1Distance::<i32>::default(),
        L2Distance::<f64>::default(),
        4,
        4,
    )?;

    assert_eq!(l1.map(&2)?, 4);
    assert!(l2.map(&2)? >= 2.0 * 2f64.sqrt());
    assert!(l2.map(&2)? < 3.0);
    assert_eq!(l2.invoke(&vec![1; 4])?, vec![4, 1, 1, 1, 1]);

    Ok(())
}

#[test]
fn test_b_ary_tree_l1_to_l2_stability_incomplete() -> Fallible<()> {
    // Five leaves are padded to eight, then three unused leaves are omitted.
    let tree = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::default()).with_size(5),
        L1Distance::<i32>::default(),
        L2Distance::<f64>::default(),
        5,
        2,
    )?;
    // The final leaf contributes once per layer, so its L2 norm is sqrt(4).
    let output = tree.invoke(&vec![0, 0, 0, 0, 1])?;
    assert_eq!(output, vec![1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1]);
    assert!(tree.map(&1)? >= 2.0);
    assert!(tree.map(&1)? < 2.1);
    Ok(())
}

#[test]
fn test_noise_b_ary_tree() -> Fallible<()> {
    let trans = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::default()),
        L1Distance::<u32>::default(),
        L1Distance::<u32>::default(),
        10,
        2,
    )?;
    let meas = (trans >> then_laplace(1., None))?;
    println!("noised {:?}", meas.invoke(&vec![1; 10])?);

    Ok(())
}

#[test]
fn test_identity() -> Fallible<()> {
    let b = 2;
    let trans = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::default()),
        L1Distance::<u32>::default(),
        L1Distance::<u32>::default(),
        10,
        b,
    )?;
    let meas = (trans.clone() >> then_laplace(0., None))?;
    let post = make_consistent_b_ary_tree::<i32, f64>(b)?;

    let noisy_tree = meas.invoke(&vec![1; 10])?;
    // casting should not lose data, as noise was integral
    let consi_leaves = post
        .eval(&noisy_tree)?
        .into_iter()
        .map(|v| v as i32)
        .collect();
    let consi_tree = make_b_ary_tree(
        VectorDomain::new(AtomDomain::<i32>::default()),
        L1Distance::<f64>::default(),
        L1Distance::<f64>::default(),
        10,
        b,
    )?
    .invoke(&consi_leaves)?;

    println!("noisy      leaves {:?}", noisy_tree[15..].to_vec());
    println!("consistent leaves {:?}", consi_leaves);

    println!("noisy      tree {:?}", noisy_tree);
    println!("consistent tree {:?}", consi_tree);

    assert_eq!(noisy_tree, consi_tree);
    Ok(())
}
