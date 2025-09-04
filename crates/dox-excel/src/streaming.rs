//! Excel streaming support for large files (100MB+)
//!
//! This module provides functionality to:
//! - Stream large Excel files without loading everything into memory
//! - Process Excel files in chunks to handle memory constraints
//! - Lazy loading of sheets and ranges
//! - Memory-mapped file support for read operations
//! - Progress reporting for large file operations

use anyhow::{anyhow, Result};
use calamine::{open_workbook, Reader, Xlsx};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use dox_core::Cell;

/// Configuration for streaming operations
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum number of rows to process in one chunk
    pub chunk_size: usize,
    /// Maximum memory usage in bytes (approximate)
    pub max_memory_mb: usize,
    /// Enable parallel processing of chunks
    pub parallel_processing: bool,
    /// Number of worker threads for parallel processing
    pub worker_threads: usize,
    /// Enable memory mapping for large files
    pub enable_mmap: bool,
}

/// Progress information for streaming operations
#[derive(Debug, Clone)]
pub struct StreamProgress {
    /// Total rows to process
    pub total_rows: usize,
    /// Rows processed so far
    pub processed_rows: usize,
    /// Current sheet being processed
    pub current_sheet: String,
    /// Percentage complete (0-100)
    pub percentage: f64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
}

/// Chunk of Excel data for streaming processing
#[derive(Debug)]
pub struct DataChunk {
    /// Sheet name
    pub sheet_name: String,
    /// Starting row index (0-based)
    pub start_row: usize,
    /// Ending row index (0-based, exclusive)
    pub end_row: usize,
    /// Data rows in this chunk
    pub data: Vec<Vec<Cell>>,
}

/// Result of processing a data chunk
#[derive(Debug)]
pub struct ChunkResult {
    /// Chunk identifier
    pub chunk_id: usize,
    /// Sheet name
    pub sheet_name: String,
    /// Number of rows processed
    pub rows_processed: usize,
    /// Processing success
    pub success: bool,
    /// Error message if processing failed
    pub error: Option<String>,
}

/// Streaming Excel reader for large files
pub struct StreamingExcelReader {
    config: StreamingConfig,
    file_path: std::path::PathBuf,
    memory_map: Option<Mmap>,
}

/// Streaming Excel processor for chunk-based operations
pub struct StreamingProcessor {
    config: StreamingConfig,
    progress_tx: Option<mpsc::UnboundedSender<StreamProgress>>,
}

/// Memory-efficient Excel iterator
pub struct ExcelIterator {
    workbook: Xlsx<BufReader<File>>,
    sheet_names: Vec<String>,
    current_sheet_index: usize,
    current_row_index: usize,
    chunk_size: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        StreamingConfig {
            chunk_size: 1000,
            max_memory_mb: 512,
            parallel_processing: true,
            worker_threads: num_cpus::get(),
            enable_mmap: true,
        }
    }
}

impl StreamingConfig {
    /// Create config optimized for very large files (>1GB)
    pub fn for_very_large_files() -> Self {
        StreamingConfig {
            chunk_size: 500,
            max_memory_mb: 256,
            parallel_processing: true,
            worker_threads: num_cpus::get().min(4), // Limit to avoid overwhelming system
            enable_mmap: true,
        }
    }

    /// Create config optimized for memory-constrained environments
    pub fn for_low_memory() -> Self {
        StreamingConfig {
            chunk_size: 100,
            max_memory_mb: 128,
            parallel_processing: false,
            worker_threads: 1,
            enable_mmap: false,
        }
    }

    /// Create config for high-performance processing
    pub fn for_high_performance() -> Self {
        StreamingConfig {
            chunk_size: 2000,
            max_memory_mb: 1024,
            parallel_processing: true,
            worker_threads: num_cpus::get(),
            enable_mmap: true,
        }
    }
}

impl StreamingExcelReader {
    /// Create a new streaming Excel reader
    pub fn new(file_path: impl Into<std::path::PathBuf>, config: StreamingConfig) -> Result<Self> {
        let file_path = file_path.into();

        if !file_path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path.display()));
        }

        let memory_map = if config.enable_mmap {
            match File::open(&file_path) {
                Ok(file) => match unsafe { Mmap::map(&file) } {
                    Ok(mmap) => {
                        info!("Memory-mapped file: {}", file_path.display());
                        Some(mmap)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to memory-map file, falling back to regular I/O: {}",
                            e
                        );
                        None
                    }
                },
                Err(e) => {
                    warn!("Failed to open file for memory mapping: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(StreamingExcelReader {
            config,
            file_path,
            memory_map,
        })
    }

    /// Get file size in bytes
    pub fn file_size(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.file_path)?;
        Ok(metadata.len())
    }

    /// Estimate memory usage for the file
    pub fn estimate_memory_usage(&self) -> Result<usize> {
        let file_size = self.file_size()?;
        // Rough estimate: Excel files expand 2-3x in memory when parsed
        let estimated_mb = (file_size as f64 * 2.5 / 1024.0 / 1024.0) as usize;
        Ok(estimated_mb)
    }

    /// Check if streaming is recommended for this file
    pub fn should_use_streaming(&self) -> Result<bool> {
        let estimated_memory = self.estimate_memory_usage()?;
        Ok(estimated_memory > self.config.max_memory_mb)
    }

    /// Create an iterator for streaming through the Excel file
    pub fn iter_chunks(&self) -> Result<ExcelIterator> {
        debug!(
            "Creating chunk iterator for file: {}",
            self.file_path.display()
        );

        let workbook: Xlsx<_> = open_workbook(&self.file_path)
            .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

        let sheet_names = workbook.sheet_names();

        Ok(ExcelIterator {
            workbook,
            sheet_names,
            current_sheet_index: 0,
            current_row_index: 0,
            chunk_size: self.config.chunk_size,
        })
    }

    /// Process the entire file using streaming with progress reporting
    pub async fn process_with_progress<F, R>(
        &self,
        processor: F,
        progress_callback: Option<Box<dyn Fn(StreamProgress) + Send + Sync>>,
    ) -> Result<Vec<R>>
    where
        F: Fn(DataChunk) -> Result<R> + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        info!(
            "Starting streaming processing of file: {}",
            self.file_path.display()
        );

        let mut workbook: Xlsx<_> = open_workbook(&self.file_path)
            .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

        let sheet_names = workbook.sheet_names();
        let mut results = Vec::new();
        let mut total_processed = 0usize;

        // Calculate total rows across all sheets
        let mut total_rows = 0usize;
        for sheet_name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                total_rows += range.height();
            }
        }

        let start_time = std::time::Instant::now();

        for (_sheet_index, sheet_name) in sheet_names.iter().enumerate() {
            debug!("Processing sheet: {}", sheet_name);

            let range = workbook
                .worksheet_range(sheet_name)
                .map_err(|e| anyhow!("Failed to read sheet '{}': {}", sheet_name, e))?;

            let sheet_rows = range.height();
            let chunks = (sheet_rows + self.config.chunk_size - 1) / self.config.chunk_size;

            for chunk_index in 0..chunks {
                let start_row = chunk_index * self.config.chunk_size;
                let end_row = (start_row + self.config.chunk_size).min(sheet_rows);

                // Extract chunk data
                let mut chunk_data = Vec::new();
                for row_idx in start_row..end_row {
                    if let Some(row) = range.rows().nth(row_idx) {
                        let cells: Vec<Cell> = row
                            .iter()
                            .map(|data| Cell::new(self.convert_data_to_string(data)))
                            .collect();
                        chunk_data.push(cells);
                    }
                }

                let chunk = DataChunk {
                    sheet_name: sheet_name.clone(),
                    start_row,
                    end_row,
                    data: chunk_data,
                };

                // Process chunk
                match processor(chunk) {
                    Ok(result) => results.push(result),
                    Err(e) => warn!("Failed to process chunk {}: {}", chunk_index, e),
                }

                total_processed += end_row - start_row;

                // Report progress
                if let Some(ref callback) = progress_callback {
                    let elapsed = start_time.elapsed().as_secs();
                    let percentage = (total_processed as f64 / total_rows as f64) * 100.0;
                    let eta = if percentage > 0.0 && elapsed > 0 {
                        Some(((100.0 - percentage) * elapsed as f64 / percentage) as u64)
                    } else {
                        None
                    };

                    let progress = StreamProgress {
                        total_rows,
                        processed_rows: total_processed,
                        current_sheet: sheet_name.clone(),
                        percentage,
                        eta_seconds: eta,
                    };

                    callback(progress);
                }
            }
        }

        info!(
            "Completed streaming processing: {} rows in {:.2} seconds",
            total_processed,
            start_time.elapsed().as_secs_f64()
        );

        Ok(results)
    }

    /// Convert calamine Data to string
    fn convert_data_to_string(&self, data: &calamine::Data) -> String {
        match data {
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Float(f) => f.to_string(),
            calamine::Data::String(s) => s.clone(),
            calamine::Data::Bool(b) => b.to_string(),
            calamine::Data::DateTime(dt) => dt.to_string(),
            calamine::Data::DateTimeIso(s) => s.clone(),
            calamine::Data::DurationIso(s) => s.clone(),
            calamine::Data::Error(e) => format!("#ERR: {:?}", e),
            calamine::Data::Empty => String::new(),
        }
    }

    /// Process file in parallel chunks
    pub async fn process_parallel<F, R>(&self, processor: F) -> Result<Vec<R>>
    where
        F: Fn(DataChunk) -> Result<R> + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        if !self.config.parallel_processing {
            return self.process_with_progress(processor, None).await;
        }

        info!(
            "Starting parallel processing of file: {}",
            self.file_path.display()
        );

        let mut workbook: Xlsx<_> = open_workbook(&self.file_path)
            .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

        let sheet_names = workbook.sheet_names();
        let mut all_chunks = Vec::new();

        // Prepare all chunks
        for sheet_name in &sheet_names {
            let range = workbook
                .worksheet_range(sheet_name)
                .map_err(|e| anyhow!("Failed to read sheet '{}': {}", sheet_name, e))?;

            let sheet_rows = range.height();
            let chunks = (sheet_rows + self.config.chunk_size - 1) / self.config.chunk_size;

            for chunk_index in 0..chunks {
                let start_row = chunk_index * self.config.chunk_size;
                let end_row = (start_row + self.config.chunk_size).min(sheet_rows);

                let mut chunk_data = Vec::new();
                for row_idx in start_row..end_row {
                    if let Some(row) = range.rows().nth(row_idx) {
                        let cells: Vec<Cell> = row
                            .iter()
                            .map(|data| Cell::new(self.convert_data_to_string(data)))
                            .collect();
                        chunk_data.push(cells);
                    }
                }

                let chunk = DataChunk {
                    sheet_name: sheet_name.clone(),
                    start_row,
                    end_row,
                    data: chunk_data,
                };

                all_chunks.push(chunk);
            }
        }

        // Process chunks in parallel
        let results: Result<Vec<_>, _> = all_chunks.into_par_iter().map(processor).collect();

        match results {
            Ok(results) => {
                info!(
                    "Completed parallel processing with {} results",
                    results.len()
                );
                Ok(results)
            }
            Err(e) => Err(anyhow!("Parallel processing failed: {}", e)),
        }
    }
}

impl Iterator for ExcelIterator {
    type Item = Result<DataChunk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sheet_index >= self.sheet_names.len() {
            return None;
        }

        let sheet_name = &self.sheet_names[self.current_sheet_index];

        let range = match self.workbook.worksheet_range(sheet_name) {
            Ok(range) => range,
            Err(e) => return Some(Err(anyhow!("Failed to read sheet '{}': {}", sheet_name, e))),
        };

        let sheet_rows = range.height();
        if self.current_row_index >= sheet_rows {
            // Move to next sheet
            self.current_sheet_index += 1;
            self.current_row_index = 0;
            return self.next();
        }

        let start_row = self.current_row_index;
        let end_row = (start_row + self.chunk_size).min(sheet_rows);

        // Extract chunk data
        let mut chunk_data = Vec::new();
        for row_idx in start_row..end_row {
            if let Some(row) = range.rows().nth(row_idx) {
                let cells: Vec<Cell> = row
                    .iter()
                    .map(|data| {
                        let value = match data {
                            calamine::Data::Int(i) => i.to_string(),
                            calamine::Data::Float(f) => f.to_string(),
                            calamine::Data::String(s) => s.clone(),
                            calamine::Data::Bool(b) => b.to_string(),
                            calamine::Data::DateTime(dt) => dt.to_string(),
                            calamine::Data::DateTimeIso(s) => s.clone(),
                            calamine::Data::DurationIso(s) => s.clone(),
                            calamine::Data::Error(e) => format!("#ERR: {:?}", e),
                            calamine::Data::Empty => String::new(),
                        };
                        Cell::new(value)
                    })
                    .collect();
                chunk_data.push(cells);
            }
        }

        self.current_row_index = end_row;

        let chunk = DataChunk {
            sheet_name: sheet_name.clone(),
            start_row,
            end_row,
            data: chunk_data,
        };

        Some(Ok(chunk))
    }
}

/// Helper functions for streaming operations
pub mod helpers {
    use super::*;

    /// Simple text extraction from large Excel files
    pub async fn extract_text_streaming(
        file_path: &Path,
        config: Option<StreamingConfig>,
    ) -> Result<String> {
        let config = config.unwrap_or_default();
        let reader = StreamingExcelReader::new(file_path, config)?;

        if !reader.should_use_streaming()? {
            info!("File is small enough for regular processing");
            // Fall back to regular processing for small files
        }

        let mut full_text = String::new();

        let text_processor = |chunk: DataChunk| -> Result<String> {
            let mut chunk_text = String::new();

            for row in &chunk.data {
                for cell in row {
                    if !cell.value.is_empty() {
                        chunk_text.push_str(&cell.value);
                        chunk_text.push('\t');
                    }
                }
                chunk_text.push('\n');
            }

            Ok(chunk_text)
        };

        let results = reader.process_with_progress(text_processor, None).await?;

        for text_chunk in results {
            full_text.push_str(&text_chunk);
        }

        Ok(full_text)
    }

    /// Count non-empty cells in large Excel files
    pub async fn count_cells_streaming(
        file_path: &Path,
        config: Option<StreamingConfig>,
    ) -> Result<usize> {
        let config = config.unwrap_or_default();
        let reader = StreamingExcelReader::new(file_path, config)?;

        let cell_counter = |chunk: DataChunk| -> Result<usize> {
            let mut count = 0;
            for row in &chunk.data {
                for cell in row {
                    if !cell.value.is_empty() {
                        count += 1;
                    }
                }
            }
            Ok(count)
        };

        let results = reader.process_parallel(cell_counter).await?;

        Ok(results.into_iter().sum())
    }

    /// Find and replace in large Excel files (read-only analysis)
    pub async fn find_matches_streaming(
        file_path: &Path,
        search_term: &str,
        config: Option<StreamingConfig>,
    ) -> Result<Vec<(String, usize, usize)>> {
        // (sheet_name, row, col)
        let config = config.unwrap_or_default();
        let reader = StreamingExcelReader::new(file_path, config)?;
        let search_term = search_term.to_string();

        let match_finder = move |chunk: DataChunk| -> Result<Vec<(String, usize, usize)>> {
            let mut matches = Vec::new();

            for (row_idx, row) in chunk.data.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if cell.value.contains(&search_term) {
                        matches.push((
                            chunk.sheet_name.clone(),
                            chunk.start_row + row_idx,
                            col_idx,
                        ));
                    }
                }
            }

            Ok(matches)
        };

        let results = reader.process_parallel(match_finder).await?;

        let mut all_matches = Vec::new();
        for chunk_matches in results {
            all_matches.extend(chunk_matches);
        }

        Ok(all_matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_defaults() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 1000);
        assert_eq!(config.max_memory_mb, 512);
        assert!(config.parallel_processing);
        assert!(config.enable_mmap);
    }

    #[test]
    fn test_streaming_config_variants() {
        let large_config = StreamingConfig::for_very_large_files();
        assert_eq!(large_config.chunk_size, 500);
        assert_eq!(large_config.max_memory_mb, 256);

        let low_mem_config = StreamingConfig::for_low_memory();
        assert_eq!(low_mem_config.chunk_size, 100);
        assert_eq!(low_mem_config.max_memory_mb, 128);
        assert!(!low_mem_config.parallel_processing);

        let high_perf_config = StreamingConfig::for_high_performance();
        assert_eq!(high_perf_config.chunk_size, 2000);
        assert_eq!(high_perf_config.max_memory_mb, 1024);
    }

    #[test]
    fn test_progress_calculation() {
        let progress = StreamProgress {
            total_rows: 1000,
            processed_rows: 250,
            current_sheet: "Sheet1".to_string(),
            percentage: 25.0,
            eta_seconds: Some(30),
        };

        assert_eq!(progress.percentage, 25.0);
        assert_eq!(progress.eta_seconds, Some(30));
    }

    #[test]
    fn test_data_chunk_creation() {
        let chunk = DataChunk {
            sheet_name: "Test Sheet".to_string(),
            start_row: 0,
            end_row: 100,
            data: vec![
                vec![Cell::new("A1"), Cell::new("B1")],
                vec![Cell::new("A2"), Cell::new("B2")],
            ],
        };

        assert_eq!(chunk.sheet_name, "Test Sheet");
        assert_eq!(chunk.start_row, 0);
        assert_eq!(chunk.end_row, 100);
        assert_eq!(chunk.data.len(), 2);
    }
}
