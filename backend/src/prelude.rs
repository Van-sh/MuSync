use chrono::Duration;

pub trait ToTimeDuration {
    fn to_time_duration(&self) -> time::Duration;
}

impl ToTimeDuration for Duration {
    fn to_time_duration(&self) -> time::Duration {
        self.to_std()
            .expect("Something went wrong: Cookie duration should be positive")
            .try_into()
            .expect("Something went wrong while handling Cookie duration")
    }
}
