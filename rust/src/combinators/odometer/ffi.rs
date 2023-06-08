use crate::{
    core::PrivacyMap,
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasure, AnyMetric, AnyObject, AnyOdometer, AnyQueryable, Downcast},
    interactive::PolyQueryable,
};

use super::{Invokable, OdometerAnswer, OdometerQuery};

pub type AnyOdometerQuery = OdometerQuery<AnyObject, AnyObject>;
pub type AnyOdometerAnswer = OdometerAnswer<AnyObject, AnyObject>;

impl Invokable<AnyDomain, AnyMetric, AnyMeasure> for AnyOdometer {
    type Output = AnyObject;
    fn invoke_wrap_and_map(
        &self,
        value: &AnyObject,
        wrapper: impl Fn(PolyQueryable) -> Fallible<PolyQueryable> + 'static,
    ) -> Fallible<(Self::Output, PrivacyMap<AnyMetric, AnyMeasure>)> {
        // wrap the child odometer to send ChildChange queries
        let answer = self.invoke_wrap(value, wrapper)?;

        // if the output of the odometer is not a queryable, then reject the query completely
        let inner_qbl = answer.downcast_ref::<AnyQueryable>()?.clone();

        let map = PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            // pack the d_in in an OdometerQuery enum before sending into the child queryable
            let query = AnyObject::new(AnyOdometerQuery::Map(d_in.clone()));
            inner_qbl.clone().eval(&query)
        });
        Ok((answer, map))
    }

    fn one_time_privacy_map(&self) -> Option<PrivacyMap<AnyMetric, AnyMeasure>> {
        None
    }

    fn input_domain(&self) -> AnyDomain {
        self.input_domain.clone()
    }

    fn input_metric(&self) -> AnyMetric {
        self.input_metric.clone()
    }

    fn output_measure(&self) -> AnyMeasure {
        self.output_measure.clone()
    }
}
