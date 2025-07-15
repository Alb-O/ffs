use colored::*;
use ffs::watch;
use std::io::Write;

fn init_logger() {
	use std::sync::Once;
	static INIT: Once = Once::new();

	INIT.call_once(|| {
		env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
			.format(|buf, record| {
				let level = record.level();
				let message = record.args();

				let colored_level = match level {
					log::Level::Error => format!("[{}]", "ERROR".red().bold()),
					log::Level::Warn => format!("[{}]", "WARN".yellow().bold()),
					log::Level::Info => format!("[{}]", "INFO".green().bold()),
					log::Level::Debug => format!("[{}]", "DEBUG".blue().bold()),
					log::Level::Trace => format!("[{}]", "TRACE".purple().bold()),
				};

				writeln!(buf, "{colored_level} {message}")
			})
			.init();
	});
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	init_logger();

	let path = std::env::args()
		.nth(1)
		.expect("Argument 1 needs to be a path");

	log::info!("Watching {}", path.underline());

	if let Err(error) = watch(path).await {
		log::error!("Error: {error}");
	}

	Ok(())
}
