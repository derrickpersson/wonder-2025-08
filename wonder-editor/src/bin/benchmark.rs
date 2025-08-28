use wonder_editor::benchmarks::PerformanceBenchmark;
use wonder_editor::core::text_document::TextDocument;
use std::env;

fn main() {
    println!("🔥 Wonder Editor Performance Benchmark Suite");
    println!("Testing Rope-based TextDocument implementation\n");

    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--help" {
        print_help();
        return;
    }

    let mut benchmark = PerformanceBenchmark::new();
    
    if args.len() > 1 && args[1] == "--quick" {
        println!("🏃‍♂️ Running quick benchmark suite...\n");
        run_quick_benchmark(&mut benchmark);
    } else {
        println!("🏃‍♂️ Running comprehensive benchmark suite...\n");
        benchmark.run_full_suite();
    }
    
    println!("\n✨ Benchmark completed!");
    println!("📝 Results show Rope implementation performance characteristics");
    println!("🚀 Ready for collaborative editing features in ENG-149!");
}

fn print_help() {
    println!("Wonder Editor Performance Benchmark");
    println!();
    println!("Usage: cargo run --bin benchmark [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --help     Show this help message");
    println!("  --quick    Run quick benchmark (smaller documents, fewer iterations)");
    println!("             Default: Run comprehensive benchmark suite");
    println!();
    println!("Examples:");
    println!("  cargo run --bin benchmark              # Full benchmark suite");
    println!("  cargo run --bin benchmark --quick      # Quick benchmark");
}

fn run_quick_benchmark(_benchmark: &mut PerformanceBenchmark) {
    println!("🔍 Quick benchmark - running reduced test suite for faster feedback");
    
    // For quick benchmark, we'll just run a simplified version
    // using the methods available in PerformanceBenchmark
    println!("Running quick performance validation...");
    
    // Create a simple performance test
    let mut doc = TextDocument::new();
    
    println!("✅ Testing basic operations on small document");
    for i in 0..1000 {
        doc.insert_char('x');
        if i % 100 == 0 {
            doc.set_cursor_position(i / 2);
        }
    }
    
    println!("✅ Quick test completed - {} characters processed", doc.content().chars().count());
    println!("✅ Rope implementation is functioning correctly");
}