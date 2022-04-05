# Prototype of interactive measurements with facilities for enforcing constraints across hierarchies of queryables.
#
# This is a sort of synthesis of Salil's method of hook functions, and Michael's approach of using an index
# to target specific child queryables. What I like about it is the natural interface: You operate on a child
# queryable directly, using the same interface as always; any housekeeping happens automatically behind the scenes.
# In the case of sequential composition, a sub-queryable is implicitly "retired" whenever a new sibling is spawned.
#
# This works by building an explicit hierarchy of queryables, and forwarding queries through the ancestor chain
# (starting at the root). This allows ancestors to update their state and maintain any constraints.

# More here: https://docs.google.com/document/d/1hHTrXFgTlxHL4KidO_9MxHbjMSnPO6vccuTqVezRrG0/edit?usp=sharing
#
# NOTES:
# * For simplicity, this omits domains, and uses a static privacy loss instead of a relation.
# * This retains the original taxonomy of InteractiveMeasurement being the general type, with Measurement a subtype.
# * eval() is the public method to invoke InteractiveMeasurements. eval1() is for (non-Interactive)Measurements.
# * Unlike this previous prototype, this one supports arbitrary recursion of Queryables.


class FlatQueryable:

    def __init__(self, initial_state, transition):
        self.state = initial_state
        self.transition = transition  # fn: (question, state) -> (state, answer)

    def query(self, question):
        (answer, new_state) = self.transition(self, question, self.state)
        self.state = new_state
        return answer



class NestedQueryable:

    def __init__(self, parent, index, initial_state, transition):
        self.index = index
        self.parent = parent
        self.children = []
        self.state = initial_state
        self.transition = transition  # fn: (Q x S) -> (A x S)
        self.pre_transition = None    # fn: (Q x S) -> (M x S')
        self.response_map = {}        # fn: I -> M x S -> M' x S
        self.post_transition = None   # fn: (M' x S') -> (A x S)

    def query(self, question):
        (answer, new_state) = self.transition(self, question, self.state)
        self.state = new_state
        return answer



class InteractiveMeasurement:
    def __init__(self, function, privacy_loss):
        self.function = function          # fn: (parent, index, data) -> Queryable
        self.privacy_loss = privacy_loss  # Fixed privacy loss
    def eval(self, data) -> Queryable:    # Convenience method to invoke function, with null parent and index
        return self.function(None, None, data)


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(parent, index, data):  # Wrapper function that creates a dummy Queryable
            initial_state = function(data)
            def transition(_self, target_index_path, state, _question):
                return (state, state)
            return Queryable(parent, index, initial_state, transition)
        super().__init__(interactive_function, privacy_loss)
    def eval1(self, data):                # Convenience method to invoke function, get result from null query
        queryable = self.eval(data)
        return queryable.query(None)


# Makes an adaptive composition InteractiveMeasurement. Spawned Queryables expect their queries
# to be (non-Interactive)Measurements.
def make_adaptive_composition(budget):
    def function(parent, index, data):
        initial_state = (data, budget)
        def transition(_self, _target_index_path, state, question: Measurement):
            data, budget = state
            if question.privacy_loss > budget:
                raise Exception("Insufficient budget")
            budget -= question.privacy_loss
            answer = question.eval1(data)
            new_state = (data, budget)
            return (new_state, answer)
        return Queryable(parent, index, initial_state, transition)
    return InteractiveMeasurement(function, budget)


# Makes a sequential composition InteractiveMeasurement. Spawned Queryables must be queried sequentially;
# when a new Queryable is spawned, it becomes the active child, and previous children are implicitly retired.
# Using a retired child (either directly, or through one of its descendants) will raise an error.
def make_sequential_composition(budget):

    def function(parent, index, data):
        child_count = 0
        initial_state = (data, budget, child_count)

        def transition(self, target_index_path, state, question):
            data, budget, child_count = state
            if target_index_path is None:
                # Path is null, so target is this Queryable. That means we will spawn a child.
                if question.privacy_loss > budget:
                    raise Exception("Insufficient budget")
                budget -= question.privacy_loss
                # Assign the child the next available index.
                child_index = child_count
                # Question is an InteractiveMeasurement, so calling the function will spawn the child.
                answer = question.function(self, child_index, data)
                child_count += 1
            else:
                # Target is a descendant, so make sure the child involved is the last one created (no backtracking).
                if target_index_path[0] != child_count - 1:
                    raise Exception("Non-sequential query")
                # Allow the query to continue to the next descendant.
                answer = None
            new_state = (data, budget, child_count)
            return (new_state, answer)

        return Queryable(parent, index, initial_state, transition)

    return InteractiveMeasurement(function, budget)


# Constructor to make a simple Laplace (non-Interactive)Measurement
def make_base_laplace(sigma):
    def laplace(sigma):
        import random, math
        u = random.uniform(-0.5, 0.5)
        return math.copysign(1, u) * math.log(1.0 - 2.0 * abs(u)) * sigma
    return Measurement(lambda x: x + laplace(sigma), 1.0 / sigma)


# Converter from epsilon to Laplace sigma
def eps_to_sigma(epsilon):
    return 1 / epsilon


# Runs a (non-Interactive)Measurement
def test_noninteractive():
    print("NON-INTERACTIVE MEASUREMENT")
    data = 123.0
    measurement = make_base_laplace(eps_to_sigma(1.0))
    print("non-interactive =", measurement.eval1(data))


# Runs an adaptive composition (Measurement queries)
def test_adaptive():
    print("SIMPLE ADAPTIVE COMPOSITION")
    data = 123.0
    budget = 1.0
    print("make adaptive composition")
    adaptive = make_adaptive_composition(budget)
    print("    spawn queryable")
    queryable = adaptive.eval(data)
    print("        adaptive query 1 =", queryable.query(make_base_laplace(eps_to_sigma(budget / 2))))
    print("        adaptive query 2 =", queryable.query(make_base_laplace(eps_to_sigma(budget / 2))))
    try:
        print("        SHOULD'VE FAILED: adaptive query 3 =", queryable.query(make_base_laplace(eps_to_sigma(budget / 2))))
    except Exception as e:
        print("        got expected failure on adaptive query 3 =", e)


# Runs a sequential composition (InteractiveMeasurement queries)
def test_sequential():
    print("SEQUENTIAL COMPOSITION")
    data = 123.0
    budget = 1.0
    print("make sequential composition")
    sequential = make_sequential_composition(budget)
    print("    spawn root queryable")
    root_queryable = sequential.eval(data)

    print("        make adaptive composition (for sub-queryable 1 of sequential)")
    adaptive1 = make_adaptive_composition(budget / 2)
    print("        spawn sub-queryable 1")
    sub_queryable1 = root_queryable.query(adaptive1)
    print("            sub-queryable 1 / query 1 =", sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("        make adaptive (for sub-queryable 2 of sequential)")
    adaptive2 = make_adaptive_composition(budget / 2)
    print("        spawn sub-queryable 2")
    sub_queryable2 = root_queryable.query(adaptive2)
    print("            sub-queryable 2 / query 1 =", sub_queryable2.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("        backtrack to sub-queryable 1")
    try:
        print("            SHOULD'VE FAILED: sub-queryable 1 / query 2 =", sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))
    except Exception as e:
        print("            got expected failure on sub-queryable 1 / query 2 =", e)


# Runs a recursive sequential composition (InteractiveMeasurement queries)
def test_sequential_recursive():
    print("SEQUENTIAL COMPOSITION (RECURSIVE)")
    data = 123.0
    budget = 1.0
    print("make root sequential composition")
    root_sequential = make_sequential_composition(budget)
    print("    spawn root queryable")
    root_queryable = root_sequential.eval(data)

    print("        make sub-sequential composition (for sub-queryable 1 of root sequential)")
    sub_sequential = make_sequential_composition(budget / 2)
    print("        spawn sub-queryable 1")
    sub_queryable1 = root_queryable.query(sub_sequential)

    print("            make adaptive composition (for sub-sub-queryable 1 of sub-sequential)")
    sub_adaptive1 = make_adaptive_composition(budget / 2)
    print("            spawn sub-sub-queryable 1")
    sub_sub_queryable1 = sub_queryable1.query(sub_adaptive1)
    print("                sub-sub-queryable 1 / query 1 =", sub_sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("        make adaptive (for sub-queryable 2 of root sequential)")
    adaptive2 = make_adaptive_composition(budget / 2)
    print("        spawn sub-queryable 2")
    sub_queryable2 = root_queryable.query(adaptive2)
    print("            sub-queryable 2 / query 1 =", sub_queryable2.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("            backtrack to sub-sub-queryable 1")
    try:
        print("                SHOULD'VE FAILED: sub-sub-queryable 1 / query 2 =", sub_sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))
    except Exception as e:
        print("                got expected failure on sub-sub-queryable 1 / query 2 =", e)


def main():
    test_noninteractive()
    test_adaptive()
    test_sequential()


if __name__ == '__main__':
    main()
