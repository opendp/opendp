use dashu::rbig;

use crate::traits::samplers::Shuffle;

use super::*;

fn argsort<T: Ord>(x: &[T]) -> Vec<usize> {
    let mut indices = (0..x.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &x[i]);
    indices
}

#[test]
fn test_peel_permute_and_flip() {
    for len in [0, 1, 2, 3, 4, 5] {
        for _trial in 0..len.min(1) {
            for scale in [rbig![0], rbig![1]] {
                let mut x = vec![rbig![0], rbig![50], rbig![100], rbig![150]];
                x.truncate(len.min(x.len()));
                x.shuffle().unwrap();

                let mut expected = argsort(&x);
                expected.reverse();

                let observed = peel_permute_and_flip(x, scale, len).unwrap();
                assert_eq!(expected, observed);
            }
        }
    }
}

#[test]
fn test_permute_and_flip() {
    for scale in [rbig![0], rbig![1]] {
        for _ in 0..100 {
            let x = [rbig![100], rbig![0], rbig![0]];
            let selection = permute_and_flip(&x, &scale).unwrap();
            assert_eq!(selection, 0);
        }
        assert_eq!(permute_and_flip(&[rbig![0]], &scale).unwrap(), 0);
        assert!(permute_and_flip(&[], &scale).is_err());
    }
}
