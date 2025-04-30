// SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use metrics::{counter, gauge};

const CLEANUP_PERIOD: Duration = Duration::from_mins(5);

struct State {
    reservations: HashMap<String, Instant>,
    last_cleanup: Instant,
}

pub struct Delayer {
    state: Mutex<State>,
}

impl Delayer {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State {
                reservations: Default::default(),
                last_cleanup: Instant::now(),
            }),
        }
    }

    pub async fn reserve(&self, key: &str, duration: Duration) {
        loop {
            let now = Instant::now();
            let reservation = {
                let mut state = self.state.lock().unwrap();
                if now > state.last_cleanup + CLEANUP_PERIOD {
                    state
                        .reservations
                        .retain(|_, reservation| *reservation > now);
                    state.last_cleanup = now;
                }
                gauge!("repology_linkchecker_delayer_reservations_total")
                    .set(state.reservations.len() as f64);
                if let Some(reservation) = state
                    .reservations
                    .get(key)
                    .filter(|reservation| **reservation > now)
                {
                    *reservation
                } else {
                    counter!("repology_linkchecker_delayer_reservation_attempts_total", "status" => "passed").increment(1);
                    state.reservations.insert(key.to_string(), now + duration);
                    return;
                }
            };

            counter!("repology_linkchecker_delayer_reservation_attempts_total", "status" => "delayed").increment(1);
            tokio::time::sleep_until(reservation.into()).await;
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn test_reserve_unrelated() {
        let delayer = Arc::new(Delayer::new());
        let start = Instant::now();

        let mut tasks = vec![];
        for n in 0..10 {
            let delayer = Arc::clone(&delayer);
            tasks.push(tokio::spawn(async move {
                delayer
                    .reserve(&format!("{n}"), Duration::from_millis(100))
                    .await;
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        let end = Instant::now();
        eprintln!("Took {:?}", end - start);
        assert!(end - start < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_reserve_related() {
        let delayer = Arc::new(Delayer::new());
        let start = Instant::now();

        let mut tasks = vec![];
        for _ in 0..10 {
            let delayer = Arc::clone(&delayer);
            tasks.push(tokio::spawn(async move {
                delayer.reserve("test", Duration::from_millis(100)).await;
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        let end = Instant::now();
        eprintln!("Took {:?}", end - start);
        assert!(end - start > Duration::from_millis(900));
        assert!(end - start < Duration::from_millis(1100));
    }
}
