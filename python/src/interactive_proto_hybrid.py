class Queryable:

    def __init__(self, _eval):
        self._eval = _eval.__get__(self)

    def evaluate(self, query):
        return self._eval(query)  # Assumes that _eval() updates self.state.


class NestingQueryable(Queryable):

    def __init__(self, data, privacy_loss, _compose_children, _validate_self_change):

        def _eval(self, query):
            answer = query.evaluate(self.data)
            if isinstance(answer, NestingQueryable):
                new_children = self.children + [answer]
                if isinstance(query, InteractiveMeasurement):
                    if not self._validate_child_change_explicit_children(answer.index, query.privacy_loss):
                        return "Not allowed"
                self.children = new_children
            return answer

        super().__init__(_eval)
        self.data = data
        self.parent = None
        self.index = -1
        self.children = []
        self.privacy_loss = privacy_loss
        self._compose_children = _compose_children.__get__(self)
        self._validate_self_change = _validate_self_change.__get__(self)

    def _graft(self, parent, index):
        self.parent = parent
        self.index = index

    def _validate_child_change(self, child_index, child_proposed_privacy_loss):
        return self._validate_child_change_explicit_children(self.children, child_index, child_proposed_privacy_loss)

    def _validate_child_change_explicit_children(self, children, child_index, child_proposed_privacy_loss):
        child_privacy_losses = [child.privacy_loss if child.index != child_index else child_proposed_privacy_loss
                                for child in children]
        proposed_privacy_loss = self._compose_children(child_privacy_losses)
        if self._validate_self_change(child_index, proposed_privacy_loss) \
                and (self.parent == None or self.parent._validate_child_change(self.index, proposed_privacy_loss)):
            self.privacy_loss = proposed_privacy_loss
            return True
        else:
            return False


class Invokable:  # Common interface for InteractiveMeasurement & Odometer

    def __init__(self, function):
        self.function = function

    def invoke(self, data):  # Convenience method to invoke function
        return self.function(data)


class InteractiveMeasurement(Invokable):

    def __init__(self, function, privacy_loss):
        super().__init__(function)
        self.privacy_loss = privacy_loss  # Fixed privacy loss


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(data):  # Wrapper function that creates a dummy Queryable
            answer = function(data)  # Invoke function once, store result as state
            def _eval(self, _question):
                return answer
            return Queryable(_eval)
        super().__init__(interactive_function, privacy_loss)
    def invoke1(self, data):                # Convenience method to invoke function, get result from null query
        queryable = self.invoke(data)
        return queryable.evaluate(None)


class Odometer(Invokable):
    pass


def make_filter(budget):
    def function(data):
        def _validate_self_change(self, _originating_child_index, proposed_privacy_loss):
            return proposed_privacy_loss <= budget
        def _compose_children(self, child_privacy_losses):
            return sum(child_privacy_losses)
        return NestingQueryable(data, budget, _validate_self_change, _compose_children)
    return InteractiveMeasurement(function, budget)

def make_odometer():
    def function(data):
        def _validate_self_change(self, _originating_child_index, _proposed_privacy_loss):
            return True
        def _compose_children(self, child_privacy_losses):
            return sum(child_privacy_losses)
        return NestingQueryable(data, 0, _validate_self_change, _compose_children)
    return Odometer(function)

def make_sequential_composition(budget):
    def function(data):
        def _validate_self_change(self, _originating_child_index, proposed_privacy_loss):
            return True
        def _compose_children(self, child_privacy_losses):
            return sum(child_privacy_losses)
        return NestingQueryable(data, budget, _validate_self_change, _compose_children)
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
    print("non-interactive =", measurement.invoke1(data))


# Runs an adaptive composition (Measurement queries)
def test_adaptive():
    print("SIMPLE ADAPTIVE COMPOSITION")
    data = 123.0
    budget = 1.0
    print("make adaptive composition")
    adaptive = make_filter(budget)
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
    sequential = make_sequential_composition(budget)
    print("    spawn root queryable")
    root_queryable = sequential.invoke(data)

    print("        make adaptive composition (for sub-queryable 1 of sequential)")
    adaptive1 = make_filter(budget / 2)
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
