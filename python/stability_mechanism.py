import os
from collections import Counter
import opendp


def main():
    lib_path = "../rust/target/debug/libopendp_ffi.dylib"
    odp = opendp.OpenDP(lib_path)

    word_counts = {}
    line_counts = {}

    data_dir = os.path.abspath(os.path.join(__file__, '..', '..', 'data'))
    for corpus_name in os.listdir(data_dir):
        corpus_path = os.path.join(data_dir, corpus_name)

        with open(corpus_path, 'r') as corpus_file:
            word_counts[corpus_name] = Counter(word for line in corpus_file for word in line.split())

        line_counts[corpus_name] = sum(1 for _ in open(corpus_path))

    for corpus_name, word_count, line_count in zip(word_counts, word_counts.values(), line_counts.values()):
        d_in = odp.data.distance_hamming(1)
        d_out = odp.data.distance_smoothed_max_divergence(0.1, .0000001)

        def check_stability(scale, threshold):
            stability_mech = odp.meas.make_stability_mechanism_l1(b"<u32, u32>", sum(word_count.values()), scale, threshold)
            check = odp.core.measurement_check(stability_mech, d_in, d_out)
            odp.core.measurement_free(stability_mech)
            return check

        threshold = 1000.
        scale = binary_search(lambda scale: check_stability(scale, threshold), 0., 100.)
        threshold = binary_search(lambda threshold: check_stability(scale, threshold), 0., 1000.)

        print("chosen scale and threshold:")
        print(scale, threshold)
        stability_mech = odp.meas.make_stability_mechanism_l1(b"<u32, u32>", line_count, scale, threshold)

        print("does chosen scale and threshold pass:")
        print(odp.core.measurement_check(stability_mech, d_in, d_out))

        laplace_mechanism = odp.meas.make_base_laplace(b"<f64>", scale)
        word_count = dict(word_count)

        vocabulary = set()
        for word in word_count:
            privatized_count = odp.core.measurement_invoke(laplace_mechanism, odp.data.f64(word_count[word]))
            if privatized_count >= threshold:
                vocabulary.add(word)

        print('results:')
        print(len(word_count))
        print(len(vocabulary))


    # print(word_counts)


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