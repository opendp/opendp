# type: ignore
def make_count(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: SymmetricDistance
):
    output_domain = AtomDomain(TO) # |\label{line:output-domain}|

    def function(data: Vec[TIA]) -> TO: # |\label{line:TO-output}|
        size = input_domain.size(data) # |\label{line:size}|
        try: # |\label{line:try-catch}|
            return TO.exact_int_cast(size) # |\label{line:exact-int-cast}|
        except FailedCast:
            return TO.MAX_CONSECUTIVE # |\label{line:except-return}|

    output_metric = AbsoluteDistance(TO)

    stability_map = new_stability_map_from_constant(TO.one()) # |\label{line:stability-map}|

    return Transformation(
        input_domain, output_domain, function,
        input_metric, output_metric, stability_map)