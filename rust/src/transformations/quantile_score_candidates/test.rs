use super::*;

#[test]
fn test_count_lte() {
    let x = (5..20).collect::<Vec<i32>>();
    let edges = vec![2, 4, 7, 12, 22];
    let mut count_lt = vec![0; edges.len()];
    let mut count_eq = vec![0; edges.len()];
    count_lt_eq_recursive(
        count_lt.as_mut_slice(),
        count_eq.as_mut_slice(),
        edges.as_slice(),
        x.as_slice(),
        0,
    );
    println!("{:?}", count_lt);
    println!("{:?}", count_eq);
    assert_eq!(count_lt, vec![0, 0, 2, 7, 15]);
    assert_eq!(count_eq, vec![0, 0, 1, 1, 0]);
}

#[test]
fn test_count_lte_repetition() {
    let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
    let edges = vec![-1, 2, 4, 7, 12, 22];
    let mut count_lt = vec![0; edges.len()];
    let mut count_eq = vec![0; edges.len()];
    count_lt_eq_recursive(
        count_lt.as_mut_slice(),
        count_eq.as_mut_slice(),
        edges.as_slice(),
        x.as_slice(),
        0,
    );
    println!("{:?}", count_lt);
    println!("{:?}", count_eq);
    assert_eq!(count_lt, vec![0, 1, 4, 5, 8, 8]);
    assert_eq!(count_eq, vec![0, 2, 0, 3, 0, 0]);
}

fn test_case(x: Vec<i32>, edges: Vec<i32>, true_lt: Vec<usize>, true_eq: Vec<usize>) {
    let mut count_lt = vec![0; edges.len()];
    let mut count_eq = vec![0; edges.len()];
    count_lt_eq_recursive(
        count_lt.as_mut_slice(),
        count_eq.as_mut_slice(),
        edges.as_slice(),
        x.as_slice(),
        0,
    );
    println!("LT");
    println!("{:?}", true_lt);
    println!("{:?}", count_lt);

    println!("EQ");
    println!("{:?}", true_eq);
    println!("{:?}", count_eq);

    assert_eq!(true_lt, count_lt);
    assert_eq!(true_eq, count_eq);
}

#[test]
fn test_count_lte_edge_cases() {
    // check constant x
    test_case(vec![0; 10], vec![-1], vec![0], vec![0]);
    test_case(vec![0; 10], vec![0], vec![0], vec![10]);
    test_case(vec![0; 10], vec![1], vec![10], vec![0]);

    // below first split
    test_case(vec![1, 2, 3, 3, 3, 3], vec![2], vec![1], vec![1]);
    test_case(vec![1, 2, 3, 3, 3, 3, 3], vec![2], vec![1], vec![1]);
    // above first split
    test_case(vec![1, 1, 1, 1, 2, 3], vec![2], vec![4], vec![1]);
    test_case(vec![1, 1, 1, 1, 2, 3, 3], vec![2], vec![4], vec![1]);
}

#[test]
fn test_scorer() -> Fallible<()> {
    let edges = vec![-1, 2, 4, 7, 12, 22];

    let x = vec![0, 2, 2, 3, 5, 7, 7, 7];
    let scores = compute_score(x, &edges, 1, 2, 8);
    println!("{:?}", scores);

    let x = vec![0, 2, 2, 3, 4, 7, 7, 7];
    let scores = compute_score(x, &edges, 1, 2, 8);
    println!("{:?}", scores);
    Ok(())
}