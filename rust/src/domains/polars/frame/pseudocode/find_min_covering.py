# type: ignore
def find_min_covering(
    must_cover: set[T], sets: list[set[T], u32]
) -> list[tuple[set[T], u32]] | None:
    
    covered = list()  # `\label{covered}`

    while must_cover:  # `\label{loop}`

        def score(pair):
            by, weight = pair
            return len(by & must_cover), -len(by), -weight

        best_match = max(sets.items(), key=score)

        if best_match is None or best_match[0].isdisjoint(must_cover):
            return None
        best_set, weight = best_match

        must_cover -= best_set  # `\label{state}`
        covered[best_set] = weight

    return covered
