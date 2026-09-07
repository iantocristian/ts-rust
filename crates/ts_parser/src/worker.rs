use std::{cell::Cell, panic::resume_unwind, thread};

thread_local! { static ON_PARSER_WORKER: Cell<bool> = const { Cell::new(false) }; }

/// Execute a complete parse or a batch on the reserved native parser stack.
/// Recursive grammar uses stacker segments; the initial reservation also covers
/// source-equivalent traversals that occur before a grammar guard. Lazy callers
/// enter this boundary before taking any lazy publication lock.
///
/// Nested parser operations run inline, so they cannot enqueue work behind a
/// transaction held by their own worker. Unwinds retain their original payload.
pub fn on_parser_worker<T: Send>(operation: impl FnOnce() -> T + Send) -> T {
    if ON_PARSER_WORKER.get() {
        return operation();
    }
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name("ts-parser".into())
            .stack_size(256 * 1024 * 1024)
            .spawn_scoped(scope, || {
                ON_PARSER_WORKER.set(true);
                operation()
            })
            .expect("could not start native parser worker");
        match worker.join() {
            Ok(value) => value,
            Err(panic) => resume_unwind(panic),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::on_parser_worker;
    use std::{
        panic::{catch_unwind, panic_any},
        thread,
    };

    #[test]
    fn nested_operations_stay_on_the_same_worker_and_can_borrow_batch_input() {
        let caller = thread::current().id();
        let bytes = [1_u8, 2, 3];
        let sum = on_parser_worker(|| {
            let worker = thread::current().id();
            assert_ne!(worker, caller);
            on_parser_worker(|| {
                assert_eq!(thread::current().id(), worker);
                bytes.iter().copied().sum::<u8>()
            })
        });
        assert_eq!(sum, 6);
    }

    #[test]
    fn worker_unwind_preserves_the_original_payload_and_next_operation_runs() {
        #[derive(Debug, PartialEq)]
        struct Marker(u32);
        let failure = catch_unwind(|| on_parser_worker(|| panic_any(Marker(42))));
        assert_eq!(
            *failure.unwrap_err().downcast::<Marker>().unwrap(),
            Marker(42)
        );
        assert_eq!(on_parser_worker(|| 7), 7);
    }
}
