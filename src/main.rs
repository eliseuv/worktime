pub mod config;
pub mod db;
pub mod state;
pub mod ui;
pub mod app;

use app::App;

fn main() -> std::io::Result<()> {
    let mut app = App::new()?;
    app.main_loop()?;
    app.shutdown()?;
    Ok(())
}
