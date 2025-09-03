use crate::error::{DoxError, DoxResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Add jitter to delays to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Create a policy for network operations
    pub fn for_network() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create a policy for file operations
    pub fn for_file_io() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }

    /// Create an aggressive retry policy
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    /// Calculate delay for the given attempt number
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let mut delay = self.initial_delay.as_millis() as f64;

        // Apply exponential backoff
        for _ in 1..attempt {
            delay *= self.backoff_multiplier;
        }

        // Cap at max delay
        delay = delay.min(self.max_delay.as_millis() as f64);

        // Add jitter if enabled
        if self.jitter {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.8..1.2);
            delay *= jitter_factor;
        }

        Duration::from_millis(delay as u64)
    }
}

/// Retry an async operation with the given policy
pub async fn retry_async<F, Fut, T>(
    policy: &RetryPolicy,
    operation_name: &str,
    mut operation: F,
) -> DoxResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = DoxResult<T>>,
{
    let mut last_error = None;

    for attempt in 1..=policy.max_attempts {
        debug!(
            "Attempting {} (attempt {}/{})",
            operation_name, attempt, policy.max_attempts
        );

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("{} succeeded after {} attempts", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(err) => {
                // Check if error is recoverable
                if !err.is_recoverable() {
                    error!(
                        "{} failed with non-recoverable error: {}",
                        operation_name, err
                    );
                    return Err(err);
                }

                warn!(
                    "{} failed (attempt {}/{}): {}",
                    operation_name, attempt, policy.max_attempts, err
                );
                last_error = Some(err);

                // Don't sleep after the last attempt
                if attempt < policy.max_attempts {
                    let delay = policy.calculate_delay(attempt);
                    debug!("Waiting {:?} before retry", delay);
                    sleep(delay).await;
                }
            }
        }
    }

    // All attempts failed
    let err = last_error.unwrap_or_else(|| DoxError::ConcurrentError {
        message: format!(
            "{} failed after {} attempts",
            operation_name, policy.max_attempts
        ),
    });

    error!(
        "{} failed after {} attempts",
        operation_name, policy.max_attempts
    );
    Err(err)
}

/// Retry a synchronous operation with the given policy
pub fn retry_sync<F, T>(
    policy: &RetryPolicy,
    operation_name: &str,
    mut operation: F,
) -> DoxResult<T>
where
    F: FnMut() -> DoxResult<T>,
{
    let mut last_error = None;

    for attempt in 1..=policy.max_attempts {
        debug!(
            "Attempting {} (attempt {}/{})",
            operation_name, attempt, policy.max_attempts
        );

        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    debug!("{} succeeded after {} attempts", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(err) => {
                // Check if error is recoverable
                if !err.is_recoverable() {
                    error!(
                        "{} failed with non-recoverable error: {}",
                        operation_name, err
                    );
                    return Err(err);
                }

                warn!(
                    "{} failed (attempt {}/{}): {}",
                    operation_name, attempt, policy.max_attempts, err
                );
                last_error = Some(err);

                // Don't sleep after the last attempt
                if attempt < policy.max_attempts {
                    let delay = policy.calculate_delay(attempt);
                    debug!("Waiting {:?} before retry", delay);
                    std::thread::sleep(delay);
                }
            }
        }
    }

    // All attempts failed
    let err = last_error.unwrap_or_else(|| DoxError::ConcurrentError {
        message: format!(
            "{} failed after {} attempts",
            operation_name, policy.max_attempts
        ),
    });

    error!(
        "{} failed after {} attempts",
        operation_name, policy.max_attempts
    );
    Err(err)
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<std::time::Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    /// Check if the circuit breaker allows the operation
    pub fn can_proceed(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        debug!("Circuit breaker timeout elapsed, transitioning to half-open");
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    debug!(
                        "Circuit breaker closing after {} successes",
                        self.success_count
                    );
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                self.state = CircuitState::HalfOpen;
                self.success_count = 1;
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.last_failure_time = Some(std::time::Instant::now());

        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    warn!(
                        "Circuit breaker opening after {} failures",
                        self.failure_count
                    );
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker reopening after failure in half-open state");
                self.state = CircuitState::Open;
                self.failure_count = 0;
                self.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }
}

/// Execute an operation with circuit breaker protection
pub async fn with_circuit_breaker<F, Fut, T>(
    breaker: &mut CircuitBreaker,
    operation_name: &str,
    operation: F,
) -> DoxResult<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = DoxResult<T>>,
{
    if !breaker.can_proceed() {
        return Err(DoxError::ConcurrentError {
            message: format!("{} blocked by circuit breaker", operation_name),
        });
    }

    match operation().await {
        Ok(result) => {
            breaker.record_success();
            Ok(result)
        }
        Err(err) => {
            if err.is_recoverable() {
                breaker.record_failure();
            }
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        assert_eq!(policy.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(800));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(1000)); // Capped at max
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let mut breaker = CircuitBreaker::new(2, 2, Duration::from_millis(100));

        // Initially closed
        assert!(breaker.can_proceed());

        // Record failures to open the circuit
        breaker.record_failure();
        assert!(breaker.can_proceed());
        breaker.record_failure();
        assert!(!breaker.can_proceed()); // Now open

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));
        assert!(breaker.can_proceed()); // Half-open

        // Success in half-open state
        breaker.record_success();
        assert!(breaker.can_proceed());
        breaker.record_success();
        assert!(breaker.can_proceed()); // Back to closed
    }
}
