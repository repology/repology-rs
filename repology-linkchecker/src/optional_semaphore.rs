// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use tokio::sync::{AcquireError, Semaphore, SemaphorePermit};

#[derive(Default)]
pub struct OptionalSemaphore(Option<Semaphore>);

#[must_use]
#[clippy::has_significant_drop]
pub struct OptionalSemaphorePermit<'a>(#[allow(unused)] Option<SemaphorePermit<'a>>);

impl OptionalSemaphore {
    pub fn new(permits: usize) -> Self {
        Self(match permits {
            0 => None,
            permits => Some(Semaphore::new(permits)),
        })
    }

    pub async fn acquire(&self) -> Result<OptionalSemaphorePermit<'_>, AcquireError> {
        match &self.0 {
            Some(semaphore) => semaphore
                .acquire()
                .await
                .map(|permit| OptionalSemaphorePermit(Some(permit))),
            None => Ok(OptionalSemaphorePermit(None)),
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_zero() {
        let semaphore = Arc::new(OptionalSemaphore::new(0));
        let start = Instant::now();

        let mut tasks = vec![];
        for _ in 0..5 {
            let semaphore = Arc::clone(&semaphore);
            tasks.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        let end = Instant::now();
        eprintln!("Took {:?}", end - start);
        assert!(end - start > Duration::from_millis(1000));
        assert!(end - start < Duration::from_millis(1900));
    }

    #[tokio::test]
    async fn test_nonzero() {
        let semaphore = Arc::new(OptionalSemaphore::new(1));
        let start = Instant::now();

        let mut tasks = vec![];
        for _ in 0..2 {
            let semaphore = Arc::clone(&semaphore);
            tasks.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        let end = Instant::now();
        eprintln!("Took {:?}", end - start);
        assert!(end - start > Duration::from_millis(2000));
        assert!(end - start < Duration::from_millis(2900));
    }
}
