# type: ignore
def new_fully_adaptive_composition_queryable(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    data: DI_Carrier,
    sequential: bool
) -> OdometerQueryable[Measurement[DI, TO, MI, MO], TO, MO_Distance]:
    d_mids = []  # Vec<MO_Distance> `\label{mutable-state}`
    
    def transition(  # `\label{transition}`
        self_: OdometerQueryable[Measurement[DI, TO, MI, MO], TO, MO_Distance],
        query: Query[OdometerQuery[Measurement[DI, TO, MI, MO]]]
     ):
        # this queryable and wrapped children communicate via an AskPermission query
        # defined here, where no-one else can access the type
        @dataclass
        class AskPermission:
            id: usize

        match query:
            # evaluate external invoke query
            case Query.External(OdometerQuery.Invoke(measurement)):  # `\label{query-wrapper-def}`
                assert_components_match(  # `\label{domain-check}`
                    DomainMismatch,
                    input_domain,
                    measurement.input_domain
                )

                assert_components_match(  # `\label{metric-check}`
                    MetricMismatch,
                    input_metric,
                    measurement.input_metric
                )

                assert_components_match(  # `\label{measure-check}`
                    MeasureMismatch,
                    output_measure,
                    measurement.output_measure
                )

                if sequential:
                    # when the output measure doesn't allow concurrent composition,
                    # wrap any interactive queryables spawned.
                    # This way, when the child gets a query it sends an AskPermission query to this parent queryable,
                    # giving this sequential odometer queryable
                    # a chance to deny the child permission to execute
                    child_id = d_mids.len()

                    seq_wrapper = Wrapper.new_recursive_pre_hook(  # `\label{pre-hook}`
                        lambda: self_.eval_internal(AskPermission(child_id))
                    )
                else:
                    seq_wrapper = None

                answer = measurement.invoke_wrap(data, seq_wrapper)  # `\label{invoke}`

                # we've now increased our privacy spend. This is our only state modification
                d_mids.push(measurement.privacy_map)  # `\label{child-privacy-map}`

                return Answer.External(OdometerAnswer.Invoke(answer))
            
            # evaluate external map query
            case Query.External(OdometerQuery.PrivacyLoss()):
                d_out = output_measure.compose(d_mids)
                return Answer.External(OdometerAnswer.Map(d_out))
            
            case Query.Internal(query):
                # check if the query is from a child queryable who is asking for permission to execute
                if isinstance(query, AskPermission):  # `\label{ask-permission-handler}`
                    # deny permission if the sequential odometer has moved on
                    if query.id + 1 != d_mids.len():
                        raise ValueError("sequential odometer has received a new query")
                    
                    # otherwise, return Ok to approve the change
                    return Answer.internal(())
                

                raise ValueError("query not recognized")

    return Queryable.new(transition)  # `\label{queryable}`
