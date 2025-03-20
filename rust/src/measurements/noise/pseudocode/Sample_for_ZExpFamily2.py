# type: ignore
class ZExpFamily2: # analogous to impl Sample for ZExpFamily<1> in Rust
    def sample(self, shift):
        return shift + sample_discrete_gaussian(self.scale) # |\label{line:add}|
