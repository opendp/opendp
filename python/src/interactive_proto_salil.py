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
    return Queryable(eval)


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


class Queryable:

    def __init__(self, eval, ok_pre=None, ok_post=None):
        self.parent = None
        self.sibling_index = -1
        self.children = []
        self.eval = eval.__get__(self)
        ok_pre = ok_pre or (lambda s, i: True)
        ok_post = ok_post or (lambda s, i: True)
        self.ok_pre = ok_pre.__get__(self)
        self.ok_post = ok_post.__get__(self)
        self.current_query = None

    def evaluate(self, query):
        self.current_query = query
        if self.ok(-1):
            answer = self.eval(query)
            if isinstance(answer, Queryable):
                answer.parent = self
                answer.sibling_index = len(self.children)
                self.children += [answer]
        else:
            answer = "Sorry Charlie!"
        self.current_query = None
        return answer

    def ok(self, child_index):
        return self.ok_pre(child_index) and (self.parent is None or self.parent.ok(self.sibling_index)) and self.ok_post(child_index)


def make_concurrent_pure_dp_odometer():

    def eval(self, query):
        answer =  query.invoke(self.data)
        if isinstance(query, InteractiveMeasurement):
            answer.privacy_loss = query.privacy_loss
        return answer

    def ok_pre(self, child_index):
        if child_index == -1:
            if isinstance(self.current_query, InteractiveMeasurement):
                self.privacy_loss_predict = self.privacy_loss + self.current_query.privacy_loss
            elif isinstance(self.current_query, Odometer):
                self.privacy_loss_predict = self.privacy_loss
            else:
                self.privacy_loss_predict = 0  # QUESTION: WHAT TO DO HERE?
        else:
            # QUESTION: DOESN'T THIS DEPEND ON CHILDREN HAVING A PRIVACY_LOSS?
            self.privacy_loss_predict = self.children[child_index].privacy_loss_predict \
                                        + [self.children[i].privacy_loss for i in range(len(self.children)) if
                                           i != child_index]
        return True

    def ok_post(self, child_index):
        self.privacy_loss = self.privacy_loss_predict
        self.privacy_loss_predict = 0
        return True

    def function(data):
        queryable = Queryable(eval, ok_pre, ok_post)
        queryable.data = data
        queryable.privacy_loss = 0
        queryable.current_query = None
        queryable.privacy_loss_predict = 0
        return queryable

    return Odometer(function)


def make_sequential_composition_interactive_meas(budget):

    def eval(self, query):
        return query.invoke(self.data)

    def ok_pre(self, child_index):
        if child_index == -1:
            return isinstance(self.current_query, InteractiveMeasurement) and \
                   self.current_query.privacy_loss + self.privacy_loss <= self.budget
        else:
            return child_index == len(self.children) - 1

    def ok_post(self, child_index):
        self.privacy_loss += self.current_query.privacy_loss
        return True

    def function(data):
        queryable = Queryable(eval, ok_pre, ok_post)
        queryable.budget = budget
        queryable.data = data
        queryable.privacy_loss = 0
        queryable.current_query = None
        return queryable

    return InteractiveMeasurement(function, budget)


def make_sequential_composition_odometer(budget):

    def eval(self, query):
        return query.invoke(self.data)

    def ok_pre(self, child_index):
        if child_index == -1:
            self.privacy_loss_predict = self.privacy_loss
            return isinstance(self.current_query, Odometer)
        else:
            if child_index == len(self.children) - 1:
                self.privacy_loss_predict = self.privacy_loss_non_active + self.children[child_index].privacy_loss_predict
                return True
            else:
                return False

    def ok_post(self, child_index):
        self.privacy_loss = self.privacy_loss_predict
        if child_index == -1:
            self.privacy_loss_non_active = self.privacy_loss
        return True

    def function(data):
        queryable = Queryable(eval, ok_pre, ok_post)
        queryable.budget = budget
        queryable.data = data
        queryable.privacy_loss = 0
        queryable.privacy_loss_predict = 0
        queryable.privacy_loss_non_active = 0
        queryable.current_query = None
        return queryable

    return InteractiveMeasurement(function, budget)


def make_filter(budget):  # THIS NEEDS TO BE REWRITTEN!

    def eval(self, query):
        return query.invoke(self.data)

    def ok_pre(self, child_index):
        if child_index == -1:
            if isinstance(self.current_query, InteractiveMeasurement):
                self.privacy_loss_predict = self.privacy_loss + self.current_query.privacy_loss
            elif isinstance(self.current_query, InteractiveMeasurement):
                self.privacy_loss_predict = self.privacy_loss
            else:
                self.privacy_loss_predict = 0
        else:
            self.privacy_loss_predict = self.children[child_index].privacy_loss_predict \
                                        + [self.children[i].privacy_loss for i in range(len(self.children)) if
                                           i != child_index]
        return self.privacy_loss_predict <= self.budget  # QUESTION: IS THIS THE ONLY CHANGE FROM ODOMETER VERSION?

    def ok_post(self, child_index):
        self.privacy_loss = self.privacy_loss_predict
        self.privacy_loss_predict = 0
        return True

    def function(data):
        queryable = Queryable(eval, ok_pre, ok_post)
        queryable.budget = budget
        queryable.data = data
        queryable.privacy_loss = 0
        queryable.current_query = None
        queryable.privacy_loss_predict = 0
        return queryable

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
