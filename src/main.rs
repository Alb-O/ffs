use ffs::watch;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
		.format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
		.init();

	let path = std::env::args()
		.nth(1)
		.expect("Argument 1 needs to be a path");

	log::info!("Watching {path}");

	if let Err(error) = watch(path).await {
		log::error!("Error: {error:?}");
	}

	Ok(())
}
