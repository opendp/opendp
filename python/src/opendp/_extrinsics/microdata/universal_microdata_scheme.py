from typing import Callable, Union, Optional

from opendp.mod import Domain, Metric, Measurement
import opendp.prelude as dp
from opendp._lib import get_np_csprng, import_optional_dependency
from opendp._extrinsics._make_private_selection import make_private_selection_threshold
from opendp._extrinsics.microdata.faithfulness import make_private_faithfulness
from opendp._extrinsics.synth import make_private_synthesizer_trainer


__all__ = ["make_private_universal_microdata_scheme"]


def make_private_universal_microdata_scheme(input_domain: Domain,
                                            input_metric: Metric,
                                            synthesizer_epsilon: float,
                                            configuration_candidates: Optional[list[dict]] = None,
                                            quality_acceptance_criteria: Optional[list[tuple]] = None,
                                            quality_epsilons: Optional[list[float]] = None,
                                            faithfulness_simiarlity: Optional[Union[Callable, dict[str, float]]] = None,
                                            faithfulness_threshold: Optional[float] = None,
                                            faithfulness_epsilon: Optional[float] = None,
                                            with_selection: bool = True,
                                            selection_stop_probability: float = 0,
                                            selection_epsilon: float = 0,
                                            selection_steps: Optional[int] = None) -> Measurement:

    np = import_optional_dependency("numpy")

    if "LazyFrameDomain" not in str(input_domain.type):
        raise ValueError("input_domain must be a LazyFrame domain")

    # if (input_domain.descriptor.get("size")) is None:
    #     raise ValueError("input_domain's size must be known")

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")

    if sum([quality_acceptance_criteria is None, quality_epsilons is None]) == 1:
        raise ValueError("quality_acceptance_criteria and quality_epsilons must be both None or both not None")

    if 0 < sum([faithfulness_simiarlity is None, faithfulness_threshold is None, faithfulness_epsilon is None]) < 3:
        raise ValueError("faithfulness_simiarlity, faithfulness_threshold, faithfulness_epsilons must be all None or all not None")

    if quality_epsilons is None and faithfulness_epsilon is None:
        raise ValueError("At least one of quality_epsilons and faithfulness_epsilon must be not None")

    output_measure = dp.max_divergence(T=float)

    if configuration_candidates is None:
        configuration_candidates = [{}]

    acceptance_criteria_epsilons: list[float] = []

    if quality_epsilons is not None:
        acceptance_criteria_epsilons.extend(quality_epsilons)

    if faithfulness_epsilon is not None:
        acceptance_criteria_epsilons.append(faithfulness_epsilon)

    epsilons = [synthesizer_epsilon, sum(acceptance_criteria_epsilons)]

    # TODO: is it correct?
    d_in = 1

    overall_pipeline = dp.c.make_sequential_composition(input_domain,
                                                        input_metric,
                                                        output_measure,
                                                        d_in,
                                                        d_mids=epsilons)

    np_csprng = get_np_csprng()

    def function(dataset):

        num_records = len(dataset.collect())
        overall_pipline_comp = overall_pipeline(dataset)
        del dataset

        # Synthesizer

        configuration = np_csprng.choice(configuration_candidates)

        synthesizer_trainer = make_private_synthesizer_trainer(input_domain,
                                                               input_metric,
                                                               synthesizer_epsilon,
                                                               **configuration["synthesizer"])

        synthesizer = overall_pipline_comp(synthesizer_trainer)  

        synth_dataset = synthesizer.sample(num_records)

        # Acceptance Criteria
        acceptance_criteria_pipeline = dp.c.make_sequential_composition(input_domain, input_metric, output_measure, d_in,
                                                                        d_mids=acceptance_criteria_epsilons)

        acceptance_criteria_pipeline_comp = overall_pipline_comp(acceptance_criteria_pipeline)

        acceptance_criteria_meas: list[Measurement] = []
        acceptance_criteria_thresholds: list[float] = []

        # Quality Acceptance Criteria
        if quality_epsilons is not None and quality_acceptance_criteria is not None:
            (quality_acceptance_criteria_makers,
             quality_accaptance_criteria_thresholds) = zip(*quality_acceptance_criteria)
            assert (len(quality_acceptance_criteria_makers)
                    == len(quality_accaptance_criteria_thresholds))

            acceptance_criteria_meas = []
            for make_private_ac, epsilon in zip(quality_acceptance_criteria_makers,
                                                quality_epsilons):
                acceptance_criteria_meas.append(dp.binary_search_chain(
                        (lambda s: make_private_ac(input_domain, input_metric, output_measure,
                                                   scale=s, reference_dataset=synth_dataset)),
                        d_in=d_in, d_out=epsilon)
                  )

            acceptance_criteria_thresholds.extend(quality_accaptance_criteria_thresholds)

        # Faithfulness Acceptance Criteria
        if faithfulness_epsilon is not None and faithfulness_threshold is not None:
            def faithfulness_meas(scale):
                return make_private_faithfulness(input_domain,
                                                 input_metric,
                                                 output_measure,
                                                 reference_dataset=synth_dataset,
                                                 similarity=faithfulness_simiarlity,
                                                 scale=scale)

            def to_unfaithfulness(matching_cardinality):
                return num_records - matching_cardinality

            unfaithfulness_meas = dp.binary_search_chain(
                lambda s: faithfulness_meas(s) >> to_unfaithfulness,
                d_in=d_in, d_out=faithfulness_epsilon)

            unfaithfulness_cardinality_threshold = int(np.ceil(faithfulness_threshold * num_records))

            acceptance_criteria_meas.append(unfaithfulness_meas)
            acceptance_criteria_thresholds.append(unfaithfulness_cardinality_threshold)

        # Gather Results and Check if all Acceptance Criteria Passed
        acceptance_criteria_results = [acceptance_criteria_pipeline_comp(meas)
                                       for meas in acceptance_criteria_meas]

        assert len(acceptance_criteria_results) == len(acceptance_criteria_thresholds)
        all_acceptance_criteria_passed = all(value <= threshold
                                             for value, threshold in zip(acceptance_criteria_results, acceptance_criteria_thresholds))

        return all_acceptance_criteria_passed, configuration, synthesizer, synth_dataset, acceptance_criteria_results

    pipeline_meas = dp.m.make_user_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(T=float),
        function,
        privacy_map=overall_pipeline.map)

    if not with_selection:
        return pipeline_meas
    else:
        return make_private_selection_threshold(pipeline_meas,
                                                threshold=1,
                                                stop_probability=selection_stop_probability,
                                                epsilon_selection=selection_epsilon,
                                                steps=selection_steps)
