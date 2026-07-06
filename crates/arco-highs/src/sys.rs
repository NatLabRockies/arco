#![allow(unsafe_code)]

#[cfg(feature = "bundled-highs")]
pub use highs_sys::*;

#[cfg(not(feature = "bundled-highs"))]
mod raw {
    #![allow(non_snake_case)]

    use std::ffi::{c_char, c_void};
    use std::os::raw::c_int;

    pub type HighsInt = c_int;

    pub const MODEL_STATUS_OPTIMAL: HighsInt = 7;
    pub const MODEL_STATUS_INFEASIBLE: HighsInt = 8;
    pub const MODEL_STATUS_UNBOUNDED_OR_INFEASIBLE: HighsInt = 9;
    pub const MODEL_STATUS_UNBOUNDED: HighsInt = 10;
    pub const MODEL_STATUS_REACHED_TIME_LIMIT: HighsInt = 13;
    pub const MODEL_STATUS_REACHED_ITERATION_LIMIT: HighsInt = 14;

    pub const STATUS_OK: HighsInt = 0;
    pub const STATUS_WARNING: HighsInt = 1;

    pub const SOLUTION_STATUS_FEASIBLE: HighsInt = 2;

    pub const MATRIX_FORMAT_COLUMN_WISE: HighsInt = 1;
    pub const OBJECTIVE_SENSE_MINIMIZE: HighsInt = 1;
    pub const OBJECTIVE_SENSE_MAXIMIZE: HighsInt = -1;

    pub const VAR_TYPE_CONTINUOUS: HighsInt = 0;
    pub const VAR_TYPE_INTEGER: HighsInt = 1;

    unsafe extern "C" {
        pub fn Highs_create() -> *mut c_void;
        pub fn Highs_destroy(highs: *mut c_void);
        pub fn Highs_version() -> *const c_char;
        pub fn Highs_run(highs: *mut c_void) -> HighsInt;
        pub fn Highs_passLp(
            highs: *mut c_void,
            num_col: HighsInt,
            num_row: HighsInt,
            num_nz: HighsInt,
            a_format: HighsInt,
            sense: HighsInt,
            offset: f64,
            col_cost: *const f64,
            col_lower: *const f64,
            col_upper: *const f64,
            row_lower: *const f64,
            row_upper: *const f64,
            a_start: *const HighsInt,
            a_index: *const HighsInt,
            a_value: *const f64,
        ) -> HighsInt;
        pub fn Highs_passMip(
            highs: *mut c_void,
            num_col: HighsInt,
            num_row: HighsInt,
            num_nz: HighsInt,
            a_format: HighsInt,
            sense: HighsInt,
            offset: f64,
            col_cost: *const f64,
            col_lower: *const f64,
            col_upper: *const f64,
            row_lower: *const f64,
            row_upper: *const f64,
            a_start: *const HighsInt,
            a_index: *const HighsInt,
            a_value: *const f64,
            integrality: *const HighsInt,
        ) -> HighsInt;
        pub fn Highs_getModelStatus(highs: *mut c_void) -> HighsInt;
        pub fn Highs_getObjectiveValue(highs: *const c_void) -> f64;
        pub fn Highs_getNumCol(highs: *const c_void) -> HighsInt;
        pub fn Highs_getNumRow(highs: *const c_void) -> HighsInt;
        pub fn Highs_getIntInfoValue(
            highs: *const c_void,
            info: *const c_char,
            value: *mut HighsInt,
        ) -> HighsInt;
        pub fn Highs_getDoubleInfoValue(
            highs: *const c_void,
            info: *const c_char,
            value: *mut f64,
        ) -> HighsInt;
        pub fn Highs_setSolution(
            highs: *mut c_void,
            col_value: *const f64,
            row_value: *const f64,
            col_dual: *const f64,
            row_dual: *const f64,
        ) -> HighsInt;
        pub fn Highs_setBoolOptionValue(
            highs: *mut c_void,
            option: *const c_char,
            value: HighsInt,
        ) -> HighsInt;
        pub fn Highs_setIntOptionValue(
            highs: *mut c_void,
            option: *const c_char,
            value: HighsInt,
        ) -> HighsInt;
        pub fn Highs_setDoubleOptionValue(
            highs: *mut c_void,
            option: *const c_char,
            value: f64,
        ) -> HighsInt;
        pub fn Highs_setStringOptionValue(
            highs: *mut c_void,
            option: *const c_char,
            value: *const c_char,
        ) -> HighsInt;
        pub fn Highs_getSolution(
            highs: *const c_void,
            col_value: *mut f64,
            col_dual: *mut f64,
            row_value: *mut f64,
            row_dual: *mut f64,
        ) -> HighsInt;
    }
}

#[cfg(not(feature = "bundled-highs"))]
pub use raw::*;
