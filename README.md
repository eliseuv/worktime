# WorkTime Tracker

A terminal-based work time tracking utility written in Rust. It tracks daily work intervals, calculates elapsed time, and provides system notifications when configurable work duration thresholds are met.

## Features

- TUI interface for tracking "In" and "Out" time intervals.
- Integrated interactive Options menu for real-time configuration without editing files manually.
- Stores history persistently using SQLite.
- Fully configurable notification intervals and messages.
- Customizable theme via hex color codes.
- Fluid navigation and editing of tracked time entries.

## Installation

Build from source using Cargo:

```bash
cargo build --release
```

The compiled binary will be placed in `target/release/worktime`. To install it to your local cargo binary directory:

```bash
cargo install --path .
```

## Configurability

WorkTime Tracker places a strong emphasis on user configurability. Configuration is managed through a YAML file, but can also be dynamically edited within the application by pressing `o` to open the interactive options menu.

Depending on your OS, the configuration file is located at:
- **Linux:** `~/.config/WorkTimeTracker/config.yaml`
- **Windows:** `%APPDATA%\WorkTimeTracker\config\config.yaml`
- **macOS:** `~/Library/Application Support/WorkTimeTracker/config.yaml`

### Configuration Sections

The configuration is broken down into four primary sections:

1. **Times:** Set your daily `total_time_hours` and the `overtime_threshold_minutes` for the history display.
2. **Notifications:** Customize the `done_message` and define an arbitrary list of interval notifications. Each interval consists of a time (in minutes remaining) and a custom message. You can use `{mins}` as a template variable in the message string.
3. **Database:** Specify a custom `db_path` if you wish to override the default SQLite storage location.
4. **Themes:** Redefine the application's entire color palette using standard hex color codes.

### Sample `config.yaml`

```yaml
times:
  total_time_hours: 8.0
  overtime_threshold_minutes: 10

notifications:
  done_message: "Workday complete."
  intervals:
    - minutes: 30
      message: "{mins} minutes remaining."
    - minutes: 10
      message: "Only {mins} minutes left."

database:
  path: null # Leave null to use the OS default path

themes:
  title: "#7dcfff"
  text: "#c0caf5"
  subtext: "#565f89"
  border: "#565f89"
  highlight: "#e0af68"
  in_state: "#9ece6a"
  out_state: "#f7768e"
```

## Usage

Start the application by running `worktime`.

### Controls

- **Clock In / Out:** Type the time in `HH:MM` format (e.g., `09:30`) and press `Enter`. The colon is optional (e.g., `930`).
- **Edit Entries:** Use `Up` and `Down` arrow keys to select a tracked entry, type a new time, and press `Enter`.
- **Delete Entries:** Press `Delete` to remove a selected entry, or press `Delete` while no entry is selected to remove the most recent one.
- **Options Menu:** Press `o` to open the configuration editor. Use arrow keys or `Enter` to navigate between fields, and `Delete` to remove custom notification intervals. Press `Ctrl+S` to save changes or `Esc` to close without saving.
- **Exit:** Press `Ctrl+D` to save the current session and exit the application. `Esc` can also be used as a universal back button to clear inputs or exit.

## Database Layout

The application relies on SQLite for persistent storage.

### Schema

The database uses a single table named `time_log` with the following layout:

| Column | Type | Description |
| :--- | :--- | :--- |
| `id` | `INTEGER PRIMARY KEY AUTOINCREMENT` | Unique identifier for the entry. |
| `time` | `TEXT NOT NULL` | The timestamp stored as an ISO 8601 RFC3339 string (e.g., `2026-06-15T09:00:00-03:00`). |
| `entry_type` | `TEXT NOT NULL` | Indicates the type of entry, limited to `"In"` or `"Out"`. |

### Data Manipulation

Instead of executing targeted updates for individual row edits, the application handles data manipulation by performing bulk synchronizations for active dates. 

When you exit the application or trigger a save, the process is as follows:
1. **Identify Affected Dates:** The application scans the current session's loaded entries to determine which unique calendar dates have been modified or interacted with.
2. **Clear Existing Data:** For each affected date, it deletes all existing records within that 24-hour period (`DELETE FROM time_log WHERE time >= start_of_day AND time <= end_of_day`).
3. **Re-insert:** It inserts the current, strictly-ordered entries back into the database.

This approach guarantees that the state of the database perfectly mirrors the state of the application's timeline at the time of saving, cleanly avoiding issues related to out-of-order edits, orphaned entries, or duplicate records.

## License

MIT
