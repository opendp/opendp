class Queryable:
    def __init__(self, context, initial_state, transition):
        self.context = context
        self.state = initial_state
        self.transition = transition
    def query(self, question):
        new_state, answer = self.transition(self.context, self.state, question)
        self.state = new_state
        return answer


class InteractiveMeasurement:
    def __init__(self, function, privacy_loss):
        self.function = function
        self.privacy_loss = privacy_loss
    def eval(self, data) -> Queryable:
        return self.function(data)


class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def interactive_function(data):
            context = function(data)
            def transition(context, _state, _question):
                return None, context
            return Queryable(context, None, transition)
        super().__init__(interactive_function, privacy_loss)
    def eval1(self, data):
        queryable = self.eval(data)
        return queryable.query(None)


def make_adaptive_composition(budget):
    def function(data):
        context = data
        initial_state = budget

        def transition(context, state, question: Measurement):
            new_state = state - question.privacy_loss
            if new_state < 0.0:
                raise Exception("Insufficient budget")
            answer = question.eval1(context)
            return (new_state, answer)

        return Queryable(context, initial_state, transition)

    return InteractiveMeasurement(function, budget)


def make_sequential_composition(budget):
    def make_manager(budget):
        context = None
        initial_state = budget, []
        def transition(_context, state, question):
            budget, children = state
            data, index, original_question = question
            if index is None:
                # Spawn of child
                if original_question.privacy_loss > budget:
                    raise Exception("Insufficient budget")
                budget -= original_question.privacy_loss
                new_index = len(children)
                new_child = question.eval(data)
                children.append(new_child)
                answer = new_index
            else:
                # Query to child
                if index != len(children) - 1:
                    raise Exception("Non-sequential query")
                child = children[index]
                answer = child.question(original_question)
            new_state = budget, children
            return new_state, answer
        return Queryable(context, initial_state, transition)
    manager = make_manager(budget)

    def function(data):
        context = manager, data
        initial_state = None
        def transition(context, _state, question: InteractiveMeasurement):
            manager, data = context
            index = None
            question = data, index, question
            child_index = manager.query(question)

            child_context = manager, data, child_index
            child_initial_state = None
            def child_transition(context, state, question):
                manager, data, index = context
                question = data, index, question
                answer = manager.query(question)
                return state, answer
            return Queryable(child_context, child_initial_state, child_transition)

        return Queryable(context, initial_state, transition)
    return InteractiveMeasurement(function, budget)


def make_base_laplace(sigma):
    def laplace(sigma):
        import random, math
        u = random.uniform(-0.5, 0.5)
        return math.copysign(1, u) * math.log(1.0 - 2.0 * abs(u)) * sigma
    return Measurement(lambda x: x + laplace(sigma), 1.0 / sigma)


def main():
    base_laplace = make_base_laplace(1)
    print(f"non-interactive = {base_laplace.eval1(1)}")

    adaptive = make_adaptive_composition(2.0)
    queryable = adaptive.eval(2)
    print(f"adaptive1 = {queryable.query(base_laplace)}")
    print(f"adaptive2 = {queryable.query(base_laplace)}")
    print(f"adaptive3 = {queryable.query(base_laplace)}")


# MAKE INTERACTIVE MEASUREMENT 1
# im_1 = make_sequential()
# MAKE INTERACTIVE MEASUREMENT 1 QUERYABLE 1
# im_1__queryable_1 = im_1.function(data)
# MAKE INTERACTIVE MEASUREMENT 1 QUERYABLE 1 INTERACTIVE MEASUREMENT 1
# im_1_queryable_1_im_1 = make_adaptive()
# MAKE INTERACTIVE MEASUREMENT 1 QUERYABLE 1 INTERACTIVE MEASUREMENT 1 QUERYABLE 1
# im_1_queryable_1_im_1_queryable_1 = im_1_queryable_1.query(im_1_queryable_1_im_1)
# MAKE INTERACTIVE MEASUREMENT 1 QUERYABLE 1 INTERACTIVE MEASUREMENT 1 QUERYABLE 1 QUERY 1
# question = make_sum()
# ans = im_1_queryable_1_im_1_queryable_1.eval(question)
# queryable_1_1_1 = queryable.eval(im_1_1)
# answer


if __name__ == '__main__':
    main()
