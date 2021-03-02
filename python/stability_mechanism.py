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
        stability_mech = odp.meas.make_stability_mechanism_l1(b"<u32, u32>", line_count, 100.0, 1000.)
        d_in = odp.data.distance_hamming(1)
        d_out = odp.data.distance_smoothed_max_divergence(2., .00001)
        print(odp.core.measurement_check(stability_mech, d_in, d_out))

    # print(word_counts)


if __name__ == "__main__":
    main()