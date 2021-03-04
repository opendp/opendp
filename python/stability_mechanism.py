import os
from collections import Counter

import opendp
lib_path = "../rust/target/debug/libopendp_ffi.dylib"
odp = opendp.OpenDP(lib_path)
max_word_count_per_individual = 20
import numpy as np
import matplotlib.pyplot as plt


def privatize_vocabulary(word_count, line_count, budget):
    # assuming each line is a different user, a user can influence up to max_word_count_per_individual counts
    d_in = odp.data.distance_hamming(max_word_count_per_individual)
    d_out = odp.data.distance_smoothed_max_divergence(*budget)

    def check_stability(scale, threshold):
        stability_mech = odp.meas.make_stability_mechanism_l1(b"<u32, u32>", line_count, scale, threshold)
        check = odp.core.measurement_check(stability_mech, d_in, d_out)
        odp.core.measurement_free(stability_mech)
        return check

    scale = binary_search(lambda scale: check_stability(scale, 1000.), 0., 100.)
    threshold = binary_search(lambda threshold: check_stability(scale, threshold), 0., 1000.)

    print("chosen scale and threshold:", scale, threshold)
    # stability_mech = odp.meas.make_stability_mechanism_l1(b"<u32, u32>", line_count, scale, threshold)
    # print("does chosen scale and threshold pass:", odp.core.measurement_check(stability_mech, d_in, d_out))

    laplace_mechanism = odp.meas.make_base_laplace(b"<f64>", scale)
    word_count = dict(word_count)

    vocabulary = set()
    for word in word_count:
        privatized_count = odp.data.to_f64(odp.core.measurement_invoke(laplace_mechanism, odp.data.from_f64(word_count[word])))
        if privatized_count >= threshold:
            vocabulary.add(word)

    return vocabulary


def main():

    word_counts = {}
    line_counts = {}

    data_dir = os.path.abspath(os.path.join(__file__, '..', '..', 'data'))
    censored_data_dir = os.path.abspath(os.path.join(__file__, '..', '..', 'data_censored'))
    for corpus_name in os.listdir(data_dir):
        corpus_path = os.path.join(data_dir, corpus_name)

        with open(corpus_path, 'r') as corpus_file:
            # truncate number of words in line to bound hamming distance
            total_counter = Counter()
            for line in corpus_file:
                individual_counter = Counter(line.split())
                for key in individual_counter:
                    individual_counter[key] = min(max_word_count_per_individual, individual_counter[key])
                total_counter += individual_counter

            word_counts[corpus_name] = total_counter

        line_counts[corpus_name] = sum(1 for _ in open(corpus_path))

    for corpus_name in word_counts:
        epsilons = np.linspace(.001, .01, 10)
        deltas = np.linspace(1e-10, 1e-8, 10)
        vocabulary_counts = np.zeros((len(epsilons), len(deltas)), dtype=int)

        for i, epsilon in enumerate(epsilons):
            for j, delta in enumerate(deltas):
                vocabulary = privatize_vocabulary(word_counts[corpus_name], line_counts[corpus_name], (epsilon, delta))
                print(f"from {len(word_counts[corpus_name])} words to {len(vocabulary)} words")
                vocabulary_counts[i, j] = len(vocabulary)

        fig, ax = plt.subplots()
        im = ax.imshow(vocabulary_counts, aspect='auto')

        # We want to show all ticks...
        ax.set_xticks(np.arange(len(epsilons)))
        ax.set_yticks(np.arange(len(deltas)))
        # ... and label them with the respective list entries
        ax.set_xticklabels([str(eps)[:5] for eps in epsilons])
        ax.set_yticklabels([f'{delta:.1e}' for delta in deltas])
        plt.xlabel('epsilon')
        plt.ylabel('delta')

        # Rotate the tick labels and set their alignment.
        plt.setp(ax.get_xticklabels(), rotation=45, ha="right", rotation_mode="anchor")

        mean_count = np.mean(vocabulary_counts)
        for i in range(len(epsilons)):
            for j in range(len(deltas)):
                text = ax.text(j, i, vocabulary_counts[i, j],
                               ha="center", va="center",
                               color="w" if vocabulary_counts[i, j] < mean_count * 1.2 else 'black')

        ax.set_title("Vocabulary Count Across Budget Constraints")
        fig.tight_layout()
        plt.show()


# corpus_path = os.path.join(data_dir, corpus_name)
        # os.makedirs(censored_data_dir, exist_ok=True)
        # censored_corpus_path = os.path.join(censored_data_dir, corpus_name)
        # with open(corpus_path, 'r') as in_file, open(censored_corpus_path, 'w') as out_file:
        #     for line in in_file:
        #         out_file.write(' '.join([word if word in vocabulary else 'CENSORED' for word in line.split()]) + '\n')


def binary_search(predicate, start, end):
    if start > end:
        raise ValueError

    if not predicate(end):
        raise ValueError("no possible value in range")

    while True:
        mid = (start + end) / 2
        passes = predicate(mid)

        if passes and end - start < .00001:
            return mid

        if passes:
            end = mid
        else:
            start = mid


if __name__ == "__main__":
    main()