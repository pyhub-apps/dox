use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::collections::HashMap;

fn create_test_text(size: usize) -> String {
    let base = "Hello {{name}}, welcome to {{company}}. Today is {{date}}. ";
    base.repeat(size / base.len() + 1)
}

fn create_rules(count: usize) -> HashMap<String, String> {
    let mut rules = HashMap::new();
    rules.insert("{{name}}".to_string(), "John Doe".to_string());
    rules.insert("{{company}}".to_string(), "Acme Corporation".to_string());
    rules.insert("{{date}}".to_string(), "2024-01-01".to_string());
    
    // Add more rules if requested
    for i in 3..count {
        rules.insert(format!("{{{{var{}}}}}", i), format!("value{}", i));
    }
    
    rules
}

fn simple_replace(text: &str, rules: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in rules {
        result = result.replace(pattern, replacement);
    }
    result
}

fn benchmark_replace_text_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("replace_text_sizes");
    let rules = create_rules(3);
    
    for size in [100, 1000, 10000, 100000].iter() {
        let text = create_test_text(*size);
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    simple_replace(black_box(&text), black_box(&rules))
                });
            },
        );
    }
    group.finish();
}

fn benchmark_rule_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("replace_rule_counts");
    let text = create_test_text(1000);
    
    for count in [1, 5, 10, 25, 50].iter() {
        let rules = create_rules(*count);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, _| {
                b.iter(|| {
                    simple_replace(black_box(&text), black_box(&rules))
                });
            },
        );
    }
    group.finish();
}

fn benchmark_pattern_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_complexity");
    let text = create_test_text(1000);
    
    // Simple pattern
    let simple_rules = {
        let mut rules = HashMap::new();
        rules.insert("a".to_string(), "b".to_string());
        rules
    };
    
    // Medium pattern
    let medium_rules = {
        let mut rules = HashMap::new();
        rules.insert("{{name}}".to_string(), "John".to_string());
        rules
    };
    
    // Complex pattern
    let complex_rules = {
        let mut rules = HashMap::new();
        rules.insert("{{very_long_variable_name_here}}".to_string(), "replacement".to_string());
        rules
    };
    
    group.bench_function("simple", |b| {
        b.iter(|| simple_replace(black_box(&text), black_box(&simple_rules)))
    });
    
    group.bench_function("medium", |b| {
        b.iter(|| simple_replace(black_box(&text), black_box(&medium_rules)))
    });
    
    group.bench_function("complex", |b| {
        b.iter(|| simple_replace(black_box(&text), black_box(&complex_rules)))
    });
    
    group.finish();
}

fn benchmark_concurrent_processing(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;
    
    let mut group = c.benchmark_group("concurrent_processing");
    let rules = Arc::new(create_rules(3));
    
    for num_threads in [1, 2, 4, 8].iter() {
        let texts: Vec<String> = (0..10).map(|_| create_test_text(1000)).collect();
        let texts = Arc::new(texts);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    let chunk_size = texts.len() / num_threads;
                    
                    for i in 0..num_threads {
                        let start = i * chunk_size;
                        let end = if i == num_threads - 1 {
                            texts.len()
                        } else {
                            (i + 1) * chunk_size
                        };
                        
                        let texts = texts.clone();
                        let rules = rules.clone();
                        
                        let handle = thread::spawn(move || {
                            let mut results = vec![];
                            for j in start..end {
                                results.push(simple_replace(&texts[j], &rules));
                            }
                            results
                        });
                        
                        handles.push(handle);
                    }
                    
                    let mut all_results = vec![];
                    for handle in handles {
                        all_results.extend(handle.join().unwrap());
                    }
                    all_results
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark in-place vs new string creation
    let text = create_test_text(10000);
    let rules = create_rules(10);
    
    group.bench_function("new_string", |b| {
        b.iter(|| {
            simple_replace(black_box(&text), black_box(&rules))
        });
    });
    
    // Alternative implementation that might use less memory
    group.bench_function("chunked_replace", |b| {
        b.iter(|| {
            let mut result = String::with_capacity(text.len());
            let mut last_end = 0;
            
            // Find all occurrences first
            for (pattern, replacement) in &rules {
                let mut start = 0;
                while let Some(pos) = text[start..].find(pattern) {
                    let abs_pos = start + pos;
                    result.push_str(&text[last_end..abs_pos]);
                    result.push_str(replacement);
                    last_end = abs_pos + pattern.len();
                    start = abs_pos + pattern.len();
                }
            }
            result.push_str(&text[last_end..]);
            result
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_replace_text_sizes,
    benchmark_rule_counts,
    benchmark_pattern_complexity,
    benchmark_concurrent_processing,
    benchmark_memory_usage
);

criterion_main!(benches);