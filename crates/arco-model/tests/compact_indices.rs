use arco_model::{Bounds, Model, Variable};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;

thread_local! {
    static ALLOCATION_ATTEMPTS: Cell<Option<usize>> = const { Cell::new(None) };
}

/// Counts allocation attempts on the test thread while forwarding to `System`.
///
/// This allocator is isolated to this integration-test binary. Counting is
/// enabled only around the measured call, so the counter does not allocate or
/// unwind while an allocator operation is in progress.
struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[expect(
    unsafe_code,
    reason = "the test-only allocator must implement GlobalAlloc"
)]
// SAFETY: Counting uses `try_with` and ignores unavailable or destroyed TLS,
// so it cannot unwind through the allocator. Each operation forwards its
// arguments unchanged to `System` and returns its result unchanged.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation_attempt();
        // SAFETY: The test allocator forwards the caller's layout unchanged to
        // the standard allocator and returns its pointer unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The pointer and layout were returned by this allocator and
        // are forwarded unchanged to the standard allocator.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation_attempt();
        // SAFETY: The test allocator forwards the caller's layout unchanged to
        // the standard allocator and returns its pointer unchanged.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation_attempt();
        // SAFETY: The pointer and layout were returned by this allocator and
        // are forwarded unchanged to the standard allocator.
        unsafe { System.realloc(pointer, layout, size) }
    }
}

fn record_allocation_attempt() {
    let _ = ALLOCATION_ATTEMPTS.try_with(|count| {
        if let Some(attempts) = count.get() {
            count.set(Some(attempts.saturating_add(1)));
        }
    });
}

fn enable_allocation_counting() -> bool {
    ALLOCATION_ATTEMPTS
        .try_with(|count| count.set(Some(0)))
        .is_ok()
}

fn disable_allocation_counting() {
    let _ = ALLOCATION_ATTEMPTS.try_with(|count| count.set(None));
}

fn allocation_attempts() -> Option<usize> {
    ALLOCATION_ATTEMPTS.try_with(Cell::get).ok().flatten()
}

fn prepared_model(variable_count: usize, constraint_capacity: usize) -> Model {
    let mut model = Model::with_capacities(variable_count, constraint_capacity);
    model
        .add_variables_uniform(Variable::continuous(Bounds::new(0.0, 1.0)), variable_count)
        .expect("test variables have valid bounds");
    model
}

fn column_values(model: &Model) -> Vec<Vec<(u32, f64)>> {
    model
        .columns()
        .map(|(_, column)| {
            column
                .iter()
                .map(|(constraint_id, value)| (constraint_id.inner(), *value))
                .collect()
        })
        .collect()
}

#[test]
fn compact_identity_rows_do_not_allocate_an_index_buffer() {
    let count = 16;
    let bounds = vec![Bounds::new(0.0, 1.0); count];
    let mut model = prepared_model(count, count);

    let counting_enabled = enable_allocation_counting();
    let result = black_box(model.add_constraints_compact(&[(0, 1.0)], &bounds));
    let attempts = counting_enabled.then(allocation_attempts).flatten();
    disable_allocation_counting();

    assert!(result.is_ok());
    assert_eq!(model.num_constraints(), count);
    assert_eq!(model.num_coefficients(), count);
    assert_eq!(
        attempts,
        Some(0),
        "identity compact path allocated: {attempts:?}"
    );
}

#[test]
fn compact_empty_inputs_and_empty_indexed_rows_preserve_validation() {
    let mut empty_compact = prepared_model(0, 0);
    assert!(
        empty_compact
            .add_constraints_compact(&[(0, 1.0)], &[])
            .is_ok()
    );
    assert_eq!(empty_compact.num_constraints(), 0);
    assert_eq!(empty_compact.num_coefficients(), 0);

    let mut empty_indexed = prepared_model(0, 0);
    assert!(
        empty_indexed
            .add_constraints_compact_indexed(&[(0, 1.0)], &[], &[])
            .is_ok()
    );
    assert_eq!(empty_indexed.num_constraints(), 0);
    assert_eq!(empty_indexed.num_coefficients(), 0);

    let valid_bounds = [Bounds::new(0.0, 1.0), Bounds::new(-1.0, 2.0)];
    let mut no_index_rows = prepared_model(1, valid_bounds.len());
    assert!(
        no_index_rows
            .add_constraints_compact_indexed(&[(0, 1.0)], &[], &valid_bounds)
            .is_ok()
    );
    assert_eq!(no_index_rows.num_constraints(), 0);
    assert_eq!(no_index_rows.num_coefficients(), 0);

    let invalid_bounds = [Bounds::new(0.0, 1.0), Bounds::new(2.0, 1.0)];
    let mut invalid_no_index_rows = prepared_model(1, invalid_bounds.len());
    let result =
        invalid_no_index_rows.add_constraints_compact_indexed(&[(0, 1.0)], &[], &invalid_bounds);
    assert!(matches!(
        result,
        Err(arco_model::ModelError::InvalidConstraintBounds { .. })
    ));
    assert_eq!(invalid_no_index_rows.num_constraints(), 0);
    assert_eq!(invalid_no_index_rows.num_coefficients(), 0);
}

#[test]
fn compact_identity_rows_match_indexed_rows() {
    let patterns = [(0, 1.5), (1, -2.0)];
    let bounds = [Bounds::new(0.0, 1.0), Bounds::new(-1.0, 2.0)];
    let mut compact = prepared_model(3, bounds.len());
    let mut indexed = prepared_model(3, bounds.len());

    compact
        .add_constraints_compact(&patterns, &bounds)
        .expect("compact rows should be valid");
    indexed
        .add_constraints_compact_indexed(&patterns, &[0, 1], &bounds)
        .expect("indexed rows should be valid");

    assert_eq!(compact.num_constraints(), indexed.num_constraints());
    assert_eq!(compact.num_coefficients(), indexed.num_coefficients());
    assert_eq!(column_values(&compact), column_values(&indexed));
}

#[test]
fn compact_rows_preserve_duplicate_patterns_and_partial_errors() {
    let duplicate_patterns = [(0, 1.0), (0, 2.0)];
    let bounds = [Bounds::new(0.0, 1.0), Bounds::new(0.0, 1.0)];
    let mut compact = prepared_model(2, bounds.len());
    let mut indexed = prepared_model(2, bounds.len());

    compact
        .add_constraints_compact(&duplicate_patterns, &bounds)
        .expect("duplicate patterns are valid storage entries");
    indexed
        .add_constraints_compact_indexed(&duplicate_patterns, &[0, 1], &bounds)
        .expect("duplicate patterns are valid storage entries");
    assert_eq!(column_values(&compact), column_values(&indexed));
    assert_eq!(compact.num_coefficients(), 4);

    let mut partial = prepared_model(2, bounds.len());
    let result = partial.add_constraints_compact(&[(0, 1.0), (0, f64::INFINITY)], &bounds);
    assert!(matches!(
        result,
        Err(arco_model::ModelError::InvalidCoefficient { .. })
    ));
    assert_eq!(partial.num_constraints(), 1);
    assert_eq!(partial.num_coefficients(), 1);
    assert_eq!(column_values(&partial), vec![vec![(0, 1.0)], vec![]]);
}

#[test]
fn indexed_rows_keep_zip_truncation_and_validate_bounds_before_mutation() {
    let bounds = [Bounds::new(0.0, 1.0), Bounds::new(0.0, 1.0)];
    let mut truncated = prepared_model(2, bounds.len());
    truncated
        .add_constraints_compact_indexed(&[(0, 1.0)], &[1], &bounds)
        .expect("the indexed path intentionally zips input slices");
    assert_eq!(truncated.num_constraints(), 1);
    assert_eq!(truncated.num_coefficients(), 1);
    assert_eq!(column_values(&truncated), vec![vec![], vec![(0, 1.0)]]);

    let mut extra_indices = prepared_model(1, bounds.len());
    extra_indices
        .add_constraints_compact_indexed(&[(0, 1.0)], &[0, 1, 2], &bounds[..1])
        .expect("the bounds slice controls indexed row count");
    assert_eq!(extra_indices.num_constraints(), 1);
    assert_eq!(extra_indices.num_coefficients(), 1);
    assert_eq!(column_values(&extra_indices), vec![vec![(0, 1.0)]]);

    let mut invalid_bounds = prepared_model(1, bounds.len());
    let invalid = [Bounds::new(0.0, 1.0), Bounds::new(2.0, 1.0)];
    let result = invalid_bounds.add_constraints_compact(&[(0, 1.0)], &invalid);
    assert!(matches!(
        result,
        Err(arco_model::ModelError::InvalidConstraintBounds { .. })
    ));
    assert_eq!(invalid_bounds.num_constraints(), 0);
    assert_eq!(invalid_bounds.num_coefficients(), 0);

    let mut invalid_variable = prepared_model(1, bounds.len());
    let result = invalid_variable.add_constraints_compact(&[(1, 1.0)], &bounds);
    assert!(matches!(
        result,
        Err(arco_model::ModelError::InvalidVariableId(_))
    ));
    assert_eq!(invalid_variable.num_constraints(), 1);
    assert_eq!(invalid_variable.num_coefficients(), 0);
}
