use std::{ffi::c_char, slice};

use polars::prelude::{Column, DataType, Field};
use polars_arrow::ffi::{ArrowSchema, export_field_to_c, import_field_from_c};
use polars_ffi_crate::version_0::{
    CallerContext, SeriesExport, export_column, import_series_buffer,
};

use crate::measurements::expr_noise::{NoisePlugin, noise_plugin_type_udf, noise_udf};

static PRIVATIZE_FIRST: &[u8] =
    b"OpenDP plugin expressions must be passed through make_private_lazyframe before execution.\0";

#[unsafe(no_mangle)]
pub extern "C" fn _polars_plugin_get_version() -> u32 {
    let (major, minor) = polars_ffi_crate::get_version();
    ((major as u32) << 16) + minor as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn _polars_plugin_get_last_error_message() -> *const c_char {
    PRIVATIZE_FIRST.as_ptr() as *const c_char
}

unsafe fn null_output_field(fields: *const ArrowSchema, len: usize, output: *mut ArrowSchema) {
    let fields = unsafe { slice::from_raw_parts(fields, len) };
    let name = fields
        .first()
        .and_then(|field| unsafe { import_field_from_c(field).ok() })
        .map(|field| Field::from(&field).name().clone())
        .unwrap_or_else(|| "opendp".into());
    let field = Field::new(name, DataType::Null);
    unsafe {
        *output = export_field_to_c(&field.to_arrow(polars::prelude::CompatLevel::newest()));
    }
}

macro_rules! export_shim {
    ($execute:ident, $field:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $execute(
            _input: *const SeriesExport,
            _input_len: usize,
            _kwargs: *const u8,
            _kwargs_len: usize,
            _output: *mut SeriesExport,
            _context: *const CallerContext,
        ) {
            // Leave output empty. Polars retrieves PRIVATIZE_FIRST as the error.
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $field(
            fields: *const ArrowSchema,
            len: usize,
            output: *mut ArrowSchema,
            _kwargs: *const u8,
            _kwargs_len: usize,
        ) {
            unsafe { null_output_field(fields, len, output) }
        }
    };
}

export_shim!(
    _polars_plugin_dp_frame_len,
    _polars_plugin_field_dp_frame_len
);
export_shim!(_polars_plugin_dp_sum, _polars_plugin_field_dp_sum);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _polars_plugin_noise_plugin(
    input: *mut SeriesExport,
    input_len: usize,
    kwargs: *const u8,
    kwargs_len: usize,
    output: *mut SeriesExport,
    _context: *const CallerContext,
) {
    let result = std::panic::catch_unwind(|| {
        let inputs = unsafe { import_series_buffer(input, input_len) }?;
        let inputs = inputs.into_iter().map(Column::from).collect::<Vec<_>>();
        let kwargs: NoisePlugin = serde_pickle::from_slice(
            unsafe { slice::from_raw_parts(kwargs, kwargs_len) },
            Default::default(),
        )
        .map_err(|err| {
            polars::error::polars_err!(
                ComputeError: "failed to parse OpenDP noise arguments: {}", err
            )
        })?;
        let result = noise_udf(&inputs, kwargs)?;
        unsafe {
            *output = export_column(&result);
        }
        polars::error::PolarsResult::Ok(())
    });
    let _ = result;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _polars_plugin_field_noise_plugin(
    fields: *const ArrowSchema,
    len: usize,
    output: *mut ArrowSchema,
    _kwargs: *const u8,
    _kwargs_len: usize,
) {
    let fields = unsafe { slice::from_raw_parts(fields, len) }
        .iter()
        .filter_map(|field| unsafe { import_field_from_c(field).ok() })
        .map(|field| Field::from(&field))
        .collect::<Vec<_>>();
    if let Ok(field) = noise_plugin_type_udf(&fields) {
        unsafe {
            *output = export_field_to_c(&field.to_arrow(polars::prelude::CompatLevel::newest()));
        }
    }
}

/// Anchor runtime-only Polars ABI entry points when OpenDP is linked into R
/// from a static archive. This does not construct a plugin expression.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__r_polars_plugin_keepalive() {
    std::hint::black_box(_polars_plugin_noise_plugin as *const ());
    std::hint::black_box(_polars_plugin_field_noise_plugin as *const ());
}
