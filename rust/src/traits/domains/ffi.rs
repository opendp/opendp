use crate::{
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, Downcast},
        util::Type,
    },
    traits::{CheckAtom, IsSizedDomain, ProductOrd},
};

impl IsSizedDomain for AnyDomain {
    fn get_size(&self) -> Fallible<usize> {
        fn monomorphize1<TIA>(domain: &AnyDomain, DIA: Type) -> Fallible<usize>
        where
            TIA: 'static + Clone + ProductOrd + CheckAtom,
        {
            fn monomorphize2<DIA: IsSizedDomain>(domain: &AnyDomain) -> Fallible<usize>
            where
                DIA: 'static,
                DIA::Carrier: 'static + Clone,
            {
                domain
                    .downcast_ref::<DIA>()
                    .map_err(|_| {
                        err!(
                            FFI,
                            "failed to downcast AnyDomain to {}",
                            Type::of::<DIA>().to_string()
                        )
                    })?
                    .get_size()
            }

            dispatch!(
                monomorphize2,
                [(DIA, [VectorDomain<AtomDomain<TIA>>])],
                (domain)
            )
        }

        let DI = Type::of_id(&self.domain.value.type_id())?;
        let TIA = DI.get_atom()?;

        dispatch!(monomorphize1, [(TIA, @numbers)], (self, DI))
    }
}
