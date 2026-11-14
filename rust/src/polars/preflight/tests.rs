use super::*;
use rmpv::Value;

// The sanitizer is the only entry point for untrusted serialized plans.
fn preflight(raw: &[u8]) -> Fallible<Vec<u8>> {
    sanitize_serialized_dslplan(raw, "/trusted/libopendp.so", &[])
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

fn root_with(value: Value) -> Value {
    let string = |s: &str| Value::String(s.into());
    Value::Map(vec![
        (string("root"), Value::Nil),
        (string("dataframes"), Value::Nil),
        (string("dsl_plans"), value),
    ])
}

fn slot(variant: &str) -> Value {
    Value::Map(vec![
        (
            Value::String("value".into()),
            Value::Map(vec![(Value::String(variant.into()), Value::Nil)]),
        ),
        (Value::String("version".into()), Value::from(1)),
    ])
}

#[test]
fn inspects_every_node_behind_ir_wrapper() {
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

    // Polars deserializes every slot, not just nodes reachable from the
    // root. An IR wrapper must not conceal an eagerly-unpickled slot.
    let mut payload =
        rmpv::decode::read_value(&mut std::io::Cursor::new(&serialized[HEADER_LENGTH..])).unwrap();
    let Value::Map(root) = &mut payload else {
        panic!("expected struct map");
    };
    let Value::Array(slots) = &mut root
        .iter_mut()
        .find(|(name, _)| name.as_str() == Some("dsl_plans"))
        .unwrap()
        .1
    else {
        panic!("expected node slot map");
    };
    slots.push(slot("PythonScan"));
    assert!(preflight(&encoded(&payload)).is_err());
}

#[test]
fn accepts_native_versioned_plan() {
    let mut bytes = Vec::new();
    DslPlan::default()
        .serialize_versioned(&mut bytes, Default::default())
        .unwrap();
    preflight(&bytes).unwrap();
}

#[test]
fn refuses_python_and_callback_variants_before_polars_deserialization() {
    for name in ["PythonScan", "PipeWithSchema", "Scan", "Sink"] {
        let bad = root_with(Value::Array(vec![slot(name)]));
        assert!(
            preflight(&encoded(&bad)).is_err(),
            "{name}"
        );
    }
    for name in FORBIDDEN_VARIANTS {
        let callback = Value::Map(vec![(Value::String((*name).into()), Value::Nil)]);
        let select = Value::Map(vec![(
            Value::String("Select".into()),
            Value::Map(vec![(
                Value::String("expr".into()),
                Value::Array(vec![callback]),
            )]),
        )]);
        let bad = root_with(Value::Array(vec![Value::Map(vec![
            (Value::String("value".into()), select),
            (Value::String("version".into()), Value::from(1)),
        ])]));
        let error = preflight(&encoded(&bad)).unwrap_err();
        assert!(
            format!("{error:?}").contains("forbidden construct"),
            "{name}"
        );
    }
    let bad = root_with(Value::Array(vec![Value::Map(vec![
        (
            Value::String("value".into()),
            Value::Map(vec![(
                Value::String("GroupBy".into()),
                Value::Map(vec![(Value::String("apply".into()), Value::Bool(true))]),
            )]),
        ),
        (Value::String("version".into()), Value::from(1)),
    ])]));
    let error = preflight(&encoded(&bad)).unwrap_err();
    assert!(format!("{error:?}").contains("group-by callbacks"));
}

fn serialized_scan(path: &str, scan_type: &str) -> Vec<u8> {
    let string = |value: &str| Value::String(value.into());
    let node = Value::Map(vec![(
        string("Scan"),
        Value::Map(vec![
            (
                string("sources"),
                Value::Map(vec![(
                    string("Paths"),
                    Value::Array(vec![Value::Map(vec![(string("Local"), string(path))])]),
                )]),
            ),
            (
                string("unified_scan_args"),
                Value::Map(vec![
                    (string("cloud_options"), Value::Nil),
                    (string("deletion_files"), Value::Nil),
                ]),
            ),
            (
                string("scan_type"),
                Value::Map(vec![(string(scan_type), Value::Map(vec![]))]),
            ),
        ]),
    )]);
    encoded(&root_with(Value::Array(vec![Value::Map(vec![
        (string("value"), node),
        (string("version"), Value::from(1)),
    ])])))
}

#[test]
fn only_approved_local_scans_can_be_deserialized() {
    let path = "/public/keys.parquet".to_string();
    let bytes = serialized_scan(&path, "Parquet");
    assert!(preflight(&bytes).is_err());
    sanitize_serialized_dslplan(&bytes, "opendp", &[path.clone()]).unwrap();

    let other = vec!["/public/other.parquet".to_string()];
    assert!(sanitize_serialized_dslplan(&bytes, "opendp", &other).is_err());

    let alias = serialized_scan("/public/../public/keys.parquet", "Parquet");
    assert!(sanitize_serialized_dslplan(&alias, "opendp", &[path.clone()]).is_err());

    let csv = serialized_scan(&path, "Csv");
    sanitize_serialized_dslplan(&csv, "opendp", &[path.clone()]).unwrap();

    let python = serialized_scan(&path, "PythonDataset");
    assert!(sanitize_serialized_dslplan(&python, "opendp", &[path]).is_err());
}

#[test]
fn checks_dangerous_tags_under_unknown_nested_variants() {
    let string = |s: &str| Value::String(s.into());
    let nested = |child: Value| {
        let unknown = Value::Map(vec![(
            string("FuturePlanNode"),
            Value::Map(vec![(
                string("arbitrary_children"),
                Value::Array(vec![Value::Map(vec![(string("nested"), child)])]),
            )]),
        )]);
        encoded(&root_with(Value::Array(vec![Value::Map(vec![
            (string("value"), unknown),
            (string("version"), Value::from(1)),
        ])])))
    };

    // Unknown structure does not stop traversal, even if it has no known
    // input/children fields.
    let harmless = nested(Value::Map(vec![(string("Filter"), Value::Nil)]));
    preflight(&harmless).unwrap();

    for tag in ["PythonScan", "OpaquePython", "Sink"] {
        let dangerous = nested(Value::Map(vec![(string(tag), Value::Nil)]));
        assert!(preflight(&dangerous).is_err(), "{tag}");
    }

    let group_by = nested(Value::Map(vec![(
        string("GroupBy"),
        Value::Map(vec![(string("apply"), Value::Bool(true))]),
    )]));
    assert!(format!("{:?}", preflight(&group_by).unwrap_err())
        .contains("group-by callbacks"));

    let scan = nested(Value::Map(vec![(
        string("Scan"),
        Value::Map(vec![
            (
                string("sources"),
                Value::Map(vec![(
                    string("Paths"),
                    Value::Array(vec![Value::Map(vec![(
                        string("Local"),
                        string("/private/secret.csv"),
                    )])]),
                )]),
            ),
            (string("unified_scan_args"), Value::Map(vec![])),
            (
                string("scan_type"),
                Value::Map(vec![(string("Csv"), Value::Map(vec![]))]),
            ),
        ]),
    )]));
    assert!(format!("{:?}", preflight(&scan).unwrap_err())
        .contains("file scans are not allowed"));
}

fn plugin_plan(symbol: &str, kwargs: Value, flags: Value) -> Vec<u8> {
    let string = |s: &str| Value::String(s.into());
    let plugin = Value::Map(vec![
        (string("lib"), string("/untrusted/libopendp.so")),
        (string("symbol"), string(symbol)),
        (string("kwargs"), kwargs),
        (string("flags"), flags),
    ]);
    let expr = Value::Map(vec![(string("FfiPlugin"), plugin)]);
    let node = Value::Map(vec![(
        string("Select"),
        Value::Map(vec![(string("expr"), Value::Array(vec![expr]))]),
    )]);
    encoded(&root_with(Value::Array(vec![Value::Map(vec![
        (string("value"), node),
        (string("version"), Value::from(1)),
    ])])))
}

#[test]
fn validates_and_rebinds_native_plugin_symbols() {
    let raw = plugin_plan("noise", Value::Binary(vec![]), Value::Bool(false));
    preflight(&raw).unwrap();

    let trusted_lib = "/server/libopendp.so";
    let rewritten = sanitize_serialized_dslplan(&raw, trusted_lib, &[]).unwrap();
    assert!(
        !rewritten
            .windows(b"/untrusted/libopendp.so".len())
            .any(|chunk| chunk == b"/untrusted/libopendp.so")
    );
    assert!(
        rewritten
            .windows(trusted_lib.len())
            .any(|chunk| chunk == trusted_lib.as_bytes())
    );

    let unknown = plugin_plan("unknown_symbol", Value::Binary(vec![]), Value::Nil);
    assert!(preflight(&unknown).is_err());

    let pickled = plugin_plan("noise", Value::Binary(vec![1]), Value::Nil);
    assert!(preflight(&pickled).is_err());

    // Client-supplied plugin flags are validated later when OpenDP builds
    // the executable measurement. Preflight only restricts symbol and kwargs.
    let unusual_flags = plugin_plan("noise", Value::Array(vec![]), Value::String("flags".into()));
    preflight(&unusual_flags).unwrap();
}

#[test]
fn rejects_malformed_plan_buffers() {
    for raw in [&b""[..], &b"not a Polars plan"[..], &b"DSL_VERSION"[..]] {
        assert!(preflight(raw).is_err());
    }
}

#[test]
fn refuses_bad_header_duplicates_trailing_bytes_and_unknown_structure() {
    let mut bytes = Vec::new();
    DslPlan::default()
        .serialize_versioned(&mut bytes, Default::default())
        .unwrap();
    let mut bad_header = bytes.clone();
    bad_header[DSL_MAGIC.len()] ^= 1;
    let version_error = preflight(&bad_header).unwrap_err();
    let version_message = format!("{version_error:?}");
    assert!(version_message.contains("client"));
    assert!(version_message.contains("server"));
    assert!(version_message.contains("Polars versions"));

    let mut bad_schema = bytes.clone();
    bad_schema[DSL_MAGIC.len() + 4] ^= 1;
    let schema_error = preflight(&bad_schema).unwrap_err();
    assert!(format!("{schema_error:?}").contains("serialization schema"));

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(preflight(&trailing).is_err());

    let duplicate = Value::Map(vec![
        (Value::String("root".into()), Value::Nil),
        (Value::String("root".into()), Value::Nil),
        (Value::String("dataframes".into()), Value::Nil),
        (Value::String("dsl_plans".into()), Value::Nil),
    ]);
    assert!(preflight(&encoded(&duplicate)).is_err());

    assert!(preflight(&encoded(&Value::Nil)).is_err());
}
