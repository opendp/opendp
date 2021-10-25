# Prototype of interactive measurements with facilities for enforcing constraints across queryables.
#
# This is a sort of synthesis of Salil's method of hook functions, and Michael's approach of using an index
# to target specific child queryables. What I like about it is the natural interface: You operate on a child
# queryable directly, using the same interface as originally, and the right housekeeping happens behind the scenes.
# In the case of sequential composition, a sub-queryable is implicitly "retired" whenever a new sibling is spawned.
# Also, it doesn't require any special knowledge of the hierarchy inside child queryables. Everything is achieved
# by wrapping our existing entities, without "injecting" any logic.
#
# This works by putting state that must be shared across queryables in a coordinating state machine
# (also implemented as a queryable, though this isn't necesary). Then queries on the parent queryable,
# as well as any child queryables, are dispatched through this coordinator. The coordinator can then
# to enforce constraints like sequential access to children, or consumption of shared budget. After
# the constraints are checked, the operation is forwarded to the destination entity.
#
# NOTES:
# * For simplicity, this omits domains, and uses a static privacy loss instead of a relation.
# * I've split queryable state into two components: static context, and varying state. I think this helps clarity, but it's not necessary.
# * This retains the original taxonomy of InteractiveMeasurement being the general case, with Measurement a subtype.
# * The packing and unpacking of queryable elements is a bit wordy; I've written it this way to be very explicit about what I'm storing where.
# * This only supports a single level of sub-queryables. I think it could be generalized to support arbitrary levels of recursion, but it'll be tricky.
# * I've only implemented sequential composition, but concurrent composition should be straightforward. I think odometers should be doable too.
# * The code could use some rework; there would likely be a lot of duplication in other forms of composition that could be refactored out.
# (I think just supplying a coordinator queryable would be enough to have a generic make_composition() function.)


class Queryable:
    def __init__(self, parent, index, initial_state, transition):
        self.parent = parent        # Static context data (the part of "state" that doesn't change)
        self.index = index
        self.state = initial_state    # Variable state
        self.transition = transition  # fn: (state, question) -> (state, answer)
    def query(self, question):
        children = []
        child = self
        while child.parent is not None:
            children.append(child)
            child = child.parent
        parents = [child.parent for child in children]
        child_indexes = [child.index for child in children]
        for i, parent in enumerate(parents):
            child_index_path = child_indexes[i:]
            answer = parent._do_transition(child_index_path, question)
            if answer:
                return answer
        return self._do_transition(None, question)
    def _do_transition(self, child_index_path, question):
        (new_state, answer) = self.transition(self, child_index_path, self.state, question)
        self.state = new_state
        return answer


class InteractiveMeasurement:
    def __init__(self, function, privacy_loss):
        self.function = function          # fn: parent x index x data -> Queryable
        self.privacy_loss = privacy_loss  # Fixed privacy loss
    def eval(self, data) -> Queryable:    # Convenience method to invoke function
        return self.function(None, None, data)


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(parent, index, data):  # Wraps static function to generate a Queryable
            initial_state = function(data)
            def transition(_self, _child_index_path, state, _question):
                return (state, state)
            return Queryable(parent, index, initial_state, transition)
        super().__init__(interactive_function, privacy_loss)
    def eval1(self, data):               # Convenience method to invoke function, get result from null query
        queryable = self.eval(data)
        return queryable.query(None)


# Makes an adaptive composition InteractiveMeasurement. Spawned Queryables require their queries
# to be (non-Interactive) Measurements.
def make_adaptive_composition(budget):
    def function(parent, index, data):
        initial_state = (data, budget)
        def transition(_self, child_index_path, state, question: Measurement):
            data, budget = state
            if child_index_path is None:
                if question.privacy_loss > budget:
                    raise Exception("Insufficient budget")
                budget -= question.privacy_loss
                answer = question.eval1(data)
            else:
                answer = None
            new_state = (data, budget)
            return (new_state, answer)
        return Queryable(parent, index, initial_state, transition)
    return InteractiveMeasurement(function, budget)


# Makes a sequential composition InteractiveMeasurement. Spawned Queryables require their queries
# to be (non-Interactive) Measurements (whose Queryables must then be (non-Interactive) Measurements).
def make_sequential_composition(budget):

    def function(parent, index, data):
        initial_state = (data, budget, 0)

        def transition(self, child_index_path, state, question):
            data, budget, child_count = state
            if child_index_path is None:
                if question.privacy_loss > budget:
                    raise Exception("Insufficient budget")
                budget -= question.privacy_loss
                child_index = child_count
                answer = question.function(self, child_index, data)  # Question is an InteractiveMeasurement.
                child_count += 1
            else:
                if child_index_path[0] != child_count - 1:  # Make sure the child we're querying is the last created one (no backtracking)
                    raise Exception("Non-sequential query")
                answer = None
            new_state = (data, budget, child_count)
            return (new_state, answer)

        return Queryable(parent, index, initial_state, transition)

    return InteractiveMeasurement(function, budget)


# Constructor to make a simple (non-Interactive) Measurement
def make_base_laplace(sigma):
    def laplace(sigma):
        import random, math
        u = random.uniform(-0.5, 0.5)
        return math.copysign(1, u) * math.log(1.0 - 2.0 * abs(u)) * sigma
    return Measurement(lambda x: x + laplace(sigma), 1.0 / sigma)


# Converter from epsilon to  Laplace sigma
def eps_to_sigma(epsilon):
    return 1 / epsilon


def test_noninteractive():
    print("NON-INTERACTIVE MEASUREMENT")
    measurement = make_base_laplace(eps_to_sigma(1.0))
    print("    non-interactive =", measurement.eval1(123))


def test_adaptive():
    print("SIMPLE ADAPTIVE COMPOSITION")
    budget = 1.0
    print("    make adaptive")
    adaptive = make_adaptive_composition(budget)
    print("    make queryable")
    queryable = adaptive.eval(123)
    print("    adaptive query 1 =", queryable.query(make_base_laplace(eps_to_sigma(budget / 2))))
    print("    adaptive query 2 =", queryable.query(make_base_laplace(eps_to_sigma(budget / 2))))
    try:
        print("    SHOULD'VE FAILED: adaptive query 3 =", queryable.query(make_base_laplace(eps_to_sigma(budget / 2))))
    except Exception as e:
        print("    expected failure on adaptive query 3 =", e)


def test_sequential():
    print("SEQUENTIAL COMPOSITION")
    budget = 1.0
    print("    make sequential")
    sequential = make_sequential_composition(budget)
    print("    get root queryable")
    root_queryable = sequential.eval(123)

    print("    make adaptive (for sub-queryable 1 of sequential)")
    adaptive1 = make_adaptive_composition(budget / 2)
    print("    get sub-queryable 1")
    sub_queryable1 = root_queryable.query(adaptive1)
    print("    sub-queryable 1 / query 1 =", sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("    make adaptive (for sub-queryable 2 of sequential)")
    adaptive2 = make_adaptive_composition(budget / 2)
    print("    get sub-queryable 2")
    sub_queryable2 = root_queryable.query(adaptive2)
    print("    sub-queryable 2 / query 1 =", sub_queryable2.query(make_base_laplace(eps_to_sigma(budget / 4))))

    try:
        print("    SHOULD'VE FAILED: sub-queryable 1 / query 2 =", sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))
    except Exception as e:
        print("    expected failure on sub-queryable 1 / query 2 =", e)

def main():
    test_noninteractive()
    test_adaptive()
    test_sequential()


if __name__ == '__main__':
    main()
