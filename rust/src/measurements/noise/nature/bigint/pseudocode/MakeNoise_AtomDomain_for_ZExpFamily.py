# type: ignore
class ZExpFamily:
    def make_noise(
        self, input_space: tuple[AtomDomain[IBig], AbsoluteDistance[RBig]]
    ) -> Measurement[AtomDomain[IBig], IBig, AbsoluteDistance[RBig], MO]:
        t_vec = make_vec(input_space)  # |\label{line:make-vec}|
        m_noise = self.make_noise(t_vec.output_space())  # |\label{line:make-noise}|

        return t_vec >> m_noise >> then_index_or_default(0)  # |\label{line:post}|
