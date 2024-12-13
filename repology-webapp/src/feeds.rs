// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use chrono::{DateTime, Utc};

const TIMESTAMP_SHIFT: Duration = Duration::from_secs(1);

pub trait EventWithTimestamp {
    fn timestamp(&self) -> DateTime<Utc>;
    fn set_timestamp(&mut self, timestamp: DateTime<Utc>);
}

pub fn unicalize_feed_timestamps(events: &mut [impl EventWithTimestamp]) {
    if let Some(last_event) = events.first() {
        let mut previous_timestamp = last_event.timestamp();

        events.iter_mut().skip(1).for_each(|event| {
            if event.timestamp() > previous_timestamp - TIMESTAMP_SHIFT {
                event.set_timestamp(previous_timestamp - TIMESTAMP_SHIFT);
            }
            previous_timestamp = event.timestamp();
        });
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_unicalize_feed_timestamps() {
        #[derive(Debug, PartialEq)]
        struct Event {
            timestamp: DateTime<Utc>,
        }

        impl EventWithTimestamp for Event {
            fn timestamp(&self) -> DateTime<Utc> {
                self.timestamp
            }

            fn set_timestamp(&mut self, timestamp: DateTime<Utc>) {
                self.timestamp = timestamp;
            }
        }

        let base = Utc::now();
        let mut events = vec![
            Event { timestamp: base },
            Event {
                timestamp: base - Duration::from_secs(5),
            },
            Event {
                timestamp: base - Duration::from_secs(5),
            },
            Event {
                timestamp: base - Duration::from_secs(5),
            },
            Event {
                timestamp: base - Duration::from_secs(10),
            },
        ];

        unicalize_feed_timestamps(&mut events);

        assert_eq!(events, vec![
            Event { timestamp: base },
            Event {
                timestamp: base - Duration::from_secs(5)
            },
            Event {
                timestamp: base - Duration::from_secs(6)
            },
            Event {
                timestamp: base - Duration::from_secs(7)
            },
            Event {
                timestamp: base - Duration::from_secs(10)
            },
        ]);
    }
}
