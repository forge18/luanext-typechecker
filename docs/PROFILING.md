# Profiling Guide

This document describes how to profile the TypedLua type checker for performance optimization.

## Overview

The type checker can be profiled using several tools:
- **Criterion.rs** - Statistical benchmarking
- **flamegraph** - CPU sampling visualization
- **perf** - Linux CPU profiler
- **Instruments** - macOS profiler

## Benchmark Suite

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench type_checking
cargo bench --bench generics

# Run specific benchmark
cargo bench synthetic_exprs

# Quick benchmark (fewer samples)
cargo bench synthetic_exprs/100 -- --quick
```

### Benchmark Categories

| Benchmark | Description | Focus Area |
|-----------|-------------|------------|
| `synthetic_expressions` | Simple expressions (100-5000) | Expression inference |
| `synthetic_types` | Type alias chains (depth 5-50) | Type resolution |
| `many_variables` | Variable declarations (100-5000) | Symbol table |
| `nested_functions` | Nested function depth (5-100) | Scope management |
| `generic_functions` | Generic function calls (10-500) | Generic instantiation |
| `method_calls` | Method dispatch (10-1000) | Method resolution |
| `union_types` | Union type declarations (100-5000) | Union handling |
| `table_literals` | Table construction (100-5000) | Table inference |
| `interface_heavy` | Interface definitions (10-500) | Interface checking |
| `class_heavy` | Class hierarchies (10-200) | Class validation |

### Benchmark Results Location

Results are stored in `target/criterion/` with:
- `report/index.html` - HTML report
- `report/data/` - Raw data
- `plots/` - Generated plots

## Flame Graph Profiling

### Installation

```bash
# Install cargo-flamegraph
cargo install flamegraph

# For full symbols, install debug build
cargo install cargo-flamegraph --features debug
```

### Generating Flame Graphs

```bash
# Generate flame graph for specific benchmark
cargo flamegraph --bench type_checking -- synthetic_exprs/100

# Generate flame graph for full benchmark suite
cargo flamegraph --bench type_checking

# With custom output
cargo flamegraph --output flamegraph.svg --bench type_checking

# Control sampling frequency
cargo flamegraph --freq 99 --bench type_checking
```

### Interpreting Flame Graphs

- **Wide bars** = more time spent (hot paths)
- **Stack depth** = call chain depth
- **Colors** = optional (category, or random)

### Common Hotspots to Look For

```
TypeChecker::infer_expression      # Expression type inference (most frequent)
TypeEnvironment::lookup_type       # Type lookups
SymbolTable::lookup                # Symbol lookups
Type::clone                        # Type cloning (memory churn)
GenericInstantiation::instantiate  # Generic instantiation
```

## Linux perf Profiling

### Setup

```bash
# Install perf (Linux only)
sudo apt-get install linux-tools-common linux-tools-$(uname -r)

# Build with debug symbols
cargo build --release --features debug
```

### Profiling Commands

```bash
# Record profile
perf record -g -- ./target/release/typedlua_typechecker check file.lua

# Record with frequency
perf record -g -F 99 -- ./target/release/typedlua_typechecker check file.lua

# Record for specific duration
perf record -g -a -- sleep 30

# Generate flame graph from perf data
perf script | stackcollapse-perf | flamegraph > out.svg
```

### perf report

```bash
# Interactive analysis
perf report

# Top functions
perf report --stdio --no-children | head -30

# Annotate with source
perf annotate --stdio
```

## macOS Instruments

### Using Instruments.app

1. Open Instruments (Xcode → Open Developer Tool → Instruments)
2. Select "Time Profiler" template
3. Choose the binary to profile:
   - Build: `cargo build --release`
   - Binary: `target/release/deps/type_checking-*`
4. Click Record
5. Interact with the app or run benchmarks
6. Click Stop

### Command Line Instruments

```bash
# Profile with time profiler
instruments -t "Time Profiler" -p $(pgrep -f type_checking)

# Profile and save trace
instruments -t "Time Profiler" -p $(pgrep -f type_checking) -o trace.trace
```

## Heap Profiling

### Using dhat

Add to `Cargo.toml`:
```toml
[dev-dependencies]
dhat = "0.3"
```

In benchmark code:
```rust
use dhat::{DHeap, HeapStats};

fn benchmark() {
    let _profiler = dhat::Profiler::new_heap();
    // ... run code to profile ...
}
```

### heaptrack

```bash
# Install
cargo install heaptrack

# Run
heaptrack ./target/release/typedlua_typechecker check file.lua

# Analyze
heaptrack --analyze heaptrack_*.hpk
```

## Memory Allocation Tracking

### jemalloc

```bash
# Build with jemalloc
LD_PRELOAD=/usr/lib/libjemalloc.so cargo build --release

# Profile allocations
MALLOC_CONF=prof:true,prof_active:true cargo build --release
```

### tracing-allocs

Enable allocation tracking in logging:

```rust
#[global_allocator]
static ALLOC: dhat::AllocDebug = dhat::AllocDebug;
```

## Key Metrics to Track

### Performance Targets

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| 100 exprs | < 2ms | ~1.3ms | ✅ |
| 1000 exprs | < 15ms | ~10.7ms | ✅ |
| 5000 exprs | < 80ms | ~55ms | ✅ |
| Symbol lookup | < 100ns | - | ⏳ |
| Type lookup | < 200ns | - | ⏳ |
| Generic instantiation | < 10μs | - | ⏳ |

### Hotspot Priorities

1. **Expression inference** - Called thousands of times per file
2. **Type environment lookups** - Per-expression overhead
3. **Symbol table operations** - Per-declaration overhead
4. **Generic instantiation** - Recursive, allocates types
5. **Type cloning** - Memory churn

## CI Integration

### Benchmark Comparison

Use `cargo-benchcmp` to compare benchmark results:

```bash
# Install
cargo install benchcmp

# Compare two runs
benchcmp before.txt after.txt
```

### Performance Regression Checks

Add to CI pipeline:

```bash
# Run benchmarks and compare
cargo bench -- --save-baseline main
git checkout main
cargo bench -- --save-baseline candidate
benchcmp main.txt candidate.txt > comparison.txt
```

## Optimization Checklist

- [ ] Run baseline benchmarks
- [ ] Generate flame graph
- [ ] Identify top-5 hotspots
- [ ] Implement optimization
- [ ] Verify no regressions
- [ ] Document improvement
- [ ] Update performance targets

## Common Optimizations

### 1. Cache Type Lookups

```rust
// Before
fn lookup(&self, name: &str) -> Option<&Type> {
    self.types.get(name)
}

// After - with LRU cache
fn lookup(&mut self, name: &str) -> Option<&Type> {
    self.cache.get(name).or_else(|| {
        self.types.get(name).map(|t| {
            self.cache.insert(name, t.clone());
            self.cache.get(name).unwrap()
        })
    })
}
```

### 2. Reduce Type Cloning

```rust
// Before - clones on every lookup
fn infer(&mut self, expr: &Expr) -> Type {
    let t = self.env.lookup(expr.name).unwrap().clone();
    t
}

// After - borrow when possible
fn infer(&self, expr: &Expr) -> Option<&Type> {
    self.env.lookup(expr.name)
}
```

### 3. Pre-allocate Collections

```rust
// Before
let mut symbols = FxHashMap::default();

// After - with capacity hint
let mut symbols = FxHashMap::with_capacity_and_hasher(1024, Default::default());
```

## Troubleshooting

### "No samples collected"

- Increase benchmark duration
- Reduce sample count
- Check for `--release` mode

### "Benchmark too fast"

- Increase code complexity in benchmark
- Add more iterations
- Use `--quick` mode for development

### "Flame graph empty"

- Ensure debug symbols present
- Check sampling frequency
- Verify binary is running