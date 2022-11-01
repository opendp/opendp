use crate::{
    core::{Domain, Function, Postprocessor},
    error::Fallible,
};

// A crate-private function to build a postprocessor
#[allow(dead_code)]
pub(crate) fn make_postprocess<DI: Domain, DO: Domain>(
    input_domain: DI,
    output_domain: DO,
    function: Function<DI, DO>,
) -> Fallible<Postprocessor<DI, DO>> {
    Ok(Postprocessor::new(input_domain, output_domain, function))
}

#[cfg(test)]
mod test {
    use crate::domains::{AllDomain, VectorDomain};

    use super::*;
    #[test]
    fn test_postprocess() -> Fallible<()> {
        let post = make_postprocess(
            AllDomain::new(),
            VectorDomain::new_all(),
            Function::new(|_| vec![12]),
        )?;

        assert_eq!(post.invoke(&"A".to_string())?, vec![12]);

        Ok(())
    }
}
