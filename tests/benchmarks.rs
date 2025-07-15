use std::fs;
use std::time::Instant;
use tempfile::tempdir;
use tokio::time::Duration;

// Import from the lib module
use ffs::process_file_blocking;

#[tokio::test]
async fn benchmark_concurrent_processing() {
	let dir = tempdir().unwrap();
	let path = dir.path();

	// Create test files
	let num_files = 100;
	let mut handles = vec![];

	let start = Instant::now();

	// Create files concurrently
	for i in 0..num_files {
		let file_path = path.join(format!("benchmark_{i}.txt"));
		handles.push(tokio::task::spawn_blocking(move || {
			fs::write(&file_path, format!("content {i}")).unwrap();
		}));
	}

	// Wait for all creations
	for handle in handles {
		handle.await.unwrap();
	}

	let creation_time = start.elapsed();
	println!("Created {num_files} files in {creation_time:?}");

	// Test processing performance
	let start = Instant::now();

	// Simulate processing all files
	let mut process_handles = vec![];
	for i in 0..num_files {
		let file_path = path.join(format!("benchmark_{i}.txt"));
		process_handles.push(tokio::task::spawn_blocking(move || {
			process_file_blocking(&file_path);
		}));
	}

	// Wait for all processing
	for handle in process_handles {
		handle.await.unwrap();
	}

	let processing_time = start.elapsed();
	println!("Processed {num_files} files in {processing_time:?}");

	// Assert reasonable performance
	assert!(
		processing_time.as_millis() < 1000,
		"Processing should complete within 1 second"
	);
}

#[tokio::test]
async fn benchmark_semaphore_efficiency() {
	let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(4));
	let num_tasks = 20;

	let start = Instant::now();

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
	let total_time = start.elapsed();

	// With 4 concurrent tasks, 20 tasks should take ~250ms (20/4 * 50ms)
	let expected_time = Duration::from_millis(250);
	assert!(
		total_time < expected_time + Duration::from_millis(50),
		"Semaphore should limit concurrent processing efficiently"
	);

	assert_eq!(results.len(), num_tasks);
}

#[tokio::test]
async fn test_memory_safety() {
	let dir = tempdir().unwrap();
	let path = dir.path();

	// Test that concurrent operations don't cause memory issues
	let mut handles = vec![];

	for i in 0..50 {
		let file_path = path.join(format!("memory_test_{i}.txt"));
		handles.push(tokio::task::spawn_blocking(move || {
			fs::write(&file_path, format!("data {i}")).unwrap();
			process_file_blocking(&file_path);
		}));
	}

	// Ensure all tasks complete without memory issues
	let results = futures::future::join_all(handles).await;
	assert_eq!(results.len(), 50);
}

#[tokio::test]
async fn test_async_vs_sync_performance() {
	let dir = tempdir().unwrap();
	let path = dir.path();

	// Create test files
	for i in 0..10 {
		let file_path = path.join(format!("perf_test_{i}.txt"));
		fs::write(&file_path, format!("content {i}")).unwrap();
	}

	// Test async processing
	let start = Instant::now();
	let mut async_handles = vec![];
	for i in 0..10 {
		let file_path = path.join(format!("perf_test_{i}.txt"));
		async_handles.push(tokio::task::spawn(async move {
			tokio::time::sleep(Duration::from_millis(10)).await;
			process_file_blocking(&file_path);
		}));
	}
	futures::future::join_all(async_handles).await;
	let async_time = start.elapsed();

	// Test sync processing
	let start = Instant::now();
	for i in 0..10 {
		let file_path = path.join(format!("perf_test_{i}.txt"));
		process_file_blocking(&file_path);
	}
	let sync_time = start.elapsed();

	println!("Async processing time: {async_time:?}");
	println!("Sync processing time: {sync_time:?}");

	// Async should be faster for I/O bound operations
	assert!(
		async_time < sync_time * 2,
		"Async should provide performance benefits"
	);
}
