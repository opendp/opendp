import collections

class Invokable:  # Common interface for InteractiveMeasurement & Odometer

    def __init__(self, function):
        self.function = function

    def invoke(self, data):  # Convenience method to invoke function
        return self.function(data)


class InteractiveMeasurement(Invokable):

    def __init__(self, function, privacy_loss):
        super().__init__(function)
        self.privacy_loss = privacy_loss  # Fixed privacy loss


def make_fixed_queryable(answer):
    def eval(self, query):
        return answer
    return PureQueryable(None, eval)


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(data):  # Wrapper function that creates a FixedQueryable
            answer = function(data)  # Invoke function once, store result as state
            return make_fixed_queryable(answer)
        super().__init__(interactive_function, privacy_loss)
    def invoke1(self, data):                # Convenience method to invoke function, get result from null query
        queryable = self.invoke(data)
        return queryable.evaluate(None)


class Odometer(Invokable):
    pass


class PureQueryable:

    def __init__(self, state, eval):
        self.state = state
        self.eval = eval


class Queryable:

    def __init__(self, state, _provision_for_self=None, _execute_for_self=None, _provision_for_child=None, _execute_for_child=None):
        self.parent = None
        self.sibling_index = -1
        self.children = []
        self.provisional_state = None
        self.state = state
        self._provision_for_self = _provision_for_self or (lambda _query, state: (None, state))
        self._execute_for_self = _execute_for_self or (lambda _query, state: (None, None, state))
        self._provision_for_child = _provision_for_child or (lambda _child_index, _child_request, state: (None, state))
        self._execute_for_child = _execute_for_child or (lambda _child_index, _child_request, state: (None, state))

    def evaluate(self, query):
        if not self.provision_for_self(query):
            return "Sorry Charlie!"
        answer = self.execute_for_self(query)
        if isinstance(answer, Queryable):
            answer.parent = self
            answer.sibling_index = len(self.children)
            self.children += [answer]
        return answer

    def provision_for_self(self, query):
        try:
            # Call the supplied implementation, returning a request for parent and provisional state
            parent_request, provisional_state = self._provision_for_self(query, self.state)
        except Exception as e:
            print(f"Provisioning for self failed: {e}")
            return False
        if self.parent is not None:
            # Move up the tree
            if not self.parent.provision_for_child(self.sibling_index, parent_request):
                return False
        # Update provisional state
        self.provisional_state = provisional_state
        return True

    def execute_for_self(self, query):
        try:
            # Call the supplied implementation (using provisional state)
            answer, parent_request, state = self._execute_for_self(query, self.provisional_state)
        except Exception as e:
            print(f"Execution for self failed: {e}")
            return None
        if self.parent is not None:
            # Move up the tree
            self.parent.execute_for_child(self.sibling_index, parent_request)
        # Update state
        self.state = state
        self.provisional_state = None
        return answer

    def provision_for_child(self, child_index, child_request):
        # Call the supplied implementation
        try:
            # Call the supplied implementation, returning a request for parent and provisional state
            parent_request, provisional_state = self._provision_for_child(child_index, child_request, self.state)
        except Exception as e:
            print(f"Provisioning for child failed: {e}")
            return False
        if self.parent is not None:
            # Move up the tree
            if not self.parent.provision_for_child(self.sibling_index, parent_request):
                return False
        # Update provisional state
        self.provisional_state = provisional_state
        return True

    def execute_for_child(self, child_index, child_request):
        try:
            # Call the supplied implementation (using provisional state)
            parent_request, state = self._execute_for_child(child_index, child_request, self.provisional_state)
        except Exception as e:
            print(f"Execution for child failed: {e}")
            return
        if self.parent is not None:
            # Move up the tree
            self.parent.execute_for_child()
        # Update state
        self.state = state
        self.provisional_state = None


def make_concurrent_odometer():
    State = collections.namedtuple("State", ["data", "privacy_loss", "child_privacy_losses"])

    def provision_for_self(query, state):
        if isinstance(query, InteractiveMeasurement):
            child_privacy_losses = state.child_privacy_losses + [query.privacy_loss]
            privacy_loss = sum(child_privacy_losses)
            state = state._replace(privacy_loss=privacy_loss, child_privacy_losses=child_privacy_losses)
            return privacy_loss, state
        elif isinstance(query, Odometer):
            child_privacy_losses = state.child_privacy_losses + [0]
            state = state._replace(child_privacy_losses=child_privacy_losses)
            return state.privacy_loss, state
        else:
            raise Exception(f"Unknown query type {query}")

    def execute_for_self(query, state):
        answer = query.invoke(state.data)
        # state.privacy_loss has already been updated in provision_for_self()
        return answer, state.privacy_loss, state

    def provision_for_child(child_index, child_request, state):
        child_privacy_losses = state.child_privacy_losses[:]
        child_privacy_losses[child_index] = child_request
        privacy_loss = sum(child_privacy_losses)
        return privacy_loss, state._replace(privacy_loss=privacy_loss, child_privacy_losses=child_privacy_losses)

    def function(data):
        state = State(data=data, privacy_loss=0, child_privacy_losses=[])
        return Queryable(state, provision_for_self, execute_for_self, provision_for_child)

    return Odometer(function)


def make_concurrent_filter(budget):
    State = collections.namedtuple("State", ["data", "budget", "privacy_loss", "child_privacy_losses"])

    def provision_for_self(query, state):
        if isinstance(query, InteractiveMeasurement):
            child_privacy_losses = state.child_privacy_losses + [query.privacy_loss]
            privacy_loss = sum(child_privacy_losses)
            if privacy_loss > budget:
                raise Exception(f"Requested self privacy loss of {privacy_loss} is greater than budget {budget}")
            privacy_loss, state._replace(privacy_loss=privacy_loss, child_privacy_losses=child_privacy_losses)
        elif isinstance(query, Odometer):
            child_privacy_losses = state.child_privacy_losses + [0]
            return state._replace(child_privacy_losses=child_privacy_losses)
        else:
            raise Exception(f"Unknown query type {query}")

    def execute_for_self(query, state):
        answer = query.invoke(state.data)
        # state.privacy_loss has already been updated in provision_for_self()
        return answer, state.privacy_loss, state

    def provision_for_child(child_index, child_request, state):
        child_privacy_losses = state.child_privacy_losses[:]
        child_privacy_losses[child_index] = child_request
        privacy_loss = sum(child_privacy_losses)
        if privacy_loss > budget:
            raise Exception(f"Requested child privacy loss of {privacy_loss} is greater than budget {budget}")
        return state.privacy_loss, state._replace(privacy_loss=privacy_loss, child_privacy_losses=child_privacy_losses)

    def function(data):
        state = State(data=data, budget=budget, privacy_loss=0, child_privacy_losses=[])
        return Queryable(state, provision_for_self, execute_for_self, provision_for_child)

    return InteractiveMeasurement(function, budget)


def make_sequential_odometer():
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
