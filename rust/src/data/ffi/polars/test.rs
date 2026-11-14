use super::*;
use crate::{
    error::ExplainUnwrap,
    ffi::util,
    measurements::expr_noise::{NoiseDistribution, Support},
};
use serde_pickle::{HashableValue, Value};

fn parameters() -> NoisePlugin {
    NoisePlugin {
        distribution: NoiseDistribution::Laplace,
        scale: 2.5,
        support: Support::Integer,
    }
}

fn encode(parameters: &NoisePlugin) -> Vec<u8> {
    serde_pickle::to_vec(parameters, Default::default()).unwrap_test()
}

#[test]
fn test_decode_noise_plugin_parameters_roundtrip() -> Fallible<()> {
    for distribution in [NoiseDistribution::Laplace, NoiseDistribution::Gaussian] {
        for support in [Support::Integer, Support::Float] {
            for scale in [0.0, 1.25, f64::MIN_POSITIVE, f64::MAX] {
                let parameters = NoisePlugin {
                    distribution,
                    scale,
                    support,
                };
                let bytes = encode(&parameters);
                let kwargs = FfiSlice {
                    ptr: bytes.as_ptr().cast(),
                    len: bytes.len(),
                };
                let result = opendp_data__decode_noise_plugin_parameters(&kwargs);
                let FfiResult::Ok(result) = result else {
                    panic!("valid noise plugin parameters were rejected");
                };
                let json = util::into_string(result)?;
                let decoded: NoisePlugin = serde_json::from_str(&json).unwrap_test();
                assert_eq!(decoded, parameters);
            }
        }
    }
    Ok(())
}

#[test]
fn test_decode_noise_plugin_parameters_python_pickle() -> Fallible<()> {
    // A Python protocol-0 dictionary, containing only strings and a float.
    let kwargs = b"(dp0\nVdistribution\np1\nVLaplace\np2\nsVscale\np3\nF2.5\nsVsupport\np4\nVInteger\np5\ns.";
    let json = decode_noise_plugin_parameters(kwargs)?;
    let decoded: NoisePlugin = serde_json::from_str(&json).unwrap_test();
    assert_eq!(decoded, parameters());
    Ok(())
}

#[test]
fn test_decode_noise_plugin_parameters_malformed() {
    let kwargs = encode(&parameters());
    let mut trailing = kwargs.clone();
    trailing.extend_from_slice(b"trailing data");
    for payload in [
        &[][..],
        b"not a pickle",
        &kwargs[..kwargs.len() - 1],
        &trailing,
        // A benign reference to a Python class is not a NoisePlugin payload.
        b"cbuiltins\nobject\n.",
    ] {
        assert!(decode_noise_plugin_parameters(payload).is_err());
    }
}

#[test]
fn test_decode_noise_plugin_parameters_schema() {
    let Value::Dict(fields) = serde_pickle::to_value(&parameters()).unwrap_test() else {
        panic!("NoisePlugin must serialize as a dictionary");
    };

    for name in ["distribution", "scale", "support"] {
        let mut missing = fields.clone();
        missing.remove(&HashableValue::String(name.to_string()));
        let payload =
            serde_pickle::value_to_vec(&Value::Dict(missing), Default::default()).unwrap_test();
        assert!(decode_noise_plugin_parameters(&payload).is_err());
    }

    for (name, value) in [
        ("distribution", Value::String("Unknown".to_string())),
        ("support", Value::String("Unknown".to_string())),
        ("scale", Value::String("2.5".to_string())),
        ("scale", Value::None),
        ("scale", Value::Bool(true)),
    ] {
        let mut invalid = fields.clone();
        invalid.insert(HashableValue::String(name.to_string()), value);
        let payload =
            serde_pickle::value_to_vec(&Value::Dict(invalid), Default::default()).unwrap_test();
        assert!(decode_noise_plugin_parameters(&payload).is_err());
    }
}

#[test]
fn test_decode_noise_plugin_parameters_preserves_unknown_fields() -> Fallible<()> {
    let Value::Dict(mut fields) = serde_pickle::to_value(&parameters()).unwrap_test() else {
        panic!("NoisePlugin must serialize as a dictionary");
    };
    // Preserve NoisePlugin's existing serde behavior: unknown fields are ignored.
    fields.insert(HashableValue::String("extra".to_string()), Value::I64(1));
    let payload =
        serde_pickle::value_to_vec(&Value::Dict(fields), Default::default()).unwrap_test();
    let json = decode_noise_plugin_parameters(&payload)?;
    let decoded: NoisePlugin = serde_json::from_str(&json).unwrap_test();
    assert_eq!(decoded, parameters());
    assert!(!json.contains("extra"));
    Ok(())
}

#[test]
fn test_decode_noise_plugin_parameters_nonfinite() {
    for scale in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let parameters = NoisePlugin {
            scale,
            ..parameters()
        };
        let error = decode_noise_plugin_parameters(&encode(&parameters)).unwrap_err();
        assert_eq!(
            error.message.as_deref(),
            Some("Noise plugin scale must be finite")
        );
    }
}

#[test]
fn test_decode_noise_plugin_parameters_ffi_slice_checks() {
    let bytes: [u8; 0] = [];
    let null_buffer = FfiSlice {
        ptr: std::ptr::null(),
        len: 0,
    };
    let empty_buffer = FfiSlice {
        ptr: bytes.as_ptr().cast(),
        len: 0,
    };
    let excessive_length = FfiSlice {
        ptr: bytes.as_ptr().cast(),
        len: isize::MAX as usize + 1,
    };
    for kwargs in [
        &null_buffer as *const FfiSlice,
        &empty_buffer,
        &excessive_length,
        std::ptr::null(),
    ] {
        let result = opendp_data__decode_noise_plugin_parameters(kwargs);
        let FfiResult::Err(error) = result else {
            panic!("invalid FFI argument was accepted");
        };
        util::into_owned(error).unwrap_test();
    }
}
