import collections


class InteractiveMeasurement:

    def __init__(self, function, privacy_loss, description="InteractiveMeasurement"):
        # Domains, Metrics & Measures omitted for simplicity
        self.function = function
        self.privacy_loss = privacy_loss  # Fixed privacy loss for simplicity
        self.description = description    # For logging purposes only

    # Public interface
    def invoke(self, data) -> "Queryable":
        return self.function(data)


class Measurement(InteractiveMeasurement):

    def __init__(self, noninteractive_function, privacy_loss, description="Measurement"):
        def function(data):
            answer = noninteractive_function(data)
            def eval(self, _query):
                return answer, self.state
            return Queryable(None, eval, "FixedQueryable")
        super().__init__(function, privacy_loss, description)

    # Public interface for a fixed query
    def invoke1(self, data):
        queryable = self.invoke(data)
        return queryable.query(None)


class Odometer:

    def __init__(self, function, description="Odometer"):
        # Domains, Metrics & Measures omitted for simplicity
        self.function = function
        self.description = description  # For logging purposes only

    # Public interface
    def invoke(self, data) -> "Queryable":
        return self.function(data)


class Queryable:

    def __init__(self, state, eval, description="Queryable"):
        self.state = state
        self.eval = eval.__get__(self)  # fn: Q x S -> A x S
        self.listener = None            # Generalization of "parent"
        self.tag = None                 # Generalization of "sibling_index"
        self.description = description  # For logging purposes only

    # Public interface
    def query(self, query):
        answer, new_state = self.eval(query)
        self.state = new_state
        return answer

    def _set_listener(self, listener, tag):
        self.listener = listener
        self.tag = tag

    def _notify_listener(self, query):
        return self.listener.query(query) if self.listener is not None else None


def _get_name(self):
    if self.listener is None:
        return "root"
    elif self.listener.listener is None:
        return f"child_{self.tag}"
    else:
        return f"{self.listener._get_name()}_{self.tag}"
Queryable._get_name = _get_name

Queryable.__set_listener = Queryable._set_listener
def _set_listener(self, listener, tag):
    self.__set_listener(listener, tag)
    if self.state is not None:
        print(f"SPAWN+  {self._get_name():11} queryable={self}")
Queryable._set_listener = _set_listener

def _mark_root(self):
    self._set_listener(None, None)
    return self
Queryable._mark_root = _mark_root

Queryable._query = Queryable.query
def query(self, query):
    if query is not None:
        print(f"QUERY>  {self._get_name():11} question={query}")
    try:
        answer = self._query(query)
    except Exception as e:
        answer = f"ERROR: {e}"
    if query is not None:
        print(f"QUERY<  {self._get_name():11} answer={answer}")
    return answer
Queryable.query = query
def _notify_listener(self, query):
    return self.listener._query(query) if self.listener is not None else None
Queryable._notify_listener = _notify_listener

def __str__(self):
    return f"{self.description}(privacy_loss={self.privacy_loss})"
InteractiveMeasurement.__str__ = __str__

def __str__(self):
    fields = f"max_privacy_loss={self.state.max_privacy_loss}" if self.state is not None and "max_privacy_loss" in self.state._fields else ""
    return f"{self.description}({fields})"
Queryable.__str__ = __str__


CheckDescendantChange = collections.namedtuple("CheckDescendantChange", ["index", "new_privacy_loss", "pre_invoke"])
GetPrivacyLoss = collections.namedtuple("GetPrivacyLoss", [])

def make_concurrent_filter(max_privacy_loss):

    def function(data):
        # The structure of stuff we store in state
        State = collections.namedtuple("State", ["data", "max_privacy_loss", "privacy_loss", "child_privacy_losses"])
        state = State(data=data, max_privacy_loss=max_privacy_loss, privacy_loss=0, child_privacy_losses=[])

        def check_new_state(self, child_index, child_privacy_loss, pre_invoke):
            # Append or replace the child privacy loss, then calculate the new sum.
            new_child_privacy_losses = self.state.child_privacy_losses.copy()
            new_child_privacy_losses[child_index:] = [child_privacy_loss]
            new_privacy_loss = sum(new_child_privacy_losses)
            if pre_invoke:
                print(f"CHECK?  {self._get_name():11} child={child_index}, child_pl={child_privacy_loss}, new_pl={new_privacy_loss}, max_pl={self.state.max_privacy_loss}")
            # Check against our max privacy loss.
            if new_privacy_loss > state.max_privacy_loss:
                raise Exception(f"New privacy loss {new_privacy_loss} exceeds max privacy loss {max_privacy_loss}")
            # Notify our parent.
            self._notify_listener(CheckDescendantChange(index=self.tag, new_privacy_loss=new_privacy_loss, pre_invoke=pre_invoke))
            # Don't update state if pre-invoke.
            return self.state if pre_invoke else self.state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses)

        def eval(self, query):
            if isinstance(query, (InteractiveMeasurement, Odometer)):
                new_child_index = len(self.state.child_privacy_losses)
                new_child_privacy_loss = query.privacy_loss if isinstance(query, InteractiveMeasurement) else 0
                _new_state = check_new_state(self, new_child_index, new_child_privacy_loss, True)
                new_child = query.invoke(data)
                new_child._set_listener(self, new_child_index)
                new_state = check_new_state(self, new_child_index, new_child_privacy_loss, False)
                # Convenience to get non-interactive answer
                answer = new_child.query(None) if isinstance(query, Measurement) else new_child
            elif isinstance(query, CheckDescendantChange):
                new_state = check_new_state(self, query.index, query.new_privacy_loss, query.pre_invoke)
                answer = "OK"
            else:
                raise Exception(f"Unrecognized query {query}")
            return answer, new_state

        return Queryable(state, eval, "ConcurrentFilterQueryable")

    return InteractiveMeasurement(function, max_privacy_loss, "ConcurrentFilter")


def make_concurrent_odometer():

    def function(data):
        # The structure of stuff we store in state
        State = collections.namedtuple("State", ["data", "privacy_loss", "child_privacy_losses"])
        state = State(data=data, privacy_loss=0, child_privacy_losses=[])

        def check_new_state(self, child_index, child_privacy_loss, pre_invoke):
            # Append or replace the child privacy loss, then calculate the new sum.
            new_child_privacy_losses = self.state.child_privacy_losses.copy()
            new_child_privacy_losses[child_index:] = [child_privacy_loss]
            new_privacy_loss = sum(new_child_privacy_losses)
            if pre_invoke:
                print(f"CHECK?  {self._get_name():11} child={child_index}, child_pl={child_privacy_loss}, new_pl={new_privacy_loss}")
            # NB: The only difference from make_concurrent_filter is that there's no budget check here.
            # Notify our parent.
            self._notify_listener(CheckDescendantChange(index=self.tag, new_privacy_loss=new_privacy_loss, pre_invoke=pre_invoke))
            # Don't update state if pre-invoke.
            return self.state if pre_invoke else self.state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses)

        def eval(self, query):
            if isinstance(query, (InteractiveMeasurement, Odometer)):
                new_child_index = len(self.state.child_privacy_losses)
                new_child_privacy_loss = query.privacy_loss if isinstance(query, InteractiveMeasurement) else 0
                _new_state = check_new_state(self, new_child_index, new_child_privacy_loss, True)
                new_child = query.invoke(data)
                new_child._set_listener(self, new_child_index)
                new_state = check_new_state(self, new_child_index, new_child_privacy_loss, False)
                # Convenience to get non-interactive answer
                answer = new_child.query(None) if isinstance(query, Measurement) else new_child
            elif isinstance(query, CheckDescendantChange):
                new_state = check_new_state(self, query.index, query.new_privacy_loss, query.pre_invoke)
                answer = "OK"
            elif isinstance(query, GetPrivacyLoss):
                new_state = self.state
                answer = self.state.privacy_loss
            else:
                raise Exception(f"Unrecognized query {query}")
            return answer, new_state

        return Queryable(state, eval, "ConcurrentOdometerQueryable")

    return Odometer(function, "ConcurrentOdometer")


def make_odomoter_to_filter(odometer, max_privacy_loss):

    def function(data):
        filter = make_concurrent_filter(max_privacy_loss)
        filter_queryable = filter.invoke(data)
        return filter_queryable.query(odometer)

    return InteractiveMeasurement(function, max_privacy_loss, "OdometerToFilter")


def make_sequential_filter(max_privacy_loss):

    def function(data):
        # The structure of stuff we store in state
        State = collections.namedtuple("State", ["data", "max_privacy_loss", "privacy_loss", "child_privacy_losses", "current_child_index"])
        state = State(data=data, max_privacy_loss=max_privacy_loss, privacy_loss=0, child_privacy_losses=[], current_child_index=-1)

        def check_new_state(self, child_index, child_privacy_loss, pre_invoke):
            # Append or replace the child privacy loss, then calculate the new sum.
            new_child_privacy_losses = self.state.child_privacy_losses.copy()
            new_child_privacy_losses[child_index:] = [child_privacy_loss]
            new_privacy_loss = sum(new_child_privacy_losses)
            if pre_invoke:
                print(f"CHECK?  {self._get_name():11} child={child_index}, child_pl={child_privacy_loss}, new_pl={new_privacy_loss}, max_pl={self.state.max_privacy_loss}, current_child={self.state.current_child_index}")
            # Check against our max privacy loss.
            if new_privacy_loss > state.max_privacy_loss:
                raise Exception(f"New privacy loss {new_privacy_loss} exceeds max privacy loss {max_privacy_loss}")
            # Check sequentiality.
            new_current_child_index = child_index
            if new_current_child_index < self.state.current_child_index:
                raise Exception(f"Non-sequential access of children")
            # Notify our parent.
            self._notify_listener(CheckDescendantChange(index=self.tag, new_privacy_loss=new_privacy_loss, pre_invoke=pre_invoke))
            # Don't update state if pre-invoke.
            return self.state if pre_invoke else self.state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses, current_child_index=new_current_child_index)

        def eval(self, query):
            if isinstance(query, (InteractiveMeasurement, Odometer)):
                new_child_index = len(self.state.child_privacy_losses)
                new_child_privacy_loss = query.privacy_loss if isinstance(query, InteractiveMeasurement) else 0
                _new_state = check_new_state(self, new_child_index, new_child_privacy_loss, True)
                new_child = query.invoke(data)
                new_child._set_listener(self, new_child_index)
                new_state = check_new_state(self, new_child_index, new_child_privacy_loss, False)
                # Convenience to get non-interactive answer
                answer = new_child.query(None) if isinstance(query, Measurement) else new_child
            elif isinstance(query, CheckDescendantChange):
                new_state = check_new_state(self, query.index, query.new_privacy_loss, query.pre_invoke)
                answer = "OK"
            elif isinstance(query, GetPrivacyLoss):
                new_state = self.state
                answer = self.state.privacy_loss
            else:
                raise Exception(f"Unrecognized query {query}")
            return answer, new_state

        return Queryable(state, eval, "SequentialFilterQueryable")

    return InteractiveMeasurement(function, max_privacy_loss, "SequentialFilter")


# Constructor to make a simple Laplace (non-Interactive)Measurement
def make_base_laplace(sigma):
    def laplace(sigma):
        import random, math
        u = random.uniform(-0.5, 0.5)
        return math.copysign(1, u) * math.log(1.0 - 2.0 * abs(u)) * sigma
    return Measurement(lambda x: x + laplace(sigma), 1.0 / sigma, "BaseLaplace")


# Converter from epsilon to Laplace sigma
def eps_to_sigma(epsilon):
    return 1 / epsilon


def test_noninteractive():
    print("\nNON-INTERACTIVE MEASUREMENT")
    data = 123.0
    measurement = make_base_laplace(eps_to_sigma(1.0))
    answer = measurement.invoke1(data)
    assert type(answer) == float


def test_concurrent_filter():
    print("\nCONCURRENT FILTER")
    budget = 1.0
    data = 123.0
    root = make_concurrent_filter(budget).invoke(data)._mark_root()
    answer = root.query(make_base_laplace(eps_to_sigma(budget * 0.5)))
    assert type(answer) == float
    answer = root.query(make_base_laplace(eps_to_sigma(budget * 0.5)))
    assert type(answer) == float

    answer = root.query(make_base_laplace(eps_to_sigma(budget * 0.5)))  # Should fail here
    assert answer == f"ERROR: New privacy loss {1.5 * budget} exceeds max privacy loss {budget}"


def test_concurrent_filter_nested():
    print("\nCONCURRENT FILTER (NESTED)")
    budget = 1.0
    data = 123.0
    root = make_concurrent_filter(budget).invoke(data)._mark_root()
    child_0 = root.query(make_concurrent_filter(budget * 0.5))
    child_0_0 = child_0.query(make_concurrent_filter(budget * 0.25))
    answer = child_0_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    answer = child_0_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    child_0_1 = child_0.query(make_concurrent_filter(budget * 0.25))
    answer = child_0_1.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    answer = child_0_1.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    child_1 = root.query(make_concurrent_filter(budget * 0.5))
    child_1_0 = child_1.query(make_concurrent_filter(budget * 0.25))
    answer = child_1_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float

    answer = child_1_0.query(make_base_laplace(eps_to_sigma(budget * 0.5)))  # Should fail here
    assert answer == f"ERROR: New privacy loss {0.625 * budget} exceeds max privacy loss {0.25 * budget}"


def test_concurrent_odometer():
    print("\nCONCURRENT ODOMETER")
    budget = 1.0
    data = 123.0
    root = make_concurrent_odometer().invoke(data)._mark_root()
    answer = root.query(make_base_laplace(eps_to_sigma(budget * 0.5)))
    assert type(answer) == float
    answer = root.query(make_base_laplace(eps_to_sigma(budget * 0.5)))
    assert type(answer) == float

    answer = root.query(GetPrivacyLoss())
    assert answer == budget


def test_odometer_to_filter():
    print("\nODOMETER TO FILTER")
    budget = 1.0
    data = 123.0
    odometer = make_concurrent_odometer()
    filter = make_odomoter_to_filter(odometer, budget)
    filter_queryable = filter.invoke(data)
    answer = filter_queryable.query(make_base_laplace(eps_to_sigma(budget * 0.5)))
    assert type(answer) == float
    answer = filter_queryable.query(make_base_laplace(eps_to_sigma(budget * 0.5)))
    assert type(answer) == float

    answer = filter_queryable.query(make_base_laplace(eps_to_sigma(budget * 0.5)))  # Should fail here
    assert answer == f"ERROR: New privacy loss {1.5 * budget} exceeds max privacy loss {budget}"


def test_sequential_filter():
    print("\nSEQUENTIAL FILTER")
    budget = 1.0
    data = 123.0
    root = make_sequential_filter(budget).invoke(data)._mark_root()
    child_0 = root.query(make_concurrent_filter(budget * 0.25))
    answer = child_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    child_1 = root.query(make_concurrent_filter(budget * 0.25))
    answer = child_1.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    child_2 = root.query(make_concurrent_filter(budget * 0.25))
    answer = child_2.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float

    answer = child_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))  # Should fail here
    assert answer == f"ERROR: Non-sequential access of children"


def test_sequential_filter_nested():
    print("\nSEQUENTIAL FILTER (NESTED)")
    budget = 1.0
    data = 123.0
    root = make_sequential_filter(budget).invoke(data)._mark_root()
    child_0 = root.query(make_concurrent_filter(budget * 0.25))
    child_0_0 = child_0.query(make_concurrent_filter(budget * 0.25))
    answer = child_0_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    child_1 = root.query(make_concurrent_filter(budget * 0.25))
    child_1_0 = child_1.query(make_concurrent_filter(budget * 0.25))
    answer = child_1_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float
    child_2 = root.query(make_concurrent_filter(budget * 0.25))
    child_2_0 = child_2.query(make_concurrent_filter(budget * 0.25))
    answer = child_2_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))
    assert type(answer) == float

    answer = child_0_0.query(make_base_laplace(eps_to_sigma(budget * 0.125)))  # Should fail here
    assert answer == f"ERROR: Non-sequential access of children"


def main():
    test_noninteractive()
    test_concurrent_filter()
    test_concurrent_filter_nested()
    test_concurrent_odometer()
    test_odometer_to_filter()
    test_sequential_filter()
    test_sequential_filter_nested()


if __name__ == '__main__':
    main()
