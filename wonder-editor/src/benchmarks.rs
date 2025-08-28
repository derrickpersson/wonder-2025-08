use std::time::{Duration, Instant};
use std::fmt;
use crate::core::text_document::TextDocument;

/// Performance benchmark results for text operations
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub document_size: usize,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub ops_per_sec: f64,
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<25} | {:<12} | {:>8} ops | {:>12.3}ms avg | {:>12.1} ops/sec",
            self.operation,
            format_size(self.document_size),
            self.iterations,
            self.avg_duration.as_secs_f64() * 1000.0,
            self.ops_per_sec
        )
    }
}

/// Format byte sizes in human-readable format
fn format_size(size: usize) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}KB", size as f64 / 1024.0)
    } else {
        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
    }
}

/// Comprehensive performance benchmark suite for TextDocument rope implementation
pub struct PerformanceBenchmark {
    pub results: Vec<BenchmarkResult>,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run comprehensive benchmark suite across multiple document sizes
    pub fn run_full_suite(&mut self) {
        println!("🚀 Starting Wonder Editor Performance Benchmark Suite");
        println!("📊 Testing Rope-based TextDocument performance across multiple sizes\n");

        let sizes = vec![
            1024,         // 1KB - small documents
            100 * 1024,   // 100KB - medium documents  
            1024 * 1024,  // 1MB - large documents
            10 * 1024 * 1024, // 10MB - very large documents
        ];

        for size in sizes {
            println!("🔍 Benchmarking {} documents:", format_size(size));
            self.benchmark_document_size(size);
            println!();
        }

        self.print_summary();
    }

    /// Benchmark all operations for a specific document size
    fn benchmark_document_size(&mut self, size: usize) {
        // Create test document of specified size
        let content = self.create_test_content(size);
        
        // Text insertion benchmarks
        self.benchmark_insert_operations(&content);
        
        // Cursor movement benchmarks  
        self.benchmark_cursor_operations(&content);
        
        // Selection operations benchmarks
        self.benchmark_selection_operations(&content);
        
        // Large edit operations benchmarks
        self.benchmark_large_edit_operations(&content);
    }

    /// Create test content of specified size with realistic text patterns
    fn create_test_content(&self, target_size: usize) -> String {
        let paragraph = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\n";
        
        let mut content = String::new();
        while content.len() < target_size {
            content.push_str(paragraph);
        }
        
        // Truncate to exact size if needed
        if content.len() > target_size {
            content.truncate(target_size);
        }
        
        content
    }

    /// Benchmark text insertion operations
    fn benchmark_insert_operations(&mut self, base_content: &str) {
        let mut doc = TextDocument::with_content(base_content.to_string());
        
        // Single character insertions
        let result = self.benchmark_operation("Single char insert", base_content.len(), 1000, || {
            doc.set_cursor_position(doc.content().chars().count() / 2);
            doc.insert_char('x');
        });
        self.results.push(result);

        // Text block insertions
        let insert_text = "New paragraph inserted here.\n";
        let mut doc = TextDocument::with_content(base_content.to_string());
        let result = self.benchmark_operation("Text block insert", base_content.len(), 100, || {
            doc.set_cursor_position(doc.content().chars().count() / 2);
            doc.insert_text(insert_text);
        });
        self.results.push(result);
    }

    /// Benchmark cursor movement operations
    fn benchmark_cursor_operations(&mut self, base_content: &str) {
        let mut doc = TextDocument::with_content(base_content.to_string());
        
        // Cursor positioning
        let doc_len = doc.content().chars().count();
        let positions: Vec<usize> = (0..1000).map(|i| (i * doc_len) / 1000).collect();
        let mut pos_iter = positions.iter().cycle();
        let result = self.benchmark_operation("Cursor positioning", base_content.len(), 1000, || {
            if let Some(&pos) = pos_iter.next() {
                doc.set_cursor_position(pos);
            }
        });
        self.results.push(result);

        // Line navigation
        let result = self.benchmark_operation("Line navigation", base_content.len(), 500, || {
            doc.move_cursor_down();
            doc.move_cursor_up();
        });
        self.results.push(result);

        // Word navigation  
        let result = self.benchmark_operation("Word navigation", base_content.len(), 500, || {
            doc.move_to_word_end();
            doc.move_to_word_start();
        });
        self.results.push(result);
    }

    /// Benchmark selection operations
    fn benchmark_selection_operations(&mut self, base_content: &str) {
        let mut doc = TextDocument::with_content(base_content.to_string());
        
        // Selection creation and extension
        let result = self.benchmark_operation("Selection operations", base_content.len(), 200, || {
            doc.set_cursor_position(doc.content().chars().count() / 3);
            doc.start_selection();
            doc.extend_selection_right();
            doc.extend_selection_right();
            doc.extend_selection_right();
            doc.clear_selection();
        });
        self.results.push(result);
    }

    /// Benchmark large edit operations
    fn benchmark_large_edit_operations(&mut self, base_content: &str) {
        // Large text replacement
        let mut doc = TextDocument::with_content(base_content.to_string());
        let replacement_text = "REPLACED TEXT ".repeat(100);
        
        let result = self.benchmark_operation("Large text replacement", base_content.len(), 10, || {
            doc.set_cursor_position(doc.content().chars().count() / 2);
            doc.start_selection();
            // Select a chunk to replace
            for _ in 0..50 {
                doc.extend_selection_right();
            }
            doc.insert_text(&replacement_text);
        });
        self.results.push(result);
    }

    /// Execute and time a benchmark operation
    pub fn benchmark_operation<F>(&self, name: &str, doc_size: usize, iterations: usize, mut operation: F) -> BenchmarkResult 
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..10 {
            operation();
        }

        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        let total_duration = start.elapsed();

        let avg_duration = total_duration / iterations as u32;
        let ops_per_sec = iterations as f64 / total_duration.as_secs_f64();

        BenchmarkResult {
            operation: name.to_string(),
            document_size: doc_size,
            iterations,
            total_duration,
            avg_duration,
            ops_per_sec,
        }
    }

    /// Print comprehensive benchmark summary
    pub fn print_summary(&self) {
        println!("📈 PERFORMANCE BENCHMARK RESULTS");
        println!("{}", "=".repeat(80));
        println!("{:<25} | {:<12} | {:>8} | {:>16} | {:>16}", 
            "Operation", "Size", "Ops", "Avg Time", "Ops/Second");
        println!("{}", "-".repeat(80));
        
        for result in &self.results {
            println!("{}", result);
            
            // Check if operation meets performance targets
            let meets_target = result.avg_duration.as_millis() < 10;
            if !meets_target && result.document_size >= 1024 * 1024 {
                println!("  ⚠️  Above 10ms target for large document");
            } else if meets_target {
                println!("  ✅ Within 10ms performance target");
            }
        }
        
        println!("{}", "=".repeat(80));
        self.analyze_performance_gains();
    }

    /// Analyze and report performance characteristics
    fn analyze_performance_gains(&self) {
        println!("\n🎯 PERFORMANCE ANALYSIS:");
        
        // Find worst case performance
        if let Some(worst) = self.results.iter().max_by_key(|r| r.avg_duration) {
            println!("• Worst case: {} took {:.3}ms", worst.operation, worst.avg_duration.as_secs_f64() * 1000.0);
        }
        
        // Find best throughput
        if let Some(best_throughput) = self.results.iter().max_by(|a, b| a.ops_per_sec.partial_cmp(&b.ops_per_sec).unwrap()) {
            println!("• Highest throughput: {} at {:.1} ops/sec", best_throughput.operation, best_throughput.ops_per_sec);
        }

        // Performance targets validation
        let sub_10ms_ops = self.results.iter().filter(|r| r.avg_duration.as_millis() < 10).count();
        let total_ops = self.results.len();
        let target_percentage = (sub_10ms_ops as f64 / total_ops as f64) * 100.0;
        
        println!("• Operations under 10ms target: {}/{} ({:.1}%)", sub_10ms_ops, total_ops, target_percentage);
        
        if target_percentage >= 90.0 {
            println!("  ✅ EXCELLENT: >90% of operations meet performance targets");
        } else if target_percentage >= 75.0 {
            println!("  ✅ GOOD: >75% of operations meet performance targets");  
        } else {
            println!("  ⚠️  Some operations may need optimization");
        }

        // Rope benefits summary
        println!("\n🚀 ROPE IMPLEMENTATION BENEFITS:");
        println!("• O(log n) complexity for all text operations");
        println!("• Efficient cursor navigation without full document scans");
        println!("• Memory efficient with copy-on-write semantics");
        println!("• Foundation ready for collaborative editing (CRDTs)");
        println!("• Zero-copy line iteration in hybrid renderer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_creation() {
        let benchmark = PerformanceBenchmark::new();
        assert_eq!(benchmark.results.len(), 0);
    }

    #[test]
    fn test_create_test_content() {
        let benchmark = PerformanceBenchmark::new();
        let content = benchmark.create_test_content(1000);
        assert!(content.len() <= 1000);
        assert!(content.len() > 900); // Should be close to target
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1536), "1.5KB");  
        assert_eq!(format_size(2097152), "2.0MB");
    }

    #[test]
    fn test_single_operation_benchmark() {
        let benchmark = PerformanceBenchmark::new();
        let mut doc = TextDocument::new();
        
        let result = benchmark.benchmark_operation("Test op", 100, 10, || {
            doc.insert_char('x');
        });
        
        assert_eq!(result.operation, "Test op");
        assert_eq!(result.iterations, 10);
        assert!(result.avg_duration.as_nanos() > 0);
        assert!(result.ops_per_sec > 0.0);
    }
}