import os
from collections import Counter

import matplotlib.pyplot as plt
import numpy as np
from opendp.meas import make_base_ptr
from opendp.trans import make_count_by
from opendp.mod import enable_features, binary_search_param
from opendp.typing import L1Distance

enable_features("floating-point", "contrib")

# This is a more complicated example that makes
# differentially private releases of vocabulary sets from audio file transcriptions
# The plot shows the size of the released vocabulary set as you vary the budget parameters.
# The script assumes (to a fault) that each row is a different individual


def get_bounded_dataset(corpus_path, dataset_distance):
    """create a dataset of words with bounded user contribution
    Reads corpus_path and does a word count, with some DP-specific count truncation
    """
    lines = list()
    with open(corpus_path, 'r') as corpus_file:
        for line in corpus_file:
            # limit the counts for each word in each line to bound dataset distance
            individual_counter = Counter(line.split())
            for key in individual_counter:
                lines += [key] * min(dataset_distance, individual_counter[key])

    return lines


def privatize_vocabulary(word_count, dataset_distance, budget):
    """privatize a vocabulary
    :param word_count: a dictionary of {word: count}
    :param dataset_distance: max number of times an individual may repeat a word
    :param budget:
    :return: privatized vocabulary as a string set of words
    """
    count_by = make_count_by(TK=str, TV=float, MO=L1Distance[float])
    # solve for scale and threshold
    scale = binary_search_param(
        lambda s: count_by >> make_base_ptr(scale=s, threshold=1e8, TK=str),
        d_in=dataset_distance, d_out=budget)
    threshold = binary_search_param(
        lambda t: count_by >> make_base_ptr(scale=scale, threshold=t, TK=str),
        d_in=dataset_distance, d_out=budget)

    print("chosen scale and threshold:", scale, threshold)

    base_stability = count_by >> make_base_ptr(scale=scale, threshold=threshold, TK=str)

    privatized_count = base_stability(word_count)

    return set(privatized_count.keys())

    # incremental evaluation of the mechanism
    # laplace_mechanism = make_base_laplace(scale)
    # word_count = dict(word_count)
    #
    # vocabulary = set()
    # for word in word_count:
    #     privatized_count = laplace_mechanism(word_count[word])
    #     if privatized_count >= threshold:
    #         vocabulary.add(word)
    #
    # return vocabulary


def get_private_vocabulary_from_path(corpus_path, dataset_distance, budget):
    """create a private vocabulary
    This function combines get_bounded_vocabulary and privatize_vocabulary
    The constituent functions are separate to facilitate integration with popular tokenizers
    """
    words = get_bounded_dataset(corpus_path, dataset_distance)
    return privatize_vocabulary(words, dataset_distance, budget)


def write_private_vocabulary(corpus_path, output_path, dataset_distance, budget):
    """write a vocabulary file in the format needed for subword-nmt"""
    vocab = get_private_vocabulary_from_path(corpus_path, dataset_distance, budget)

    with open(output_path, 'w') as output_file:
        for key, count in sorted(vocab.items(), key=lambda x: x[1], reverse=True):
            output_file.write(key + " " + str(count) + "\n")


if __name__ == "__main__":

    # download the data if it does not exist
    data_dir = os.path.join(os.path.dirname(__file__), 'data')
    ami_dir = os.path.join(data_dir, 'AMI')
    if not os.path.exists(ami_dir):
        import requests
        ami_remote_dir = "https://raw.githubusercontent.com/opendp/dp-test-datasets/master/data/AMI/"
        os.makedirs(ami_dir)
        def download_ami(name):
            print(f'downloading {name} into {ami_dir}')
            with open(os.path.join(ami_dir, name), 'w') as ami_file:
                ami_file.write(requests.get(ami_remote_dir + "AMI_E_text").text)

        download_ami("AMI_E_text")
        download_ami("AMI_I_text")
        download_ami("AMI_T_text")

    # each user says each word at most this many times
    # If they say the word more than this many times, then truncate
    max_word_count_per_individual = 20

    for corpus_name in os.listdir(ami_dir):
        print(f'Processing {corpus_name}')
        corpus_path = os.path.join(ami_dir, corpus_name)

        # non-dp word dataset
        words = get_bounded_dataset(corpus_path, max_word_count_per_individual)
        # we make the (unlikely) assumption that each user influences at most one row
        # For this to be correct, we need a separate file that pairs each row with a user ID
        # Additional preprocessing is necessary if we consider the distinct_count of user IDs to be private

        epsilons = list(map(float, np.linspace(1., 10., 10)))
        deltas = list(map(float, np.linspace(1e-10, 1e-6, 10)))
        vocabulary_counts = np.zeros((len(epsilons), len(deltas)), dtype=int)

        # for each combination of epsilon and delta...
        for i, epsilon in enumerate(epsilons):
            for j, delta in enumerate(deltas):
                # ...save the size of the resulting vocabulary set
                vocabulary = privatize_vocabulary(
                    words,  # carrier (real data)
                    max_word_count_per_individual, (epsilon, delta))  # distance
                print(f"from {len(set(words))} words to {len(vocabulary)} words")
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

        # censor words that are not in the histogram release
        # ami_censored_dir = os.path.join(data_dir, 'AMI_censored')
        # os.makedirs(ami_censored_dir, exist_ok=True)
        # censored_corpus_path = os.path.join(ami_censored_dir, corpus_name)
        # with open(corpus_path, 'r') as in_file, open(censored_corpus_path, 'w') as out_file:
        #     for line in in_file:
        #         out_file.write(' '.join([word if word in vocabulary else 'CENSORED' for word in line.split()]) + '\n')
