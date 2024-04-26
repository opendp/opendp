use super::*;

#[test]
fn test_vec() {
    let form = vec![1, 2, 3];
    test_round_trip(form);
}

// #[test]
// #[should_panic]
// fn test_bogus() {
//     let form = (Data::new(vec![1, 2, 3]), Data::new(99.9));
//     let data = Data::new(form);
//     let _retrieved: Vec<String> = data.into_form();
// }

fn test_round_trip<T: 'static + IsVec + PartialEq>(form: T) {
    let data = Column(form.box_clone());
    assert_eq!(&form, data.as_form().unwrap_test());
    assert_eq!(form, data.into_form().unwrap_test())
}
