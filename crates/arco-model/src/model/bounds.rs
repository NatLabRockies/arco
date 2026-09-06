use crate::types::{Bounds, Constraint};

pub(crate) const BOUNDS_BLOCK_SIZE: usize = 256;

pub(crate) trait BoundValue: Copy {
    fn same_bits(&self, other: &Self) -> bool;
}

impl BoundValue for Bounds {
    fn same_bits(&self, other: &Self) -> bool {
        self.lower.to_bits() == other.lower.to_bits()
            && self.upper.to_bits() == other.upper.to_bits()
    }
}

impl BoundValue for Constraint {
    fn same_bits(&self, other: &Self) -> bool {
        self.bounds.same_bits(&other.bounds)
    }
}

#[derive(Debug, Clone)]
enum BoundBlock<T> {
    Uniform(T),
    Dense(Vec<T>),
}

#[derive(Debug, Clone)]
pub(crate) struct BoundBlocks<T> {
    blocks: Vec<BoundBlock<T>>,
    len: usize,
}

impl<T: BoundValue> BoundBlocks<T> {
    pub(crate) fn new() -> Self {
        Self {
            blocks: Vec::new(),
            len: 0,
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let mut blocks = Self::new();
        blocks.reserve(capacity);
        blocks
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        let required_blocks = (self.len + additional).div_ceil(BOUNDS_BLOCK_SIZE);
        self.blocks
            .reserve(required_blocks.saturating_sub(self.blocks.len()));
    }

    pub(crate) fn capacity(&self) -> usize {
        self.blocks.capacity() * BOUNDS_BLOCK_SIZE
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn push(&mut self, value: T) {
        let offset = self.len % BOUNDS_BLOCK_SIZE;
        match (offset, self.blocks.last_mut()) {
            (0, _) => self.blocks.push(BoundBlock::Uniform(value)),
            (_, Some(block)) => match block {
                BoundBlock::Uniform(existing) if existing.same_bits(&value) => {}
                BoundBlock::Uniform(existing) => {
                    let mut dense = Vec::with_capacity(BOUNDS_BLOCK_SIZE);
                    dense.resize(offset, *existing);
                    dense.push(value);
                    *block = BoundBlock::Dense(dense);
                }
                BoundBlock::Dense(values) => values.push(value),
            },
            (_, None) => {
                debug_assert_eq!(self.len, 0, "nonempty storage must have a block");
                self.blocks.push(BoundBlock::Uniform(value));
            }
        }
        self.len += 1;
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let block = self.blocks.get(index / BOUNDS_BLOCK_SIZE)?;
        let offset = index % BOUNDS_BLOCK_SIZE;
        match block {
            BoundBlock::Uniform(value) => Some(value),
            BoundBlock::Dense(values) => values.get(offset),
        }
    }

    pub(crate) fn iter(&self) -> BoundBlocksIter<'_, T> {
        BoundBlocksIter {
            blocks: self.blocks.iter(),
            dense: None,
            uniform: None,
            remaining: self.len,
        }
    }
}

impl<T: BoundValue> Default for BoundBlocks<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct BoundBlocksIter<'a, T> {
    blocks: std::slice::Iter<'a, BoundBlock<T>>,
    dense: Option<std::slice::Iter<'a, T>>,
    uniform: Option<(&'a T, usize)>,
    remaining: usize,
}

impl<'a, T> Iterator for BoundBlocksIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.remaining == 0 {
                return None;
            }

            if let Some(values) = &mut self.dense {
                if let Some(value) = values.next() {
                    self.remaining -= 1;
                    return Some(value);
                }
                self.dense = None;
            }

            if let Some((value, remaining)) = &mut self.uniform {
                if *remaining > 0 {
                    *remaining -= 1;
                    self.remaining -= 1;
                    return Some(*value);
                }
                self.uniform = None;
            }

            let block = self.blocks.next()?;
            match block {
                BoundBlock::Uniform(value) => self.uniform = Some((value, BOUNDS_BLOCK_SIZE)),
                BoundBlock::Dense(values) => self.dense = Some(values.iter()),
            }
        }
    }
}

impl<T> ExactSizeIterator for BoundBlocksIter<'_, T> {
    fn len(&self) -> usize {
        self.remaining
    }
}

#[cfg(test)]
mod tests {
    use crate::model::bounds::{BOUNDS_BLOCK_SIZE, BoundBlock, BoundBlocks};
    use crate::types::{Bounds, Constraint};
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    use std::hint::black_box;

    thread_local! {
        static ALLOCATED_BYTES: Cell<Option<usize>> = const { Cell::new(None) };
    }

    struct CountingAllocator;

    #[global_allocator]
    static ALLOCATOR: CountingAllocator = CountingAllocator;

    #[expect(
        unsafe_code,
        reason = "the test-only allocator must implement GlobalAlloc"
    )]
    // SAFETY: Each operation forwards its unchanged arguments to System. The
    // thread-local counter uses try_with and cannot unwind through allocation.
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record_allocation(layout.size());
            // SAFETY: The layout is forwarded unchanged to the system allocator.
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            // SAFETY: The pointer and layout came from this allocator.
            unsafe { System.dealloc(pointer, layout) };
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            record_allocation(layout.size());
            // SAFETY: The layout is forwarded unchanged to the system allocator.
            unsafe { System.alloc_zeroed(layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            record_allocation(size);
            // SAFETY: The pointer, old layout, and new size are forwarded unchanged.
            unsafe { System.realloc(pointer, layout, size) }
        }
    }

    fn record_allocation(bytes: usize) {
        let _ = ALLOCATED_BYTES.try_with(|cell| {
            if let Some(total) = cell.get() {
                cell.set(Some(total.saturating_add(bytes)));
            }
        });
    }

    fn measure_uniform_bound_blocks(count: usize) -> usize {
        ALLOCATED_BYTES.with(|total| total.set(Some(0)));
        let mut storage = BoundBlocks::with_capacity(count);
        for _ in 0..count {
            storage.push(bounds(1.0));
        }
        let bytes = ALLOCATED_BYTES
            .with(Cell::get)
            .expect("allocation measurement must be initialized");
        ALLOCATED_BYTES.with(|total| total.set(None));
        black_box(storage);
        bytes
    }

    fn measure_dense_bounds(count: usize) -> usize {
        ALLOCATED_BYTES.with(|total| total.set(Some(0)));
        let mut storage = Vec::with_capacity(count);
        storage.resize(count, bounds(1.0));
        let bytes = ALLOCATED_BYTES
            .with(Cell::get)
            .expect("allocation measurement must be initialized");
        ALLOCATED_BYTES.with(|total| total.set(None));
        black_box(storage);
        bytes
    }

    fn bounds(value: f64) -> Bounds {
        Bounds::new(value, value + 1.0)
    }

    #[test]
    fn empty_and_block_boundaries_are_indexable() {
        let mut storage = BoundBlocks::new();
        assert_eq!(storage.len(), 0);
        assert!(storage.get(0).is_none());

        for index in 0..=BOUNDS_BLOCK_SIZE {
            storage.push(bounds(1.0));
            assert_eq!(storage.get(index), Some(&bounds(1.0)));
        }
        assert_eq!(storage.len(), BOUNDS_BLOCK_SIZE + 1);
        assert_eq!(storage.iter().count(), storage.len());
    }

    #[test]
    fn mixed_values_use_dense_only_for_the_mixed_block() {
        let mut storage = BoundBlocks::new();
        for index in 0..(BOUNDS_BLOCK_SIZE * 2 + 3) {
            storage.push(bounds(if index == BOUNDS_BLOCK_SIZE + 1 {
                2.0
            } else {
                1.0
            }));
        }

        let values: Vec<_> = storage.iter().map(|value| value.lower).collect();
        assert_eq!(values.len(), BOUNDS_BLOCK_SIZE * 2 + 3);
        assert_eq!(values[BOUNDS_BLOCK_SIZE + 1].to_bits(), 2.0f64.to_bits());
        assert_eq!(
            values
                .iter()
                .filter(|value| value.to_bits() == 2.0f64.to_bits())
                .count(),
            1
        );
    }

    #[test]
    fn late_divergence_keeps_dense_payload_within_one_block() {
        let mut storage = BoundBlocks::new();
        for _ in 0..(BOUNDS_BLOCK_SIZE - 1) {
            storage.push(bounds(1.0));
        }
        storage.push(bounds(2.0));

        let BoundBlock::Dense(values) = &storage.blocks[0] else {
            panic!("a divergent block must become dense");
        };
        assert_eq!(values.len(), BOUNDS_BLOCK_SIZE);
        assert!(values.capacity() <= BOUNDS_BLOCK_SIZE);
    }

    #[test]
    fn signed_zero_bits_do_not_merge() {
        let mut storage = BoundBlocks::new();
        storage.push(Bounds::new(-0.0, 1.0));
        storage.push(Bounds::new(0.0, 1.0));

        assert_eq!(storage.get(0).unwrap().lower.to_bits(), (-0.0f64).to_bits());
        assert_eq!(storage.get(1).unwrap().lower.to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn arbitrary_values_round_trip_through_mixed_blocks() {
        let mut state = 0x1234_5678_9abc_def0u64;
        let expected: Vec<_> = (0..(BOUNDS_BLOCK_SIZE * 3 + 19))
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let lower = ((state >> 32) % 10_000) as f64;
                Bounds::new(lower, lower + 1.0)
            })
            .collect();
        let mut storage = BoundBlocks::new();
        for bound in &expected {
            storage.push(*bound);
        }

        for (actual, expected) in storage.iter().zip(&expected) {
            assert_eq!(actual.lower.to_bits(), expected.lower.to_bits());
            assert_eq!(actual.upper.to_bits(), expected.upper.to_bits());
        }
        assert_eq!(storage.iter().count(), expected.len());
    }

    #[test]
    fn clone_preserves_values() {
        let mut storage = BoundBlocks::with_capacity(BOUNDS_BLOCK_SIZE + 1);
        storage.push(bounds(1.0));
        storage.push(bounds(2.0));
        let clone = storage.clone();

        assert_eq!(clone.len(), 2);
        assert_eq!(
            clone.iter().map(|value| value.lower).collect::<Vec<_>>(),
            [1.0, 2.0]
        );
    }

    #[test]
    fn uniform_values_have_no_dense_payload_per_block() {
        let mut variables = BoundBlocks::new();
        let mut constraints = BoundBlocks::new();
        for _ in 0..(BOUNDS_BLOCK_SIZE * 1_000) {
            variables.push(bounds(1.0));
            constraints.push(Constraint {
                bounds: bounds(1.0),
            });
        }

        assert_eq!(variables.blocks.len(), 1_000);
        assert_eq!(constraints.blocks.len(), 1_000);
        assert!(
            variables
                .blocks
                .iter()
                .all(|block| matches!(block, BoundBlock::Uniform(_)))
        );
        assert!(
            constraints
                .blocks
                .iter()
                .all(|block| matches!(block, BoundBlock::Uniform(_)))
        );
    }

    #[test]
    fn uniform_bounds_avoid_the_dense_payload_allocation() {
        let count = BOUNDS_BLOCK_SIZE * 1_000;
        let compact_bytes = measure_uniform_bound_blocks(count);
        let dense_bytes = measure_dense_bounds(count);

        assert!(
            compact_bytes < dense_bytes / 4,
            "uniform bound storage allocated {compact_bytes} bytes versus dense {dense_bytes}"
        );
    }
}
