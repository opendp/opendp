class Queryable:
    def __init__(self, parameters, state, transition):
        self.parameters = parameters
        self.state = state
        self.transition = transition
    def eval(self, query):
        new_state, answer = self.transition(self.parameters, self.state, query)
        self.state = new_state
        return answer


class InteractiveMeasurement:
    def __init__(self, function, privacy_loss):
        self.function = function
        self.privacy_loss = privacy_loss
    def calc(self, data) -> Queryable:
        return self.function(data)

class Measurement(InteractiveMeasurement):
    def __init__(self, function, privacy_loss):
        def non_interactive_function(data):
            parameters = function(data)
            def transition(parameters, _state, _query):
                return parameters
            return Queryable(parameters, None, transition)
        super().__init__(non_interactive_function, privacy_loss)
    def calc1(self, data):
        queryable = self.calc(data)
        return queryable.eval(None)


def make_adaptive_composition(budget):
    def function(data):
        parameters = data
        initial_state = budget

        def transition(parameters, state, query: Measurement):
            new_state = state - query.privacy_loss
            if new_state < 0.0:
                raise Exception("Insufficient budget")
            answer = query.calc(parameters)
            return (new_state, answer)

        return Queryable(parameters, initial_state, transition)

    return InteractiveMeasurement(function, budget)


def make_sequential_composition(budget):
    def function(data):
        parameters = data
        initial_state = budget

        def wrapping_transition(parameters, state, query: InteractiveMeasurement):


        def transition(parameters, state, query: InteractiveMeasurement):
            new_state = state - query.privacy_loss
            if new_state < 0.0:
                raise Exception("Insufficient budget")
            answer = query.calc(parameters)
            return (new_state, answer)

        return Queryable(parameters, initial_state, transition)

    return InteractiveMeasurement(function, budget)



