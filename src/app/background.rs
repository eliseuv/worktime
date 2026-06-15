use crate::state::AppState;
use chrono::Local;
use notify_rust::Notification;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration as StdDuration,
};

pub fn start_background_thread(state: Arc<Mutex<AppState>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(StdDuration::from_secs(1));
            let now = Local::now();
            let mut s = state.lock().unwrap();

            let worked = s.calculate_worked_time(now);
            let worked_minutes = worked.num_minutes();
            let total_time_minutes = (s.config.times.total_time_hours * 60.0) as i64;
            let remaining_minutes = total_time_minutes - worked_minutes;

            if remaining_minutes <= 0 && !s.notified_done {
                let body = &s.config.notifications.done_message;
                let _ = Notification::new()
                    .summary("WorkTime Alert")
                    .body(body)
                    .show();
                s.notified_done = true;
            }

            let intervals = s.config.notifications.intervals.clone();
            for interval in intervals {
                if remaining_minutes <= interval.minutes
                    && remaining_minutes > 0
                    && !s.notified_intervals.contains(&interval.minutes)
                {
                    let body = interval.message.replace("{mins}", &interval.minutes.to_string());
                    let _ = Notification::new()
                        .summary("WorkTime Alert")
                        .body(&body)
                        .show();
                    s.notified_intervals.insert(interval.minutes);
                }
            }
        }
    });
}
