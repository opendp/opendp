import collections


class InteractiveMeasurement:

    def __init__(self, function, privacy_loss):
        # Domains, Metrics & Measures omitted for simplicity
        self.function = function
        self.privacy_loss = privacy_loss  # Fixed privacy loss for simplicity

    # Public interface
    def invoke(self, data) -> "Queryable":
        return self.function(data)


class Measurement(InteractiveMeasurement):

    def __init__(self, function, privacy_loss):
        def make_fixed_queryable(data):
            answer = function(data)
            def eval(_tag, _query, state):
                return state
            return Queryable(answer, eval)
        super().__init__(make_fixed_queryable, privacy_loss)

    # Public interface for a fixed query
    def invoke1(self, data):
        queryable = self.invoke(data)
        return queryable.query(None)


class Odometer:

    def __init__(self, function):
        # Domains, Metrics & Measures omitted for simplicity
        self.function = function

    # Public interface
    def invoke(self, data) -> "Queryable":
        return self.function(data)


class OriginalQueryable:

    def __init__(self, state, eval):
        self.state = state
        self.eval = eval  # fn: Q x S -> A x S

    # Public interface
    def query(self, query):
        answer, new_state = self.eval(query, self.state)
        self.state = new_state
        return answer


class Queryable:

    def __init__(self, state, eval):
        self.state = state
        self.eval = eval  # fn: T x Q x S -> A x S

    # Queries are marked with a tag, which allows us to distinguish between user and system queries.
    def _query(self, tag, query):
        answer, new_state = self.eval(tag, query)
        self.state = new_state
        return answer

    # Public interface
    def query(self, query):
        return self._query("__USER__", query)


def make_naive_leaf_queryable(state, process_query):
    # The structure of stuff we store in state
    State = collections.namedtuple("State", ["parent", "index", "inner_state"])
    state = State(parent=None, index=-1, inner_state=state)

    def eval(tag, query, state):
        if tag == "__USER__":
            answer, new_inner_state, change = process_query(query, state.inner_state)  # Raises exception on failure
            if state.parent is not None:
                parent_query = (state.index, change)
                _ignored = state.parent._query("CHILD_DID_CHANGE", parent_query)  # Raises exception on failure
            new_state = state._replace(inner_state=new_inner_state)

        elif tag == "JOIN_TREE":
            answer = "OK"
            parent, index = query
            new_state = state._replace(parent=parent, index=index)

        else:
            raise Exception(f"Unknown query tag {tag}")

        return answer, new_state

    return Queryable(state, eval)


def make_naive_branch_queryable(state, process_query, process_child_change):
    State = collections.namedtuple("State", ["children", "inner_state"])
    state = State(children=[], inner_state=state)

    def eval(tag, query, state):
        if tag == "__USER__":
            answer, new_inner_state = process_query(query, state.inner_state)  # Raises exception on failure
            if isinstance(answer, Queryable):
                new_child = answer
                new_child_index = len(state.children)
                # TODO: How to get reference to the Queryable object here?
                self = None
                child_query = (self, new_child_index)
                new_child._query("JOIN_TREE", child_query)
                new_children = state.children + [new_child]
                new_state = state._replace(children=new_children, inner_state=new_inner_state)
            else:
                new_state = state._replace(inner_state=new_inner_state)

        elif tag == "CHILD_DID_CHANGE":
            child_index, child_change = query
            change, new_inner_state = process_child_change(child_index, child_change, state.inner_state)  # Raises exception on failure
            if state.parent is not None:
                parent_query = (state.index, change)
                state.parent._query("CHILD_DID_CHANGE", parent_query)  # Raises exception on failure
            answer = "OK"
            new_state = state._replace(inner_state=new_inner_state)

        else:
            raise Exception(f"Unknown query tag {tag}")

        return answer, new_state

    return Queryable(state, eval)


def make_naive_node_queryable(state, process_query, process_child_change):
    State = collections.namedtuple("State", ["parent", "index", "children", "inner_state"])
    state = State(parent=None, index=-1, children=[], inner_state=state)

    def eval(tag, query, state):
        if tag == "__USER__":
            answer, change, new_inner_state = process_query(query, state.inner_state)  # Raises exception on failure
            if state.parent is not None:
                parent_query = (state.index, change)
                _ignored = state.parent._query("CHILD_DID_CHANGE", parent_query)  # Raises exception on failure
            if isinstance(answer, Queryable):
                new_child = answer
                new_child_index = len(state.children)
                # TODO: How to get reference to the Queryable object here?
                self = None
                child_query = (self, new_child_index)
                new_child._query("JOIN_TREE", child_query)
                new_children = state.children + [new_child]
            else:
                new_children = state.children
            new_state = state._replace(children=new_children, inner_state=new_inner_state)

        elif tag == "CHILD_DID_CHANGE":
            child_index, child_change = query
            change, new_inner_state = process_child_change(child_index, child_change, state.inner_state)  # Raises exception on failure
            if state.parent is not None:
                parent_query = (state.index, change)
                state.parent._query("CHILD_DID_CHANGE", parent_query)  # Raises exception on failure
            answer = "OK"
            new_state = state._replace(inner_state=new_inner_state)

        elif tag == "JOIN_TREE":
            parent, index = query
            answer = "OK"
            new_state = state._replace(parent=parent, index=index)

        else:
            raise Exception(f"Unknown query tag {tag}")

        return answer, new_state

    return Queryable(state, eval)


def make_better_node_queryable(state, provision_query, execute_query, process_child_change):
    State = collections.namedtuple("State", ["parent", "index", "children", "provisional_inner_state", "inner_state"])
    state = State(parent=None, index=-1, children=[], provisional_inner_state=None, inner_state=state)

    def eval(tag, query, state):
        if tag == "__USER__":
            change, interim_inner_state = provision_query(query, state.inner_state)  # Raises exception on failure
            if state.parent is not None:
                parent_query = (state.index, change)
                _ignored = state.parent._query("CHILD_WILL_CHANGE", parent_query)  # Raises exception on failure
            answer, new_inner_state = execute_query(query, interim_inner_state)
            if state.parent is not None:
                parent_query = state.index
                _ignored = state.parent._query("CHILD_DID_CHANGE", parent_query)  # Not expected to fail
            if isinstance(answer, Queryable):
                new_child = answer
                new_child_index = len(state.children)
                # TODO: How to get reference to the Queryable object here?
                self = None
                child_query = (self, new_child_index)
                new_child._query("JOIN_TREE", child_query)
                new_children = state.children + [new_child]
            else:
                new_children = state.children
            new_state = state._replace(children=new_children, inner_state=new_inner_state)

        elif tag == "CHILD_WILL_CHANGE":
            child_index, child_change = query
            change, provisional_inner_state = process_child_change(child_index, child_change, state.inner_state)  # Raises exception on failure
            if state.parent is not None:
                parent_query = (state.index, change)
                state.parent._query("CHILD_WILL_CHANGE", parent_query)  # Raises exception on failure
            answer = "OK"
            new_state = state._replace(provisional_inner_state=provisional_inner_state)

        elif tag == "CHILD_DID_CHANGE":
            if state.parent is not None:
                parent_query = state.index
                state.parent._query("CHILD_DID_CHANGE", parent_query)  # Not expected to fail
            answer = "OK"
            new_state = state._replace(inner_state=state.provisional_inner_state, provisional_inner_state=None)

        elif tag == "JOIN_TREE":
            parent, index = query
            answer = "OK"
            new_state = state._replace(parent=parent, index=index)

        else:
            raise Exception(f"Unknown query tag {tag}")

        return answer, new_state

    Queryable(state, eval)


def make_node_queryable(state, provision_query, execute_query, process_child_change):
    return make_better_node_queryable(state, provision_query, execute_query, process_child_change)


def make_concurrent_odometer_of_odometers():
    State = collections.namedtuple("State", ["data", "privacy_loss", "child_privacy_losses"])

    def provision_query(_query, state):
        change = 0
        new_child_privacy_losses = state.child_privacy_losses + [0]
        new_state = state._replace(child_privacy_losses=new_child_privacy_losses)
        return change, new_state

    def execute_query(query, state):
        odometer = query
        answer = odometer.invoke(state.data)
        # state has already been updated in provision_query()
        return answer, state

    def process_child_change(child_index, child_change, state):
        new_child_privacy_losses = state.child_privacy_losses[:]
        new_child_privacy_losses[child_index] = child_change
        new_privacy_loss = sum(new_child_privacy_losses)
        change = new_privacy_loss
        new_state = state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses)
        return change, new_state

    def function(data):
        state = State(data=data, privacy_loss=0, child_privacy_losses=[])
        return make_node_queryable(state, provision_query, execute_query, process_child_change)

    return Odometer(function)


def make_concurrent_filter_of_filters(budget):
    State = collections.namedtuple("State", ["data", "budget", "privacy_loss", "child_privacy_losses"])

    def provision_query(query, state):
        filter = query
        new_child_privacy_losses = state.child_privacy_losses + [filter.privacy_loss]
        new_privacy_loss = sum(new_child_privacy_losses)
        if new_privacy_loss > budget:
            raise Exception(f"Requested query privacy loss of {new_privacy_loss} is greater than budget {budget}")
        change = new_privacy_loss
        new_state = state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses)
        return change, new_state

    def execute_query(query, state):
        filter = query
        answer = filter.invoke(state.data)
        # state has already been updated in provision_query()
        return answer, state

    def process_child_change(child_index, child_change, state):
        change = state.privacy_loss
        return change, state

    def function(data):
        state = State(data=data, budget=budget, privacy_loss=0, child_privacy_losses=[])
        return make_node_queryable(state, provision_query, execute_query, process_child_change)

    return InteractiveMeasurement(function, budget)


def odometer_queryable_to_filter_queryable(queryable, budget):
    # TODO
    pass


def make_sequential_odometer():
    # TODO: TRANSLATE TO NEW ARCHITECTURE
    State = collections.namedtuple("State", ["data", "current_child_index", "privacy_loss", "child_privacy_losses"])

    def provision_for_self(query, state):
        if isinstance(query, InteractiveMeasurement):
            child_privacy_losses = state.child_privacy_losses + [query.privacy_loss]
            privacy_loss = sum(child_privacy_losses)
            return privacy_loss, state._replace(privacy_loss=privacy_loss, child_privacy_losses=child_privacy_losses)
        elif isinstance(query, Odometer):
            child_privacy_losses = state.child_privacy_losses + [0]
            return state.privacy_loss, state._replace(child_privacy_losses=child_privacy_losses)
        else:
            raise Exception(f"Unknown query type {query}")

    def execute_for_self(query, state):
        answer = query.invoke(state.data)
        return answer, state.privacy_loss, state

    def provision_for_child(child_index, child_request, state):
        if child_index < state.current_child_index:
            raise Exception(f"Attempt to use child {child_index} after child {state.current_child_index}")
        current_child_index = child_index
        child_privacy_losses = state.child_privacy_losses[:]
        child_privacy_losses[child_index] = child_request
        privacy_loss = sum(child_privacy_losses)
        return privacy_loss, state._replace(current_child_index=current_child_index, privacy_loss=privacy_loss, child_privacy_losses=child_privacy_losses)

    def function(data):
        state = State(data=data, current_child_index=-1, privacy_loss=0, child_privacy_losses=[])
        return Queryable(state, execute_for_self, provision_for_self, provision_for_child)

    return Odometer(function)


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
    print("non-interactive =", measurement.invoke1(data))


# Runs an adaptive composition (Measurement queries)
def test_adaptive():
    print("SIMPLE ADAPTIVE COMPOSITION")
    data = 123.0
    budget = 1.0
    print("make adaptive composition")
    adaptive = make_concurrent_filter(budget)
    print("    spawn queryable")
    queryable = adaptive.invoke(data)
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
    sequential = make_sequential_filter(budget)
    print("    spawn root queryable")
    root_queryable = sequential.invoke(data)

    print("        make adaptive composition (for sub-queryable 1 of sequential)")
    adaptive1 = make_concurrent_filter(budget / 2)
    print("        spawn sub-queryable 1")
    sub_queryable1 = root_queryable.query(adaptive1)
    print("            sub-queryable 1 / query 1 =", sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("        make adaptive (for sub-queryable 2 of sequential)")
    adaptive2 = make_filter(budget / 2)
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
    root_queryable = root_sequential.invoke(data)

    print("        make sub-sequential composition (for sub-queryable 1 of root sequential)")
    sub_sequential = make_sequential_composition(budget / 2)
    print("        spawn sub-queryable 1")
    sub_queryable1 = root_queryable.query(sub_sequential)

    print("            make adaptive composition (for sub-sub-queryable 1 of sub-sequential)")
    sub_adaptive1 = make_filter(budget / 2)
    print("            spawn sub-sub-queryable 1")
    sub_sub_queryable1 = sub_queryable1.query(sub_adaptive1)
    print("                sub-sub-queryable 1 / query 1 =", sub_sub_queryable1.query(make_base_laplace(eps_to_sigma(budget / 4))))

    print("        make adaptive (for sub-queryable 2 of root sequential)")
    adaptive2 = make_filter(budget / 2)
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
