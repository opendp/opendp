use crate::{
    core::{Domain, Function, StabilityMap, Transformation},
    error::Fallible,
    metrics::AgnosticMetric,
};

// A crate-private function to build a postprocessor
#[allow(dead_code)]
pub(crate) fn make_postprocess<DI, DO>(
    input_domain: DI,
    output_domain: DO,
    function: Function<DI, DO>,
) -> Fallible<Transformation<DI, DO, AgnosticMetric, AgnosticMetric>>
where
    DI: Domain,
    DO: Domain,
{
    Ok(Transformation::new(
        input_domain,
        output_domain,
        function,
        AgnosticMetric::default(),
        AgnosticMetric::default(),
        StabilityMap::new(|_| ()),
    ))
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
            Function::new(|_| vec![12]))?;

        assert_eq!(post.invoke(&"A".to_string())?, vec![12]);

        Ok(())
    }
}