use std::time::Duration;

pub const MAX_STEPS: u64 = 20_000_000_000;
pub const CHUNK_STEPS: usize = 2_000_000;
pub const TIMEOUT_SECS: u64 = 900;
pub const UART_BYTES: usize = 1_048_576;
pub const INPUT_BYTES: usize = 131_072;

pub struct Budget {
    pub steps: u64,
    pub chunks: u64,
    limit: u64,
    chunk: usize,
    chunk_limit: u64,
    timeout: Duration,
}

impl Budget {
    pub fn new(limit: u64, chunk: usize, seconds: u64) -> Result<Self, String> {
        if !(1..=MAX_STEPS).contains(&limit)
            || !(1..=CHUNK_STEPS).contains(&chunk)
            || !(1..=TIMEOUT_SECS).contains(&seconds)
        {
            return Err("bounds must be positive and cannot exceed the standard limits".into());
        }
        Ok(Self {
            steps: 0,
            chunks: 0,
            limit,
            chunk,
            chunk_limit: limit.div_ceil(chunk as u64),
            timeout: Duration::from_secs(seconds),
        })
    }

    pub fn next(&self, elapsed: Duration) -> Result<usize, String> {
        self.check_time(elapsed)?;
        if self.steps >= self.limit || self.chunks >= self.chunk_limit {
            return Err(format!(
                "guest budget exhausted: steps={}, chunks={}, seconds={:.3}",
                self.steps,
                self.chunks,
                elapsed.as_secs_f64()
            ));
        }
        Ok((self.limit - self.steps).min(self.chunk as u64) as usize)
    }

    pub fn check_time(&self, elapsed: Duration) -> Result<(), String> {
        if elapsed >= self.timeout {
            return Err("configured host timeout exceeded".into());
        }
        Ok(())
    }

    /// Every call consumes a chunk, including idle calls that execute zero instructions.
    /// Thus chunks strictly increases to a finite limit independently of guest progress.
    pub fn record(&mut self, requested: usize, executed: usize) -> Result<(), String> {
        if executed > requested {
            return Err("VM exceeded the requested instruction budget".into());
        }
        self.steps += executed as u64;
        self.chunks += 1;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/budget.rs"]
mod tests;
