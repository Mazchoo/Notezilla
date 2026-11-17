use crate::constants::{GEOMETRIC_RATIO, NUMBER_OF_RETRIES, RETRY_INITIAL_DELAY_MS};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use wasm_bindgen::JsValue;

/// Return the waits before each of `retries` retries: `initial_ms * ratio^n`, capped at `i32::MAX`.
pub fn backoff_delays_ms(retries: u32, initial_ms: u32, ratio: f64) -> impl Iterator<Item = i32> {
    (0..retries).map(move |n| {
        let exponent = i32::try_from(n).unwrap_or(i32::MAX);
        let delay = f64::from(initial_ms) * ratio.powi(exponent);
        delay.clamp(0.0, f64::from(i32::MAX)) as i32
    })
}

/// Bump `generation` and return a check that stays true until the next bump.
///
/// A newer probe bumps the counter, so an older retry loop sees it is stale and stops.
pub fn begin_generation(generation: &'static AtomicU64) -> impl Fn() -> bool {
    let mine = generation.fetch_add(1, Ordering::Relaxed) + 1;
    move || generation.load(Ordering::Relaxed) == mine
}

/// Resolve after `ms` milliseconds via `setTimeout`. Resolves at once without a window.
pub async fn sleep_ms(ms: i32) {
    let timeout = js_sys::Promise::new(&mut |resolve, _reject| {
        let Some(window) = web_sys::window() else {
            let _ = resolve.call0(&JsValue::UNDEFINED);
            return;
        };
        if window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .is_err()
        {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(timeout).await;
}

/// Run `attempt`, retrying with geometric backoff (`NUMBER_OF_RETRIES`, `GEOMETRIC_RATIO`)
/// until it succeeds.
///
/// `on_failure` runs after every failed attempt, so the first failure is reported at once.
/// Stops without calling either callback as soon as `is_current` turns false.
pub async fn retry_with_backoff<T, Fut>(
    attempt: impl FnMut() -> Fut,
    is_current: impl Fn() -> bool,
    on_success: impl FnOnce(T),
    on_failure: impl FnMut(String),
) where
    Fut: Future<Output = Result<T, String>>,
{
    retry_with_delays(
        backoff_delays_ms(NUMBER_OF_RETRIES, RETRY_INITIAL_DELAY_MS, GEOMETRIC_RATIO),
        sleep_ms,
        attempt,
        is_current,
        on_success,
        on_failure,
    )
    .await;
}

/// Retry loop behind [`retry_with_backoff`], with the delays and sleep injected.
async fn retry_with_delays<T, Fut, SleepFut>(
    mut delays: impl Iterator<Item = i32>,
    sleep: impl Fn(i32) -> SleepFut,
    mut attempt: impl FnMut() -> Fut,
    is_current: impl Fn() -> bool,
    on_success: impl FnOnce(T),
    mut on_failure: impl FnMut(String),
) where
    Fut: Future<Output = Result<T, String>>,
    SleepFut: Future<Output = ()>,
{
    loop {
        let result = attempt().await;
        if !is_current() {
            return;
        }
        match result {
            Ok(value) => {
                on_success(value);
                return;
            }
            Err(error) => on_failure(error),
        }
        let Some(delay) = delays.next() else {
            return;
        };
        sleep(delay).await;
        if !is_current() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{backoff_delays_ms, begin_generation, retry_with_delays};
    use std::cell::{Cell, RefCell};
    use std::future::{ready, Future};
    use std::pin::pin;
    use std::sync::atomic::AtomicU64;
    use std::task::{Context, Poll, Waker};

    /// Poll `future` once and return its output; the futures under test never pend.
    fn run_ready<F: Future>(future: F) -> F::Output {
        let mut cx = Context::from_waker(Waker::noop());
        match pin!(future).poll(&mut cx) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("future pended"),
        }
    }

    /// Run the retry loop over `outcomes`, returning (attempts, sleeps, failures, success).
    fn run_retry(
        delays: Vec<i32>,
        outcomes: Vec<Result<u32, String>>,
        stale_after_attempt: Option<usize>,
    ) -> (usize, Vec<i32>, Vec<String>, Option<u32>) {
        let attempts = Cell::new(0usize);
        let sleeps = RefCell::new(Vec::new());
        let failures = RefCell::new(Vec::new());
        let success = Cell::new(None);
        let outcomes = RefCell::new(outcomes.into_iter());
        run_ready(retry_with_delays(
            delays.into_iter(),
            |ms| {
                sleeps.borrow_mut().push(ms);
                ready(())
            },
            || {
                attempts.set(attempts.get() + 1);
                ready(outcomes.borrow_mut().next().expect("outcome"))
            },
            || stale_after_attempt.is_none_or(|n| attempts.get() < n),
            |value| success.set(Some(value)),
            |error| failures.borrow_mut().push(error),
        ));
        (
            attempts.get(),
            sleeps.into_inner(),
            failures.into_inner(),
            success.get(),
        )
    }

    #[test]
    /// Assert each delay is the previous one times the ratio, starting at the initial delay.
    fn backoff_delays_grow_geometrically() {
        assert_eq!(
            backoff_delays_ms(4, 1000, 2.0).collect::<Vec<_>>(),
            vec![1000, 2000, 4000, 8000]
        );
        assert_eq!(
            backoff_delays_ms(3, 100, 3.0).collect::<Vec<_>>(),
            vec![100, 300, 900]
        );
        assert_eq!(backoff_delays_ms(0, 1000, 2.0).count(), 0);
    }

    #[test]
    /// Assert huge delays cap at `i32::MAX` instead of overflowing.
    fn backoff_delays_cap_at_i32_max() {
        assert_eq!(backoff_delays_ms(1, u32::MAX, 1.0).next(), Some(i32::MAX));
        assert_eq!(backoff_delays_ms(40, 1000, 10.0).last(), Some(i32::MAX));
    }

    #[test]
    /// Assert a first-try success reports success without failing or sleeping.
    fn retry_succeeds_first_try() {
        let (attempts, sleeps, failures, success) = run_retry(vec![10, 20], vec![Ok(7)], None);
        assert_eq!(attempts, 1);
        assert!(sleeps.is_empty());
        assert!(failures.is_empty());
        assert_eq!(success, Some(7));
    }

    #[test]
    /// Assert each failure is reported before its backoff and a later success still lands.
    fn retry_reports_each_failure_then_succeeds() {
        let (attempts, sleeps, failures, success) = run_retry(
            vec![10, 20, 40],
            vec![Err("a".into()), Err("b".into()), Ok(3)],
            None,
        );
        assert_eq!(attempts, 3);
        assert_eq!(sleeps, vec![10, 20]);
        assert_eq!(failures, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(success, Some(3));
    }

    #[test]
    /// Assert the loop gives up after the last retry with every failure reported.
    fn retry_gives_up_after_the_last_retry() {
        let (attempts, sleeps, failures, success) = run_retry(
            vec![10, 20],
            vec![Err("a".into()), Err("b".into()), Err("c".into())],
            None,
        );
        assert_eq!(attempts, 3);
        assert_eq!(sleeps, vec![10, 20]);
        assert_eq!(failures.len(), 3);
        assert_eq!(success, None);
    }

    #[test]
    /// Assert a superseded loop drops its result and stops retrying.
    fn retry_stops_when_superseded() {
        let (attempts, _, failures, success) =
            run_retry(vec![10, 20], vec![Err("a".into()), Ok(1)], Some(2));
        assert_eq!(attempts, 2);
        assert_eq!(failures, vec!["a".to_string()]);
        assert_eq!(success, None);
    }

    #[test]
    /// Assert a generation check turns false once a newer generation begins.
    fn begin_generation_goes_stale_after_the_next_bump() {
        static GENERATION: AtomicU64 = AtomicU64::new(0);
        let first = begin_generation(&GENERATION);
        assert!(first());
        let second = begin_generation(&GENERATION);
        assert!(!first());
        assert!(second());
    }
}
