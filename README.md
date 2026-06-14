# WorkTime Tracker

A simple TUI-based work time tracker written in Rust. It tracks your daily work hours, calculates estimated finish times, and provides system notifications when your workday is complete.

## Features

- Tracks "In" and "Out" times with total hours calculation
- Displays remaining time and estimated finish time
- Stores history persistently using SQLite
- Sends native system notifications based on configurable intervals
- Provides a dashboard for viewing daily history and overtime
- Fully configurable via a YAML file

## Installation

Ensure you have Rust and Cargo installed. Build from source:

```bash
cargo build --release
```

The compiled binary will be available in `target/release/worktime`. You can also install it directly to your cargo bin directory:

```bash
cargo install --path .
```

## Configuration

The application uses a YAML configuration file. If a file does not exist, it will be automatically created with default values at the following location depending on your OS:

- **Linux:** `~/.config/WorkTimeTracker/config.yaml`
- **Windows:** `%APPDATA%\WorkTimeTracker\config\config.yaml`
- **macOS:** `~/Library/Application Support/WorkTimeTracker/config.yaml`

### Configuration Options

- `total_time_hours`: Target number of work hours per day.
- `overtime_threshold_minutes`: Threshold in minutes before displaying an overtime warning in the history pane.
- `notifications`: Custom messages and intervals for system notifications.
- `db_path`: (Optional) Custom path for the SQLite database.
- `theme`: Custom hex color codes for the application's UI.

### Sample `config.yaml`

```yaml
total_time_hours: 8.0
overtime_threshold_minutes: 10

notifications:
  done_message: "Your work hours are up!"
  intervals:
    - minutes: 30
      message: "{mins} minutes remaining."
    - minutes: 10
      message: "Only {mins} minutes left!"

# Optional custom database path
# db_path: "/path/to/your/custom/worktime.db"

theme:
  title: "#7dcfff"
  text: "#c0caf5"
  subtext: "#565f89"
  border: "#565f89"
  highlight: "#e0af68"
  in_state: "#9ece6a"
  out_state: "#f7768e"
```

## Usage

Start the application by running the compiled binary.

### Controls

- **Clock In / Out:** Type the time in `HH:MM` format (e.g., `09:30`) and press `Enter`. The colon is optional (e.g., `930` works).
- **Edit Entries:** Use `Up` and `Down` arrow keys to select an entry, type a new time, and press `Enter`.
- **Delete Entries:** Press `Delete` to remove a selected entry, or press `Delete` while no entry is selected to remove the most recent entry.
- **Exit:** Press `Ctrl+D` to save the current session and exit.

## Database

By default, the SQLite database is stored in your OS's data directory:

- **Linux:** `~/.local/share/WorkTimeTracker/worktime.db`
- **Windows:** `%APPDATA%\WorkTimeTracker\data\worktime.db`
- **macOS:** `~/Library/Application Support/WorkTimeTracker/worktime.db`

## License

MIT
