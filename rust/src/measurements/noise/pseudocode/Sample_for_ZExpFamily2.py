# type: ignore
class ZExpFamily2: # analogous to impl Sample for ZExpFamily<1> in Rust
    def sample(self, shift):
        return [x_i + sample_discrete_gaussian(self.scale) for x_i in shift] # |\label{line:add}|
