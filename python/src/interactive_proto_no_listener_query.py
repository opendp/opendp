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

    def __init__(self, noninteractive_function, privacy_loss):
        def function(data):
            answer = noninteractive_function(data)
            def eval(self, _query):
                return answer, self.state
            return Queryable(None, eval)
        super().__init__(function, privacy_loss)

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


class Queryable:

    def __init__(self, state, eval):
        self.state = state
        self.eval = eval.__get__(self)  # fn: Q x S -> A x S

    # Public interface
    def query(self, query):
        answer, new_state = self.eval(query)
        self.state = new_state
        return answer


################################################################################
# BEGIN LOGGING CODE
# THIS SECTION IS ONLY FOR DEBUGGING PURPOSES, AND CAN BE IGNORED
################################################################################
import inspect

def _caller_func(depth=1):
    return inspect.stack()[depth + 1].function

def _caller_func_is_test(depth=1):
    return _caller_func(depth + 1).startswith("test_")

InteractiveMeasurement.___init__ = InteractiveMeasurement.__init__
def __init__(self, function, privacy_loss):
    self.___init__(function, privacy_loss)
    constructor = _caller_func()
    if not constructor.startswith("make_"):
        constructor = _caller_func(2)
    self.description = "".join([word.capitalize() for word in constructor[5:].split("_")]) if constructor.startswith("make_") else "UNKNOWN"
InteractiveMeasurement.__init__ = __init__

Odometer.___init__ = Odometer.__init__
def __init__(self, function):
    self.___init__(function)
    constructor = _caller_func()
    self.description = "".join([word.capitalize() for word in constructor[5:].split("_")]) if constructor.startswith("make_") else "UNKNOWN"
Odometer.__init__ = __init__

Queryable.___init__ = Queryable.__init__
def __init__(self, state, eval):
    self.___init__(state, eval)
    interactive_measurement = inspect.getargvalues(inspect.stack()[2].frame).locals["self"]
    self.description = f"{interactive_measurement.description}Queryable"
    if self.state is not None:
        self._log("SPAWN+", queryable=self)
Queryable.__init__ = __init__

def _get_parent(self):
    return getattr(self.state, "parent", None)
Queryable._get_parent = _get_parent

def _get_sibling_index(self):
    return getattr(self.state, "sibling_index", None)
Queryable._get_sibling_index = _get_sibling_index

def _get_name(self):
    if self._get_parent() is None:
        return "root"
    elif self._get_parent()._get_parent() is None:
        return f"child_{self._get_sibling_index()}"
    else:
        return f"{self._get_parent()._get_name()}_{self._get_sibling_index()}"
Queryable._get_name = _get_name

Queryable._query = Queryable.query
def query(self, query):
    if _caller_func_is_test():
        if query is not None:
            self._log("QUERY>", question=query)
        try:
            answer = self._query(query)
        except Exception as e:
            answer = f"ERROR: {e}"
        if query is not None:
            self._log("QUERY<", answer=answer)
        return answer
    else:
        return self._query(query)
Queryable.query = query

def _log(self, event, **kwargs):
    print(f"{event:6}  {self._get_name():11} {', '.join(f'{k}={v}' for k, v in kwargs.items())}")
Queryable._log = _log

def __str__(self):
    return f"{self.description}(privacy_loss={self.privacy_loss})"
InteractiveMeasurement.__str__ = __str__

def __str__(self):
    fields = f"max_privacy_loss={self.state.max_privacy_loss}" if hasattr(self.state, "max_privacy_loss") else ""
    return f"{self.description}({fields})"
Queryable.__str__ = __str__
################################################################################
# END LOGGING CODE
################################################################################


# Classes for different query types
JoinTree = collections.namedtuple("JoinTree", ["parent", "sibling_index"])
CheckDescendantChange = collections.namedtuple("CheckDescendantChange", ["index", "new_privacy_loss", "pre_invoke"])
GetPrivacyLoss = collections.namedtuple("GetPrivacyLoss", [])


def make_concurrent_filter(max_privacy_loss):

    def function(data):
        # The structure of stuff we store in state
        State = collections.namedtuple("State", ["data", "parent", "sibling_index", "max_privacy_loss", "privacy_loss", "child_privacy_losses"])
        state = State(data=data, parent=None, sibling_index=-1, max_privacy_loss=max_privacy_loss, privacy_loss=0, child_privacy_losses=[])

        def check_new_state(self, child_index, child_privacy_loss, pre_invoke):
            # Append or replace the child privacy loss, then calculate the new sum.
            new_child_privacy_losses = self.state.child_privacy_losses.copy()
            new_child_privacy_losses[child_index:] = [child_privacy_loss]
            new_privacy_loss = sum(new_child_privacy_losses)
            if pre_invoke:
                self._log("CHECK?", child=child_index, child_pl=child_privacy_loss, new_pl=new_privacy_loss, max_pl=self.state.max_privacy_loss)
            # Check against our max privacy loss.
            if new_privacy_loss > state.max_privacy_loss:
                raise Exception(f"New privacy loss {new_privacy_loss} exceeds max privacy loss {max_privacy_loss}")
            # Notify our parent.
            if self.state.parent:
                self.state.parent.query(CheckDescendantChange(index=self.state.sibling_index, new_privacy_loss=new_privacy_loss, pre_invoke=pre_invoke))
            # Don't update state if pre-invoke.
            return self.state if pre_invoke else self.state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses)

        def eval(self, query):
            if isinstance(query, (InteractiveMeasurement, Odometer)):
                new_child_index = len(self.state.child_privacy_losses)
                new_child_privacy_loss = query.privacy_loss if isinstance(query, InteractiveMeasurement) else 0
                _new_state = check_new_state(self, new_child_index, new_child_privacy_loss, True)
                new_child = query.invoke(data)
                new_child.query(JoinTree(parent=self, sibling_index=new_child_index))
                new_state = check_new_state(self, new_child_index, new_child_privacy_loss, False)
                # Convenience to get non-interactive answer
                answer = new_child.query(None) if isinstance(query, Measurement) else new_child
            elif isinstance(query, CheckDescendantChange):
                new_state = check_new_state(self, query.index, query.new_privacy_loss, query.pre_invoke)
                answer = "OK"
            elif isinstance(query, JoinTree):
                new_state = self.state._replace(parent=query.parent, sibling_index=query.sibling_index)
                answer = "OK"
            else:
                raise Exception(f"Unrecognized query {query}")
            return answer, new_state

        return Queryable(state, eval)

    return InteractiveMeasurement(function, max_privacy_loss)


def make_concurrent_odometer():

    def function(data):
        # The structure of stuff we store in state
        State = collections.namedtuple("State", ["data", "parent", "sibling_index", "privacy_loss", "child_privacy_losses"])
        state = State(data=data, parent=None, sibling_index=-1, privacy_loss=0, child_privacy_losses=[])

        def check_new_state(self, child_index, child_privacy_loss, pre_invoke):
            # Append or replace the child privacy loss, then calculate the new sum.
            new_child_privacy_losses = self.state.child_privacy_losses.copy()
            new_child_privacy_losses[child_index:] = [child_privacy_loss]
            new_privacy_loss = sum(new_child_privacy_losses)
            if pre_invoke:
                self._log("CHECK?", child=child_index, child_pl=child_privacy_loss, new_pl=new_privacy_loss)
            # NB: The only difference from make_concurrent_filter is that there's no budget check here.
            # Notify our parent.
            if self.state.parent:
                self.state.parent.query(CheckDescendantChange(index=self.state.sibling_index, new_privacy_loss=new_privacy_loss, pre_invoke=pre_invoke))
            # Don't update state if pre-invoke.
            return self.state if pre_invoke else self.state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses)

        def eval(self, query):
            if isinstance(query, (InteractiveMeasurement, Odometer)):
                new_child_index = len(self.state.child_privacy_losses)
                new_child_privacy_loss = query.privacy_loss if isinstance(query, InteractiveMeasurement) else 0
                _new_state = check_new_state(self, new_child_index, new_child_privacy_loss, True)
                new_child = query.invoke(data)
                new_child.query(JoinTree(parent=self, sibling_index=new_child_index))
                new_state = check_new_state(self, new_child_index, new_child_privacy_loss, False)
                # Convenience to get non-interactive answer
                answer = new_child.query(None) if isinstance(query, Measurement) else new_child
            elif isinstance(query, CheckDescendantChange):
                new_state = check_new_state(self, query.index, query.new_privacy_loss, query.pre_invoke)
                answer = "OK"
            elif isinstance(query, JoinTree):
                new_state = self.state._replace(parent=query.parent, sibling_index=query.sibling_index)
                answer = "OK"
            elif isinstance(query, GetPrivacyLoss):
                new_state = self.state
                answer = self.state.privacy_loss
            else:
                raise Exception(f"Unrecognized query {query}")
            return answer, new_state

        return Queryable(state, eval)

    return Odometer(function)


def make_odomoter_to_filter(odometer, max_privacy_loss):

    def function(data):
        filter = make_concurrent_filter(max_privacy_loss)
        filter_queryable = filter.invoke(data)
        return filter_queryable.query(odometer)

    return InteractiveMeasurement(function, max_privacy_loss)


def make_sequential_filter(max_privacy_loss):

    def function(data):
        # The structure of stuff we store in state
        State = collections.namedtuple("State", ["data", "parent", "sibling_index", "max_privacy_loss", "privacy_loss", "child_privacy_losses", "current_child_index"])
        state = State(data=data, parent=None, sibling_index=-1, max_privacy_loss=max_privacy_loss, privacy_loss=0, child_privacy_losses=[], current_child_index=-1)

        def check_new_state(self, child_index, child_privacy_loss, pre_invoke):
            # Append or replace the child privacy loss, then calculate the new sum.
            new_child_privacy_losses = self.state.child_privacy_losses.copy()
            new_child_privacy_losses[child_index:] = [child_privacy_loss]
            new_privacy_loss = sum(new_child_privacy_losses)
            if pre_invoke:
                self._log("CHECK?", child=child_index, child_pl=child_privacy_loss, new_pl=new_privacy_loss, max_pl=self.state.max_privacy_loss, current_child=self.state.current_child_index)
            # Check against our max privacy loss.
            if new_privacy_loss > state.max_privacy_loss:
                raise Exception(f"New privacy loss {new_privacy_loss} exceeds max privacy loss {max_privacy_loss}")
            # Check sequentiality.
            new_current_child_index = child_index
            if new_current_child_index < self.state.current_child_index:
                raise Exception(f"Non-sequential access of children")
            # Notify our parent.
            if self.state.parent:
                self.state.parent.query(CheckDescendantChange(index=self.state.sibling_index, new_privacy_loss=new_privacy_loss, pre_invoke=pre_invoke))
            # Don't update state if pre-invoke.
            return self.state if pre_invoke else self.state._replace(privacy_loss=new_privacy_loss, child_privacy_losses=new_child_privacy_losses, current_child_index=new_current_child_index)

        def eval(self, query):
            if isinstance(query, (InteractiveMeasurement, Odometer)):
                new_child_index = len(self.state.child_privacy_losses)
                new_child_privacy_loss = query.privacy_loss if isinstance(query, InteractiveMeasurement) else 0
                _new_state = check_new_state(self, new_child_index, new_child_privacy_loss, True)
                new_child = query.invoke(data)
                new_child.query(JoinTree(parent=self, sibling_index=new_child_index))
                new_state = check_new_state(self, new_child_index, new_child_privacy_loss, False)
                # Convenience to get non-interactive answer
                answer = new_child.query(None) if isinstance(query, Measurement) else new_child
            elif isinstance(query, CheckDescendantChange):
                new_state = check_new_state(self, query.index, query.new_privacy_loss, query.pre_invoke)
                answer = "OK"
            elif isinstance(query, JoinTree):
                new_state = self.state._replace(parent=query.parent, sibling_index=query.sibling_index)
                answer = "OK"
            else:
                raise Exception(f"Unrecognized query {query}")
            return answer, new_state

        return Queryable(state, eval)

    return InteractiveMeasurement(function, max_privacy_loss)


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
    root = make_concurrent_filter(budget).invoke(data)
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
    root = make_concurrent_filter(budget).invoke(data)
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
    root = make_concurrent_odometer().invoke(data)
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
    root = make_sequential_filter(budget).invoke(data)
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
    root = make_sequential_filter(budget).invoke(data)
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
