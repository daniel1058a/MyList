use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    ToWatch,
    Watching,
    Finished,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToWatch => "to_watch",
            Self::Watching => "watching",
            Self::Finished => "finished",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "to_watch" => Some(Self::ToWatch),
            "watching" => Some(Self::Watching),
            "finished" => Some(Self::Finished),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WatchError {
    InvalidProgress,
    InvalidRating,
    InvalidWatchDates,
    ProgressResetRequired,
    RewatchRequiresCompletion,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub total_episodes: Option<i64>,
    pub watched_episodes: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub watched_episodes: i64,
    pub status: Status,
    pub started_on: Option<String>,
    pub finished_on: Option<String>,
    pub completed: bool,
    pub active: bool,
}

impl Session {
    pub fn apply_progress(&mut self, progress: &Progress) -> Result<(), WatchError> {
        const LIMIT: i64 = u32::MAX as i64;
        let Progress {
            total_episodes,
            watched_episodes,
        } = *progress;
        if total_episodes.is_some_and(|total| !(1..=LIMIT).contains(&total))
            || !(0..=total_episodes.unwrap_or(LIMIT)).contains(&watched_episodes)
        {
            return Err(WatchError::InvalidProgress);
        }
        self.watched_episodes = watched_episodes;
        let status = match (watched_episodes, total_episodes) {
            (0, _) => Status::ToWatch,
            (seen, Some(total)) if seen == total => Status::Finished,
            _ => Status::Watching,
        };
        self.record_status(status);
        Ok(())
    }

    pub fn set_status(
        &mut self,
        status: Status,
        total: Option<i64>,
        reset_progress: bool,
    ) -> Result<(), WatchError> {
        match status {
            Status::ToWatch if self.watched_episodes > 0 && !reset_progress => {
                return Err(WatchError::ProgressResetRequired);
            }
            Status::ToWatch => self.watched_episodes = 0,
            Status::Finished => self.watched_episodes = total.unwrap_or(self.watched_episodes),
            Status::Watching => {}
        }
        self.record_status(status);
        Ok(())
    }

    fn record_status(&mut self, status: Status) {
        self.status = status;
        self.completed |= status == Status::Finished;
    }

    pub fn check_rewatch(&self) -> Result<(), WatchError> {
        if !self.completed {
            return Err(WatchError::RewatchRequiresCompletion);
        }
        Ok(())
    }
}

pub fn validate_rating(rating: Option<i64>) -> Result<(), WatchError> {
    if rating.is_some_and(|value| !(10..=100).contains(&value)) {
        return Err(WatchError::InvalidRating);
    }
    Ok(())
}

pub fn validate_dates(started: Option<&str>, finished: Option<&str>) -> Result<(), WatchError> {
    if started.is_some_and(|date| !valid_date(date))
        || finished.is_some_and(|date| !valid_date(date))
        || matches!((started, finished), (Some(start), Some(end)) if start > end)
    {
        return Err(WatchError::InvalidWatchDates);
    }
    Ok(())
}

fn valid_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = date[..4].parse().expect("four ASCII digits");
    let month: u32 = date[5..7].parse().expect("two ASCII digits");
    let day: u32 = date[8..].parse().expect("two ASCII digits");
    let days = match month {
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    year > 0 && (1..=days).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial() -> Session {
        Session {
            id: 1,
            watched_episodes: 0,
            status: Status::ToWatch,
            started_on: None,
            finished_on: None,
            completed: false,
            active: true,
        }
    }

    #[test]
    fn automatic_state_handles_unknown_totals_and_completion() {
        let mut session = initial();
        for (seen, total, expected) in [
            (0, Some(8), Status::ToWatch),
            (1, None, Status::Watching),
            (3, Some(8), Status::Watching),
            (8, Some(8), Status::Finished),
            (7, Some(8), Status::Watching),
            (8, Some(8), Status::Finished),
        ] {
            session
                .apply_progress(&Progress {
                    total_episodes: total,
                    watched_episodes: seen,
                })
                .unwrap();
            assert_eq!(session.status, expected);
        }
        assert!(session.completed);
        for (seen, total) in [
            (-1, None),
            (9, Some(8)),
            (0, Some(0)),
            (2, Some(1)),
            (i64::MAX, None),
        ] {
            assert_eq!(
                session.apply_progress(&Progress {
                    total_episodes: total,
                    watched_episodes: seen
                }),
                Err(WatchError::InvalidProgress)
            );
            assert_eq!(session.watched_episodes, 8);
        }
    }

    #[test]
    fn manual_status_preserves_progress_requires_reset_and_keeps_completion() {
        let mut session = initial();
        assert_eq!(
            session.check_rewatch(),
            Err(WatchError::RewatchRequiresCompletion)
        );
        session.set_status(Status::Watching, None, false).unwrap();
        assert_eq!(session.watched_episodes, 0);
        session
            .apply_progress(&Progress {
                total_episodes: None,
                watched_episodes: 4,
            })
            .unwrap();
        session.set_status(Status::Finished, None, false).unwrap();
        assert_eq!(session.watched_episodes, 4);
        session
            .set_status(Status::Watching, Some(8), false)
            .unwrap();
        assert_eq!(session.watched_episodes, 4);
        session
            .set_status(Status::Finished, Some(8), false)
            .unwrap();
        assert_eq!(session.watched_episodes, 8);
        assert_eq!(
            session.set_status(Status::ToWatch, Some(8), false),
            Err(WatchError::ProgressResetRequired)
        );
        assert_eq!(session.status, Status::Finished);
        session.set_status(Status::ToWatch, Some(8), true).unwrap();
        assert_eq!(session.watched_episodes, 0);
        assert!(session.completed);
    }

    #[test]
    fn validates_calendar_dates_and_nullable_rating_tenths() {
        for invalid in [
            "2025-02-29",
            "1900-02-29",
            "0000-01-01",
            "2024-04-31",
            "2024-2-01",
            "٢٠٢٤-02-01",
        ] {
            assert_eq!(
                validate_dates(Some(invalid), None),
                Err(WatchError::InvalidWatchDates)
            );
        }
        assert!(validate_dates(None, Some("2000-02-29")).is_ok());
        assert!(validate_dates(Some("2024-02-29"), Some("2024-02-29")).is_ok());
        assert_eq!(
            validate_dates(Some("2024-03-01"), Some("2024-02-29")),
            Err(WatchError::InvalidWatchDates)
        );
        for rating in [None, Some(10), Some(87), Some(100)] {
            assert!(validate_rating(rating).is_ok());
        }
        for rating in [Some(-1), Some(9), Some(101)] {
            assert_eq!(validate_rating(rating), Err(WatchError::InvalidRating));
        }
    }
}
