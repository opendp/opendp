# type: ignore
class FloatExpFamily:
    def make_noise_threshold(
        self,
        input_space: tuple[
            MapDomain[AtomDomain[TK], MapDomain[TV]], L0PI[P, AbsoluteDistance[QI]]
        ],
        threshold: TV,
    ) -> Measurement[
        MapDomain[AtomDomain[TK], AtomDomain[TV]],
        HashMap[TK, TV],
        L0PI[P, AbsoluteDistance[QI]],
        MO,
    ]:
        scale, k = self.scale, self.k
        distribution = ZExpFamily(
            scale=integerize_scale(scale, k)
        )  # |\label{line:dist}|

        if threshold.is_sign_negative():
            raise f"threshold ({threshold}) must not be negative"

        threshold = RBig.try_from(threshold)
        threshold = x_mul_2k(threshold, -k).round().into_parts()[1]

        t_int = make_float_to_bigint_threshold(input_space, k)
        m_noise = distribution.make_noise_threshold(t_int.output_space(), threshold)
        return t_int >> m_noise >> then_deintegerize_hashmap(k)
