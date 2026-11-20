use super::*;
use rmpv::Value;

fn string(value: &str) -> Value {
    Value::String(value.into())
}

fn tagged(name: &str, payload: Value) -> Value {
    Value::Map(vec![(string(name), payload)])
}

fn encoded(body: &Value) -> Vec<u8> {
    let mut header = Vec::new();
    DslPlan::default()
        .serialize_versioned(&mut header, Default::default())
        .unwrap();
    header.truncate(HEADER_LENGTH);
    rmpv::encode::write_value(&mut header, body).unwrap();
    header
}

fn root_with(nodes: Value) -> Value {
    Value::Map(vec![
        (string("root"), Value::Nil),
        (string("dataframes"), Value::Nil),
        (string("dsl_plans"), nodes),
    ])
}

fn slot(plan: Value) -> Value {
    Value::Map(vec![
        (string("value"), plan),
        (string("version"), Value::from(1)),
    ])
}

fn serialized_plan(plan: Value) -> Vec<u8> {
    encoded(&root_with(Value::Array(vec![slot(plan)])))
}

fn serialized_expr(expr: Value) -> Vec<u8> {
    serialized_plan(tagged(
        "Select",
        Value::Map(vec![(string("expr"), Value::Array(vec![expr]))]),
    ))
}

fn preflight(raw: &[u8]) -> Fallible<Vec<u8>> {
    sanitize_serialized_dslplan(raw, "/trusted/libopendp.so", false)
}

#[test]
fn checks_unreachable_plan_nodes() {
    use std::sync::Arc;

    let wrapped = DslPlan::IR {
        dsl: Arc::new(DslPlan::default()),
        version: 0,
        node: None,
    };
    let mut serialized = Vec::new();
    wrapped
        .serialize_versioned(&mut serialized, Default::default())
        .unwrap();
    preflight(&serialized).unwrap();

    let mut payload =
        rmpv::decode::read_value(&mut std::io::Cursor::new(&serialized[HEADER_LENGTH..])).unwrap();
    let Value::Map(root) = &mut payload else {
        panic!("expected serialized plan map");
    };
    let Value::Array(slots) = &mut root
        .iter_mut()
        .find(|(key, _)| key.as_str() == Some("dsl_plans"))
        .unwrap()
        .1
    else {
        panic!("expected serialized plan slots");
    };
    slots.push(slot(tagged("PythonScan", Value::Nil)));
    assert!(preflight(&encoded(&payload)).is_err());
}

#[test]
fn rejects_unsupported_operations() {
    for name in ["Cache", "Scan", "Sink", "PythonScan", "FuturePlanNode"] {
        let error = preflight(&serialized_plan(tagged(name, Value::Nil))).unwrap_err();
        assert!(
            format!("{error:?}").contains("unsupported plan operation"),
            "{name}"
        );
    }

    for name in ["AnonymousFunction", "SubPlan", "Slice"] {
        let error = preflight(&serialized_expr(tagged(name, Value::Nil))).unwrap_err();
        assert!(
            format!("{error:?}").contains("unsupported expression operation"),
            "{name}"
        );
    }

    let function = tagged(
        "Function",
        Value::Map(vec![
            (string("input"), Value::Array(vec![])),
            (string("function"), string("Shift")),
        ]),
    );
    let error = preflight(&serialized_expr(function)).unwrap_err();
    assert!(format!("{error:?}").contains("unsupported function operation"));
}

#[test]
fn rejects_groupby_callbacks() {
    let groupby = tagged(
        "GroupBy",
        Value::Map(vec![(string("apply"), Value::Boolean(true))]),
    );
    let error = preflight(&serialized_plan(groupby)).unwrap_err();
    assert!(format!("{error:?}").contains("group-by callbacks"));
}

#[test]
fn rejects_callbacks_embedded_in_dtype_expressions() {
    let cast = tagged(
        "Cast",
        Value::Map(vec![
            (string("expr"), tagged("Column", string("value"))),
            (
                string("dtype"),
                tagged("OfExpr", tagged("AnonymousFunction", Value::Nil)),
            ),
        ]),
    );
    let error = preflight(&serialized_expr(cast)).unwrap_err();
    assert!(format!("{error:?}").contains("unsupported expression operation: AnonymousFunction"));
}

#[test]
fn rejects_malformed_messagepack() {
    let duplicate = tagged(
        "Literal",
        Value::Map(vec![
            (string("duplicate"), Value::Nil),
            (string("duplicate"), Value::Nil),
        ]),
    );
    let error = preflight(&serialized_expr(duplicate)).unwrap_err();
    assert!(format!("{error:?}").contains("duplicate MessagePack map field"));

    let extension = tagged(
        "Literal",
        Value::Map(vec![(string("payload"), Value::Ext(1, vec![1, 2, 3]))]),
    );
    assert!(preflight(&serialized_expr(extension)).is_err());

    let root = Value::Map(vec![
        (string("root"), Value::Nil),
        (string("root"), Value::Nil),
        (string("dataframes"), Value::Nil),
        (string("dsl_plans"), Value::Nil),
    ]);
    assert!(preflight(&encoded(&root)).is_err());
    assert!(preflight(&encoded(&Value::Nil)).is_err());
}

#[test]
fn checks_header_and_trailing_bytes() {
    let mut bytes = Vec::new();
    DslPlan::default()
        .serialize_versioned(&mut bytes, Default::default())
        .unwrap();
    preflight(&bytes).unwrap();

    for raw in [&b""[..], &b"not a Polars plan"[..], &b"DSL_VERSION"[..]] {
        assert!(preflight(raw).is_err());
    }

    let mut bad_version = bytes.clone();
    bad_version[DSL_MAGIC.len()] ^= 1;
    let version_error = preflight(&bad_version).unwrap_err();
    assert!(format!("{version_error:?}").contains("Polars versions"));

    let mut bad_schema = bytes.clone();
    bad_schema[DSL_MAGIC.len() + 4] ^= 1;
    assert!(format!("{:?}", preflight(&bad_schema).unwrap_err()).contains("serialization schema"));

    let mut trailing = bytes;
    trailing.push(0);
    assert!(preflight(&trailing).is_err());
}
