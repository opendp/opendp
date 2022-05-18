class Queryable:

    def __init__(self):
        pass

    def evaluate(self, query):
        return self._eval(query)  # Assumes subclass updates state in _eval().

    def _eval(self, query):
        raise Exception  # FOR SUBCLASSES TO IMPLEMENT


class NestingQueryable(Queryable):

    def __init__(self, data, parent):
        super().__init__()
        self.data = data
        self.parent = parent
        self.index = -1
        self.children = []
        self.privacy_loss = 0

    def _eval(self, query):
        answer = query.evaluate(self.data)
        if isinstance(answer, NestingQueryable):
            answer.parent = self
            answer.index = len(self.children)
            new_children = self.children + [answer]
            if isinstance(query, InteractiveMeasurement) \
               and not self._validate_child_change(new_children, answer.index, query.privacy_loss):
                return "Not allowed"
            else:
                self.children = new_children
        return answer

    def validate_child_change(self, child_index, child_proposed_privacy_loss):
        return self._validate_child_change(self.children, child_index, child_proposed_privacy_loss)

    def _validate_child_change(self, children, child_index, child_proposed_privacy_loss):
        child_privacy_losses = [child.privacy_loss if child.index != child_index else child_proposed_privacy_loss \
                                for child in children]
        proposed_privacy_loss = self._compose_children(child_privacy_losses)
        if self._validate_self_change(child_index, proposed_privacy_loss) \
                and (self.parent == None or self.parent._validate_child_change_explicit_children(self.index, proposed_privacy_loss)):
            self.privacy_loss = proposed_privacy_loss
            return True
        else:
            return False

    def _validate_self_change(self, originating_child_index, proposed_privacy_loss):
        raise Exception  # FOR SUBCLASSES TO IMPLEMENT

    def _compose_children(self, child_privacy_losses):
        raise Exception  # FOR SUBCLASSES TO IMPLEMENT


class OdometerQueryable(NestingQueryable):

    def __init__(self, data, parent):
        super().__init__(data, parent)

    def _validate_self_change(self, child_index, proposed_privacy_loss):
        return True


class FilterQueryable(NestingQueryable):

    def __init__(self, data, parent, budget):
        super().__init__(data, parent)
        self.budget = budget

    def _validate_self_change(self, child_index, proposed_privacy_loss):
        return proposed_privacy_loss <= self.budget


class SequentialQueryable(NestingQueryable):

    def __init__(self, data, parent):
        super().__init__(data, parent)

    def _validate_self_change(self, child_index, proposed_privacy_loss):
        return child_index == len(self.children) - 1




# PARENT    CHILD       QUERY           ACTION
# filter    filter      measurement     child adds and checks loss, child sends zero loss to parent, parent adds and checks loss
# filter    odometer    measurement     child adds loss, child sends total loss to parent, parent adds and checks loss
# odometer  odometer    measurement     child adds loss, child sends total loss to parent, parent adds loss
# odometer  filter      measurement     child adds and checks loss, sends zero loss to parent, parent adds loss

# filter    filter      odometer        no effect?
# filter    odometer    odometer        no effect?
# odometer  odometer    odometer        no effect?
# odometer  filter      odometer        no effect?







class InteractiveMeasurement:
    def __init__(self, function, privacy_loss):
        self.function = function          # fn: InputDomain -> Queryable
        self.privacy_loss = privacy_loss  # Fixed privacy loss
    def invoke(self, data) -> Queryable:    # Convenience method to invoke function
        return self.function(data)


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(data):  # Wrapper function that creates a dummy Queryable
            initial_state = function(data)
            def transition(_self, target_index_path, state, _question):
                return (state, state)
            return Queryable(initial_state, transition)
        super().__init__(interactive_function, privacy_loss)
    def invoke1(self, data):                # Convenience method to invoke function, get result from null query
        queryable = self.invoke(data)
        return queryable.evaluate(None)


class Odometer:
    pass


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
            answer = question.invoke1(data)
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
    print("non-interactive =", measurement.invoke1(data))


# Runs an adaptive composition (Measurement queries)
def test_adaptive():
    print("SIMPLE ADAPTIVE COMPOSITION")
    data = 123.0
    budget = 1.0
    print("make adaptive composition")
    adaptive = make_adaptive_composition(budget)
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
    root_queryable = root_sequential.invoke(data)

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
