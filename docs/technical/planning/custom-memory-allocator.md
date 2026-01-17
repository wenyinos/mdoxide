# Custom Memory Allocator

## Overview

Ferrite uses platform-specific high-performance memory allocators to reduce heap fragmentation and improve allocation performance during long editing sessions.

- **Windows**: mimalloc (Microsoft's compact, fast allocator)
- **Unix** (Linux/macOS): jemalloc (battle-tested allocator from Meta/Facebook)

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Platform-specific allocator dependencies and feature flag |
| `src/main.rs` | Global allocator configuration with `#[global_allocator]` |

## Implementation Details

### Feature Flag

The allocator is controlled by the `high-perf-alloc` feature, enabled by default:

```toml
[features]
default = ["bundle-icon", "high-perf-alloc"]
high-perf-alloc = ["mimalloc", "tikv-jemallocator"]
```

### Platform-Specific Dependencies

```toml
[target.'cfg(windows)'.dependencies]
mimalloc = { version = "0.1", default-features = false, optional = true }

[target.'cfg(unix)'.dependencies]
tikv-jemallocator = { version = "0.6", optional = true }
```

### Global Allocator Declaration

In `src/main.rs`, conditional compilation selects the appropriate allocator:

```rust
#[cfg(all(feature = "high-perf-alloc", target_os = "windows"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(feature = "high-perf-alloc", unix))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

## Dependencies Used

| Crate | Version | Platform | Purpose |
|-------|---------|----------|---------|
| `mimalloc` | 0.1 | Windows | Microsoft's high-performance allocator |
| `tikv-jemallocator` | 0.6 | Unix | jemalloc wrapper (TiKV's maintained fork) |

## Benefits

1. **Reduced fragmentation**: Better memory reuse over long sessions
2. **Faster allocations**: Optimized for small allocations (common in text editors)
3. **Thread-local caches**: Reduced lock contention on multi-threaded operations
4. **Consistent performance**: Less variance in allocation latency

## Trade-offs

- Slightly higher baseline memory (~1-3MB overhead for allocator metadata)
- Benefits most visible with sustained use and many open/close cycles

## Usage

### Default Build (with allocator)

```bash
cargo build --release
```

### Build Without Allocator

```bash
cargo build --release --no-default-features --features bundle-icon
```

### Verify Allocator on Linux

```bash
# jemalloc should be statically linked, but you can check symbols:
nm target/release/ferrite | grep -i jemalloc
```

## Notes

- Uses `tikv-jemallocator` instead of the deprecated `jemallocator` crate
- Allocators are statically linked (no runtime DLL dependencies)
- CI builds automatically use the appropriate allocator per platform
