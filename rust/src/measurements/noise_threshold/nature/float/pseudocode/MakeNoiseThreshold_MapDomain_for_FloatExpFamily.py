# type: ignore
class FloatExpFamily:
    def make_noise_threshold(
        self,
        input_space: tuple[
            MapDomain[AtomDomain[TK], MapDomain[TV]], L0PInfDistance[P, AbsoluteDistance[QI]]
        ],
        threshold: TV,
    ) -> Measurement[
        MapDomain[AtomDomain[TK], AtomDomain[TV]],
        HashMap[TK, TV],
        L0PInfDistance[P, AbsoluteDistance[QI]],
        MO,
    ]:
        scale, k = self.scale, self.k
        distribution = ZExpFamily(
            scale=integerize_scale(scale, k)
        )  # |\label{line:dist}|

        if threshold.is_sign_negative():
            raise f"threshold ({threshold}) must not be negative"

        r_threshold = RBig.try_from(threshold)
        r_threshold = x_mul_2k(r_threshold, -k).round()

        t_int = make_float_to_bigint_threshold(input_space, threshold, k)
        m_noise = distribution.make_noise_threshold(t_int.output_space(), r_threshold)
        return t_int >> m_noise >> then_deintegerize_hashmap(k)
