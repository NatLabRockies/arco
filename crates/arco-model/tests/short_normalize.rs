use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;

use arco_model::VariableId;
use arco_model::expr::Expr;

thread_local! {
    static ALLOCATION_ATTEMPTS: Cell<Option<usize>> = const { Cell::new(None) };
}

/// Counts allocation attempts on the test thread while forwarding to `System`.
///
/// This allocator is isolated to this integration-test binary. The counter is
/// thread-local and only uses `Cell`; counting is enabled only around the
/// measured call, so counting neither allocates nor unwinds.
struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[expect(
    unsafe_code,
    reason = "the test-only allocator must implement GlobalAlloc"
)]
// SAFETY: Counting uses `try_with` and ignores unavailable or destroyed TLS,
// so it cannot unwind through the allocator. Each allocation operation forwards
// its arguments unchanged to `System` and returns its result unchanged.
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

fn normalized_terms_by_id(expression: &Expr) -> Vec<(u32, f64)> {
    let mut terms = expression
        .normalized_terms()
        .into_iter()
        .map(|(variable, coefficient)| (variable.inner(), coefficient))
        .collect::<Vec<_>>();
    terms.sort_by_key(|(variable, _)| *variable);
    terms
}

#[test]
fn short_normalization_preserves_zero_filtering() {
    let expression = Expr::from_linear(vec![(VariableId::new(1), 0.0), (VariableId::new(2), 4.0)]);

    assert_eq!(normalized_terms_by_id(&expression), vec![(2, 4.0)]);
}

#[test]
fn short_normalization_preserves_empty_and_all_zero_inputs() {
    assert!(Expr::from_linear(Vec::new()).normalized_terms().is_empty());
    assert!(
        Expr::from_linear(vec![(VariableId::new(1), 0.0)])
            .normalized_terms()
            .is_empty()
    );
    assert!(
        Expr::from_linear(vec![(VariableId::new(1), 0.0), (VariableId::new(2), -0.0),])
            .normalized_terms()
            .is_empty()
    );
}

#[test]
fn short_normalization_preserves_duplicate_cancellation() {
    let expression = Expr::from_linear(vec![(VariableId::new(2), 4.0), (VariableId::new(2), -4.0)]);

    assert!(expression.normalized_terms().is_empty());
}

#[test]
fn short_normalization_preserves_duplicate_accumulation() {
    let expression = Expr::from_linear(vec![(VariableId::new(3), 2.0), (VariableId::new(3), 5.0)]);

    assert_eq!(normalized_terms_by_id(&expression), vec![(3, 7.0)]);
}

#[test]
fn short_normalization_preserves_nonfinite_classification() {
    let expression = Expr::from_linear(vec![
        (VariableId::new(1), f64::NAN),
        (VariableId::new(2), f64::INFINITY),
    ]);

    let terms = normalized_terms_by_id(&expression);
    assert_eq!(terms.len(), 2);
    assert!(terms[0].1.is_nan());
    assert_eq!(terms[1], (2, f64::INFINITY));
}

#[test]
fn short_normalization_preserves_infinite_duplicate_classification() {
    let expression = Expr::from_linear(vec![
        (VariableId::new(1), f64::INFINITY),
        (VariableId::new(1), f64::NEG_INFINITY),
    ]);

    let terms = expression.normalized_terms();
    assert_eq!(terms.len(), 1);
    assert!(terms[0].1.is_nan());
}

#[test]
fn long_normalization_keeps_hash_map_fallback_behavior() {
    let expression = Expr::from_linear(vec![
        (VariableId::new(1), 2.0),
        (VariableId::new(2), 3.0),
        (VariableId::new(1), 4.0),
    ]);

    assert_eq!(
        normalized_terms_by_id(&expression),
        vec![(1, 6.0), (2, 3.0)]
    );
}

fn normalization_allocation_attempts(expression: &Expr) -> Option<usize> {
    let counting_enabled = enable_allocation_counting();
    let normalized = black_box(expression.normalized_terms());
    let attempts = counting_enabled.then(allocation_attempts).flatten();
    disable_allocation_counting();
    black_box(normalized);
    attempts
}

#[test]
fn short_normalization_uses_at_most_one_output_allocation() {
    let one_term = Expr::from_linear(vec![(VariableId::new(1), 1.0)]);
    let two_terms = Expr::from_linear(vec![(VariableId::new(1), 1.0), (VariableId::new(2), 2.0)]);

    let one_term_attempts = normalization_allocation_attempts(&one_term);
    let two_term_attempts = normalization_allocation_attempts(&two_terms);

    assert!(
        one_term_attempts.is_some_and(|attempts| attempts <= 1),
        "one-term normalization counter unavailable or exceeded one allocation: {one_term_attempts:?}"
    );
    assert!(
        two_term_attempts.is_some_and(|attempts| attempts <= 1),
        "two-term normalization counter unavailable or exceeded one allocation: {two_term_attempts:?}"
    );
}
