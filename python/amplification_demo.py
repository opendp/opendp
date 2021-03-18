
def make_poisson_sample(measurement, p):
    measurement = measurement.copy()

    def privacy_relation(d_in, d_out) -> bool:
        d_out_prime = (ln(exp(d_out) - 1.) / p + 1.) / d_in
        return measurement.privacy_relation(d_in, d_out_prime)

    def poisson_sample(data):
        output = []
        for entry in data:
            if sample_bernoulli(p):
                output.append(entry)
        return output

    measurement.privacy_relation = privacy_relation
    measurement.function = poisson_sample

    return measurement


speakers = set()
lines = 0
with open('utt2spk', 'r') as spk_file:
    for line in spk_file:
        speakers.add(line.split()[0])
        lines += 1

print(len(speakers))
print(lines)