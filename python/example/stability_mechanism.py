import os
from collections import Counter

import matplotlib.pyplot as plt
import numpy as np
import opendp
from opendp.v1.meas import make_base_laplace, make_base_stability
from opendp.v1.trans import make_count_by
from opendp.v1.typing import L2Distance, SymmetricDistance


def get_bounded_vocabulary(corpus_path, dataset_distance):
    """create a non-private vocabulary with bounded user contribution"""
    total_counter = Counter()
    with open(corpus_path, 'r') as corpus_file:
        for line in corpus_file:
            # truncate number of words in line to bound dataset distance
            individual_counter = Counter(line.split())
            for key in individual_counter:
                individual_counter[key] = min(dataset_distance, individual_counter[key])
            total_counter += individual_counter
    return total_counter


def privatize_vocabulary(word_count, line_count, dataset_distance, budget):
    """privatize a vocabulary with bounded user contribution"""

    scale = opendp.binary_search(
        lambda s: check_stability(s, 1000., line_count,  dataset_distance, budget),
        0., 100.)
    threshold = opendp.binary_search(
        lambda thresh: check_stability(scale, thresh, line_count, dataset_distance, budget),
        0., 1000.)

    print("chosen scale and threshold:", scale, threshold)
    # stability_mech = make_base_stability(line_count, scale, threshold, L2Distance[float], str, int)
    # print("does chosen scale and threshold pass:", stability_mech.check(d_in, d_out))

    laplace_mechanism = make_base_laplace(scale, float)
    word_count = dict(word_count)

    vocabulary = set()
    for word in word_count:
        privatized_count = laplace_mechanism(word_count[word])
        if privatized_count >= threshold:
            vocabulary.add(word)

    return vocabulary


def check_stability(scale, threshold, line_count, dataset_distance, budget):
    count_by = make_count_by(line_count, SymmetricDistance, L2Distance[float], str, int)
    base_stability = make_base_stability(line_count, scale, threshold, L2Distance[float], str, int)
    stability_mech = count_by >> base_stability

    # assuming each line is a different user, a user can influence up to max_word_count_per_individual counts
    return stability_mech.check(dataset_distance, budget)


def get_private_vocabulary(corpus_path, dataset_distance, budget):
    """create a private vocabulary"""
    word_counts = get_bounded_vocabulary(corpus_path, dataset_distance)
    line_count = sum(1 for _ in open(corpus_path))

    return privatize_vocabulary(word_counts, line_count, dataset_distance, budget)


def write_private_vocabulary(corpus_path, output_path, dataset_distance, budget):
    """write a vocabulary file in the format needed for subword-nmt"""
    vocab = get_private_vocabulary(corpus_path, dataset_distance, budget)

    with open(output_path, 'w') as output_file:
        for key, count in sorted(vocab.items(), key=lambda x: x[1], reverse=True):
            output_file.write(key + " " + str(count) + "\n")


if __name__ == "__main__":
    max_word_count_per_individual = 20

    data_dir = os.path.abspath(os.path.join(__file__, '..', '..', '..', 'data'))
    censored_data_dir = os.path.abspath(os.path.join(__file__, '..', '..', '..', 'data_censored'))

    for corpus_name in os.listdir(data_dir):
        corpus_path = os.path.join(data_dir, corpus_name)

        word_counts = get_bounded_vocabulary(corpus_path, max_word_count_per_individual)
        line_count = sum(1 for _ in open(corpus_path))

        epsilons = np.linspace(.001, .01, 10)
        deltas = np.linspace(1e-10, 1e-8, 10)
        vocabulary_counts = np.zeros((len(epsilons), len(deltas)), dtype=int)

        for i, epsilon in enumerate(epsilons):
            for j, delta in enumerate(deltas):
                vocabulary = privatize_vocabulary(
                    word_counts, line_count,  # carrier (real data)
                    max_word_count_per_individual, (epsilon, delta))  # distance
                print(f"from {len(word_counts)} words to {len(vocabulary)} words")
                vocabulary_counts[i, j] = len(vocabulary)

        fig, ax = plt.subplots()
        _im = ax.imshow(vocabulary_counts, aspect='auto')

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
                _text = ax.text(j, i, vocabulary_counts[i, j],
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
