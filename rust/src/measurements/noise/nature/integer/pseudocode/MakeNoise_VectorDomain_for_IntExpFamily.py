# type: ignore
class IntExpFamily:
    def make_noise(self, input_space) -> Measurement[AtomDomain[T], T, AbsoluteDistance[QI], MO]:
        distribution = ZExpFamily(scale=integerize_scale(self.scale, 0)) # |\label{line:dist}|

        t_int = make_int_to_bigint(input_space)
        m_noise = distribution.make_noise(t_int.output_space())
        return t_int >> m_noise >> then_saturating_cast()
    