use crate::{measurements::then_laplace, metrics::L1Distance};

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
}

#[test]
fn test_make_b_ary_tree() -> Fallible<()> {
    let trans =
        make_b_ary_tree::<L1Distance<i32>, i32>(Default::default(), L1Distance::default(), 10, 2)?;
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
fn test_noise_b_ary_tree() -> Fallible<()> {
    let meas = (make_b_ary_tree::<_, i32>(Default::default(), L1Distance::default(), 10, 2)?
        >> then_laplace(1., None))?;
    println!("noised {:?}", meas.invoke(&vec![1; 10])?);

    Ok(())
}

#[test]
fn test_identity() -> Fallible<()> {
    let b = 2;
    let trans = make_b_ary_tree::<_, i32>(Default::default(), L1Distance::default(), 10, b)?;
    let meas = (trans.clone() >> then_laplace(0., None))?;
    let post = make_consistent_b_ary_tree::<i32, f64>(b)?;

    let noisy_tree = meas.invoke(&vec![1; 10])?;
    // casting should not lose data, as noise was integral
    let consi_leaves = post
        .eval(&noisy_tree)?
        .into_iter()
        .map(|v| v as i32)
        .collect();
    let consi_tree =
        make_b_ary_tree::<_, i32>(Default::default(), L1Distance::<f64>::default(), 10, b)?
            .invoke(&consi_leaves)?;

    println!("noisy      leaves {:?}", noisy_tree[15..].to_vec());
    println!("consistent leaves {:?}", consi_leaves);

    println!("noisy      tree {:?}", noisy_tree);
    println!("consistent tree {:?}", consi_tree);

    assert_eq!(noisy_tree, consi_tree);
    Ok(())
}
