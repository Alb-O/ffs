use std::fs;
use std::path::Path;
use tempfile::tempdir;
use tokio::time::{Duration, timeout};

// Import from the lib module
use ffs::{process_event, process_file_blocking, watch};

#[tokio::test]
async fn test_process_event_async() {
	use notify::event::{Event, EventKind};

	let event = Event {
		kind: EventKind::Create(notify::event::CreateKind::File),
		paths: vec![Path::new("/tmp/test.txt").to_path_buf()],
		attrs: notify::event::EventAttributes::default(),
	};

	// Test that process_event completes without panic
	let handle = tokio::task::spawn(async move {
		process_event(event).await;
	});

	timeout(Duration::from_secs(5), handle)
		.await
		.unwrap()
		.unwrap();
}

#[tokio::test]
async fn test_process_file_blocking() {
	let dir = tempdir().unwrap();
	let file_path = dir.path().join("blocking_test.txt");
	fs::write(&file_path, "test").unwrap();

	// Test blocking processing
	let handle = tokio::task::spawn_blocking(move || {
		process_file_blocking(&file_path);
	});

	timeout(Duration::from_secs(5), handle)
		.await
		.unwrap()
		.unwrap();
}

#[tokio::test]
async fn test_semaphore_limits() {
	let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
	let num_tasks = 20;

	let mut handles = vec![];
	for i in 0..num_tasks {
		let permit = semaphore.clone().acquire_owned().await.unwrap();
		handles.push(tokio::task::spawn(async move {
			tokio::time::sleep(Duration::from_millis(50)).await;
			drop(permit);
			i
		}));
	}

	// Wait for all tasks
	let results = futures::future::join_all(handles).await;
	assert_eq!(results.len(), num_tasks);
}

#[tokio::test]
async fn test_error_handling() {
	let dir = tempdir().unwrap();
	let path = dir.path().to_path_buf();

	// Test error handling with valid directory
	let result = watch(path).await;
	assert!(result.is_ok()); // Should succeed with valid directory
}
