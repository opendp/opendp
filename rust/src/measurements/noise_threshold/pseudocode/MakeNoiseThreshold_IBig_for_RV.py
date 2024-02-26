# type: ignore
# analogous to impl MakeNoise<VectorDomain<AtomDomain<IBig>>, MI, MO> for RV in Rust
class RV:
    def make_noise_threshold(
        self,
        input_space: tuple[MapDomain[AtomDomain[TK], AtomDomain[IBig]], MI],
        threshold: IBig,
    ) -> Measurement[
        MapDomain[AtomDomain[TK], AtomDomain[IBig]], HashMap[TK, IBig], MI, MO
    ]:
        input_domain, input_metric = input_space
        output_measure = MO.default()
        privacy_map = self.noise_threshold_privacy_map( # |\label{line:privacy-map}|
            input_metric, output_measure, threshold
        )

        def function(data: HashMap[TK, IBig]) -> HashMap[TK, IBig]:
            out = []
            for k, v in data.items():
                v = self.sample(v) # |\label{line:sample}|
                if v >= threshold:
                    out.append((k, v))
            # shuffle the output to avoid leaking the order of the input
            random.shuffle(out) # |\label{line:shuffle}|
            return dict(out)

        return Measurement.new(
            input_domain,
            Function.new_fallible(function),
            input_metric,
            output_measure,
            privacy_map,
        )
