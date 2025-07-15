use notify::event::Event;
use notify::{Config, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

pub async fn watch<P: AsRef<Path> + Send + 'static>(path: P) -> notify::Result<()> {
	// Create async channel for events
	let (tx, mut rx) = mpsc::channel::<notify::Result<Event>>(100);

	// Create watcher in a blocking task
	let watcher_handle = task::spawn_blocking(move || {
		let mut watcher = notify::recommended_watcher(move |res| {
			// Use blocking_send since we're in a blocking context
			if let Err(e) = tx.blocking_send(res) {
				log::error!("Failed to send event: {e}");
			}
		})
		.expect("Failed to create watcher");

		// Configure the watcher
		watcher
			.configure(Config::default())
			.expect("Failed to configure watcher");

		// Start watching the path
		watcher
			.watch(path.as_ref(), RecursiveMode::Recursive)
			.expect("Failed to watch path");

		// Keep the watcher alive
		loop {
			std::thread::park();
		}
	});

	// Create a semaphore to limit concurrent processing
	let semaphore = Arc::new(tokio::sync::Semaphore::new(num_cpus::get()));

	// Process events in parallel
	while let Some(res) = rx.recv().await {
		match res {
			Ok(event) => {
				let permit = semaphore.clone().acquire_owned().await.unwrap();

				// Spawn a new task for each event to process in parallel
				task::spawn(async move {
					process_event(event).await;
					drop(permit); // Release the semaphore permit
				});
			}
			Err(error) => {
				log::error!("Error: {error:?}");
			}
		}
	}

	// This will never complete in normal operation
	watcher_handle.await.unwrap();

	Ok(())
}

pub async fn process_event(event: Event) {
	// Process the event based on its type
	match event.kind {
		notify::EventKind::Create(_) => {
			for path in &event.paths {
				log::info!("Created: {}", path.display());
			}
		}
		notify::EventKind::Modify(modify_kind) => match modify_kind {
			notify::event::ModifyKind::Name(rename_mode) => {
				// Only log the complete rename relationship when we have both paths
				if let notify::event::RenameMode::Both = rename_mode {
					if event.paths.len() >= 2 {
						let from_path = &event.paths[0];
						let to_path = &event.paths[1];
						log::info!("Rename: {} → {}", from_path.display(), to_path.display());
					}
				}
				// For From/To modes, let the Create/Remove events handle the logging
			}
			_ => {
				for path in &event.paths {
					log::info!("Modified: {}", path.display());
				}
			}
		},
		notify::EventKind::Remove(_) => {
			for path in &event.paths {
				log::info!("Removed: {}", path.display());
			}
		}
		// Filter out Access/Open events (do nothing)
		notify::EventKind::Access(_) | notify::EventKind::Other => {
			// Do not report Access/Open events
		}
		notify::EventKind::Any => {
			// Optionally handle other events if needed
		}
	}

	// Additional parallel processing for each file
	let handles: Vec<_> = event
		.paths
		.iter()
		.map(|path| {
			let path = path.clone();
			task::spawn_blocking(move || process_file_blocking(&path))
		})
		.collect();

	// Wait for all parallel tasks to complete
	for handle in handles {
		if let Err(e) = handle.await {
			log::error!("Task failed: {e}");
		}
	}
}

pub fn process_file_blocking(path: &Path) {
	// Simulate CPU-intensive work
	log::debug!("Processing file: {path:?}");

	// Use rayon for parallel processing if needed
	rayon::scope(|s| {
		s.spawn(|_| {
			// This would be where actual file processing happens
			log::trace!("Parallel processing for: {path:?}");
		});
	});
}
