class Queryable:
    def __init__(self, context, initial_state, transition):
        self.context = context        # Static context data (the part of "state" that doesn't change)
        self.state = initial_state    # Variable state
        self.transition = transition  # fn: (context, state, question) -> (state, answer)
    def query(self, question):
        (new_state, answer) = self.transition(self.context, self.state, question)
        self.state = new_state
        return answer


class InteractiveMeasurement:
    def __init__(self, function, privacy_loss):
        self.function = function          # fn: data -> Queryable
        self.privacy_loss = privacy_loss  # Fixed privacy loss
    def eval(self, data) -> Queryable:    # Convenience method to invoke function
        return self.function(data)


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(data):  # Wraps static function to generate a Queryable
            context = function(data)
            def transition(context, _state, _question):
                return None, context
            return Queryable(context, None, transition)
        super().__init__(interactive_function, privacy_loss)
    def eval1(self, data):               # Convenience method to invoke function, get result from null query
        queryable = self.eval(data)
        return queryable.query(None)


# Makes an adaptive composition InteractiveMeasurement. Spawned Queryables require their queries
# to be (non-Interactive) Measurements.
def make_adaptive_composition(budget):
    def function(data):
        context = data          # Static context contains the input data
        initial_state = budget  # State contains the remaining budget
        def transition(context, state, question: Measurement):
            data = context
            budget = state
            if question.privacy_loss > budget:
                raise Exception("Insufficient budget")
            budget -= question.privacy_loss
            new_state = budget
            answer = question.eval1(data)
            return (new_state, answer)
        return Queryable(context, initial_state, transition)
    return InteractiveMeasurement(function, budget)


# Makes a sequential composition InteractiveMeasurement. Spawned Queryables require their queries
# to be (non-Interactive) Measurements (whose Queryables must then be (non-Interactive) Measurements).
def make_sequential_composition(budget):

    # State which is shared across sub-Queryables is managed by a coordinator. For convenience, we model this
    # using a Queryable itself, though this isn't necessary.
    coordinator_context = None                # Static context is unused
    coordinator_initial_state = (budget, [])  # Variable state contains budget and list of children
    def coordinator_transition(_context, state, question):
        (budget, children) = state
        (data, index, original_question) = question  # Question contains data, child index, and original question
        if index is None:
            # Spawn of a new child
            if original_question.privacy_loss > budget:
                raise Exception("Insufficient budget")
            budget -= original_question.privacy_loss
            new_index = len(children)
            new_child = original_question.eval(data)  # Original question is an InteractiveMeasurement.
            children.append(new_child)
            answer = new_index  # We return the index of the spawned child
        else:
            # Query to an existing child
            if index != len(children) - 1:  # Make sure the child we're querying is the last created one (no backtracking)
                raise Exception("Non-sequential query")
            child = children[index]
            answer = child.query(original_question)
        new_state = (budget, children)
        return (new_state, answer)
    coordinator = Queryable(coordinator_context, coordinator_initial_state, coordinator_transition)

    def function(data):
        context = (coordinator, data)  # Static context contains coordinator and data
        initial_state = None           # Variable state is unused
        def transition(context, _state, question: InteractiveMeasurement):
            (coordinator, data) = context
            # Construct a spawn query for the coordinator
            null_index = None
            question = (data, null_index, question)
            child_index = coordinator.query(question)  # Returns the index of the spawned child

            # We need a wrapper Queryable for each spawned child. This will intercept queries to the child,
            # and dispatch them through the coordinator.
            child_wrapper_context = (coordinator, data, child_index)
            child_wrapper_initial_state = None
            def child_wrapper_transition(context, _state, question):
                (coordinator, data, index) = context
                # Construct an existing child query for the coordinator
                question = (data, index, question)
                answer = coordinator.query(question)  # Returns whatever the child returns
                # If we wanted to support neeper nesting, it'd happen here.
                return (None, answer)
            child_wrapper = Queryable(child_wrapper_context, child_wrapper_initial_state, child_wrapper_transition)
            return (None, child_wrapper)
        return Queryable(context, initial_state, transition)

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
