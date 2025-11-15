# type: ignore
class IntExpFamily:
    def make_noise_threshold(
        self,
        input_space: tuple[
            MapDomain[AtomDomain[TK], AtomDomain[TV]], L0PInfDistance[P, AbsoluteDistance[QI]]
        ],
        threshold: TV,
    ) -> Measurement[
        MapDomain[AtomDomain[TK], AtomDomain[TV]],
        HashMap[TK, TV],
        L0PInfDistance[P, AbsoluteDistance[QI]],
        MO,
    ]:
        distribution = ZExpFamily(
            scale=integerize_scale(self.scale, 0)
        )  # |\label{line:dist}|

        threshold = UBig.try_from(threshold)

        t_int = make_int_to_bigint_threshold(input_space)
        m_noise = distribution.make_noise_threshold(t_int.output_space())
        f_native_int = then_saturating_cast_hashmap()

        return t_int >> m_noise >> f_native_int
