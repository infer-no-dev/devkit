use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use devkit::context::ContextManager;
use devkit::codegen::CodeGenerator;
use devkit::ai::AIManager;
use devkit::config::Config;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_context_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = Config::default();
    let mut group = c.benchmark_group("context_analysis");
    
    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("files", size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let context_manager = ContextManager::new(config.clone()).unwrap();
                    // Simulate analyzing files
                    context_manager.analyze_directory(PathBuf::from("./src")).await.unwrap_or_default();
                });
            });
        });
    }
    group.finish();
}

fn bench_code_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = Config::default();
    let mut group = c.benchmark_group("code_generation");
    
    group.measurement_time(Duration::from_secs(30));
    
    group.bench_function("simple_function", |b| {
        b.iter(|| {
            rt.block_on(async {
                let ai_manager = AIManager::new(config.clone()).await.unwrap();
                let code_gen = CodeGenerator::new(ai_manager);
                
                let _result = code_gen.generate_code(
                    "Create a simple function that adds two numbers",
                    Some("rust".to_string()),
                    None
                ).await;
            });
        });
    });
    
    group.finish();
}

fn bench_agent_coordination(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = Config::default();
    let mut group = c.benchmark_group("agent_coordination");
    
    group.bench_function("multi_agent_task", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Benchmark agent system coordination
                // This would require proper agent system setup
            });
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_context_analysis,
    bench_code_generation,
    bench_agent_coordination
);
criterion_main!(benches);