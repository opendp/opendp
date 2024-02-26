# type: ignore
class FloatExpFamily:
    def make_noise(self, input_space) -> Measurement[AtomDomain[T], T, AbsoluteDistance[QI], MO]:
        scale, k = self.scale, self.k
        distribution = ZExpFamily(scale=integerize_scale(scale, k)) # |\label{line:dist}|

        t_int = make_float_to_bigint(input_space)
        m_noise = distribution.make_noise(t_int.output_space())
        return t_int >> m_noise >> then_deintegerize_vec(k)
    