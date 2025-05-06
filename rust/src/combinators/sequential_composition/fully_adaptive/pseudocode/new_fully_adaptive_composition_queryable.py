# type: ignore
def new_fully_adaptive_composition_queryable(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
    data: DI_Carrier,
) -> OdometerQueryable[Measurement[DI, TO, MI, MO], TO, MO_Distance]:

    require_sequentiality = matches(
        output_measure.composability(Adaptivity.FullyAdaptive)?,
        Composability.Sequential
    )

    privacy_maps = []  # Vec<MO_Distance> `\label{mutable-state}`

    def transition(  # `\label{transition}`
        self_: OdometerQueryable[Measurement[DI, TO, MI, MO], TO, MO_Distance],
        query: Query[OdometerQuery[Measurement[DI, TO, MI, MO]]],
    ):
        # this queryable and wrapped children communicate via an AskPermission query
        # defined here, where no-one else can access the type
        @dataclass
        class AskPermission:
            id: usize

        match query:
            # evaluate external invoke query
            case Query.External(
                OdometerQuery.Invoke(meas)
            ):  # `\label{query-wrapper-def}`
                assert_components_match(  # `\label{domain-check}`
                    DomainMismatch, input_domain, meas.input_domain
                )

                assert_components_match(  # `\label{metric-check}`
                    MetricMismatch, input_metric, meas.input_metric
                )

                assert_components_match(  # `\label{measure-check}`
                    MeasureMismatch, output_measure, meas.output_measure
                )

                let enforce_sequentiality = Rc::new(RefCell::new(false));

                if require_sequentiality:
                    # when the output measure doesn't allow concurrent composition,
                    # wrap any interactive queryables spawned.
                    # This way, when the child gets a query it sends an AskPermission query
                    # to this parent queryable, giving this sequential odometer queryable
                    # a chance to deny the child permission to execute
                    child_id = privacy_maps.len()

                    def callback():
                        if enforce_sequentiality.borrow():
                            self_.eval_internal(AskPermission(child_id))
                        else:
                            return ()

                    seq_wrapper = Wrapper.new_recursive_pre_hook(  # `\label{pre-hook}`
                        callback
                    )
                else:
                    seq_wrapper = None

                answer = meas.invoke_wrap(data, seq_wrapper)  # `\label{invoke}`

                enforce_sequentiality.borrow_mut() = True
                
                # We've now increased our privacy spend.
                # This is our only state modification
                privacy_maps.push(meas.privacy_map)  # `\label{child-privacy-map}`

                return Answer.External(OdometerAnswer.Invoke(answer))

            # evaluate external privacy loss query
            case Query.External(OdometerQuery.PrivacyLoss()):
                d_mids = [m.eval(d_in) for m in privacy_maps]
                d_out = output_measure.compose(d_mids)
                return Answer.External(OdometerAnswer.Map(d_out))

            case Query.Internal(query):
                # Check if the query is from a child queryable
                #     who is asking for permission to execute
                if isinstance(query, AskPermission):  # `\label{ask-permission-handler}`
                    # deny permission if the sequential odometer has moved on
                    if query.id + 1 != privacy_maps.len():
                        raise ValueError("sequential odometer has received a new query")

                    # otherwise, return Ok to approve the change
                    return Answer.internal(())

                raise ValueError("query not recognized")

    return Queryable.new(transition)  # `\label{queryable}`
