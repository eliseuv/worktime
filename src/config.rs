use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub text: String,
    pub border: String,
    pub title: String,
    pub highlight: String,
    pub in_state: String,
    pub out_state: String,
    pub subtext: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            text: "#c0caf5".into(),
            border: "#565f89".into(),
            title: "#7dcfff".into(),
            highlight: "#e0af68".into(),
            in_state: "#9ece6a".into(),
            out_state: "#f7768e".into(),
            subtext: "#565f89".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotificationInterval {
    pub minutes: i64,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotificationConfig {
    pub done_message: String,
    pub intervals: Vec<NotificationInterval>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            done_message: "Go home".into(),
            intervals: vec![
                NotificationInterval {
                    minutes: 30,
                    message: "Only {mins} minutes remaining".into(),
                },
                NotificationInterval {
                    minutes: 10,
                    message: "Hit the point!".into(),
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimesConfig {
    pub total_time_hours: f64,
    #[serde(default = "default_overtime_threshold")]
    pub overtime_threshold_minutes: i64,
}

impl Default for TimesConfig {
    fn default() -> Self {
        Self {
            total_time_hours: 8.0,
            overtime_threshold_minutes: 10,
        }
    }
}

fn default_overtime_threshold() -> i64 {
    10
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub times: TimesConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub themes: ThemeConfig,
}



impl AppConfig {
    pub fn get_config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "WorkTime", "WorkTimeTracker") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            Some(config_dir.join("config.yaml"))
        } else {
            None
        }
    }

    pub fn get_db_path(&self) -> PathBuf {
        if let Some(path) = &self.database.path {
            PathBuf::from(path)
        } else if let Some(proj_dirs) = ProjectDirs::from("com", "WorkTime", "WorkTimeTracker") {
            let data_dir = proj_dirs.data_dir();
            if !data_dir.exists() {
                let _ = fs::create_dir_all(data_dir);
            }
            data_dir.join("worktime.db")
        } else {
            PathBuf::from("worktime.db")
        }
    }

    pub fn load_or_default() -> Self {
        if let Some(path) = Self::get_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_yaml::from_str::<AppConfig>(&content) {
                        // Write back to disk to populate any newly added default fields
                        if let Ok(yaml) = serde_yaml::to_string(&config) {
                            let _ = fs::write(&path, yaml);
                        }
                        return config;
                    }
                }
            }

            let default_config = Self::default();
            if let Ok(yaml) = serde_yaml::to_string(&default_config) {
                let _ = fs::write(&path, yaml);
            }
            return default_config;
        }

        Self::default()
    }
}
