use crate::args::ColorMode;
use crate::stats::{format_bytes_human, format_with_commas};
use anyhow::{Context, Result};
use colored::Colorize;
use jpx_engine::Runtime;
use serde_json::Value;
use std::io::{self, Write};
use std::time::Instant;

/// Run benchmark for expression(s)
pub(crate) fn run_benchmark(
    runtime: &Runtime,
    expressions: &[String],
    data: &Value,
    iterations: u32,
    warmup: u32,
    color_mode: &ColorMode,
) -> Result<()> {
    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    // Helper for colored output
    let heading = |s: &str| -> String {
        if use_color {
            s.green().bold().to_string()
        } else {
            s.to_string()
        }
    };

    let label = |s: &str| -> String {
        if use_color {
            s.dimmed().to_string()
        } else {
            s.to_string()
        }
    };

    let highlight = |s: &str| -> String {
        if use_color {
            s.cyan().bold().to_string()
        } else {
            s.to_string()
        }
    };

    let number = |s: &str| -> String {
        if use_color {
            s.yellow().to_string()
        } else {
            s.to_string()
        }
    };

    // Calculate input size
    let input_json = serde_json::to_string(data)?;
    let input_size = input_json.len();
    let item_count = match data {
        Value::Array(arr) => Some(arr.len()),
        Value::Object(obj) => Some(obj.len()),
        _ => None,
    };

    // Compile all expressions first
    let compiled: Vec<_> = expressions
        .iter()
        .map(|expr| {
            runtime
                .compile(expr)
                .with_context(|| format!("Failed to compile expression: {}", expr))
        })
        .collect::<Result<Vec<_>>>()?;

    // Combined expression string for display
    let expr_display = if expressions.len() == 1 {
        expressions[0].clone()
    } else {
        expressions.join(" | ")
    };

    println!();
    println!("{}", heading("BENCHMARK"));
    println!("{}", "═".repeat(60));
    println!();

    // Show expression
    println!("{} {}", label("Expression:"), highlight(&expr_display));

    // Show input info
    let size_str = format_bytes_human(input_size);
    if let Some(count) = item_count {
        println!(
            "{} {} ({} items)",
            label("Input size:"),
            size_str,
            format_with_commas(count)
        );
    } else {
        println!("{} {}", label("Input size:"), size_str);
    }
    println!();

    // Warmup runs
    if warmup > 0 {
        print!("{} {} iterations... ", label("Warmup:"), warmup);
        io::stdout().flush()?;
        for _ in 0..warmup {
            let mut result: Value = data.clone();
            for expr in &compiled {
                result = expr.search(&result).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }
        println!("done");
    }

    // Benchmark runs
    print!(
        "{} {} iterations... ",
        label("Running:"),
        number(&iterations.to_string())
    );
    io::stdout().flush()?;

    let mut timings: Vec<f64> = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let mut result: Value = data.clone();
        let start = Instant::now();
        for expr in &compiled {
            result = expr.search(&result).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        let elapsed = start.elapsed();
        timings.push(elapsed.as_secs_f64() * 1000.0); // Convert to milliseconds
    }

    println!("done");
    println!();

    // Calculate statistics (total_cmp is total over all f64, so no unwrap of a
    // possible NaN comparison).
    timings.sort_by(|a, b| a.total_cmp(b));

    let total: f64 = timings.iter().sum();
    let mean = total / timings.len() as f64;
    #[allow(clippy::manual_is_multiple_of)] // is_multiple_of is unstable
    let median = if timings.len() % 2 == 0 {
        (timings[timings.len() / 2 - 1] + timings[timings.len() / 2]) / 2.0
    } else {
        timings[timings.len() / 2]
    };
    let min = timings.first().copied().unwrap_or(0.0);
    let max = timings.last().copied().unwrap_or(0.0);

    // Standard deviation
    let variance: f64 =
        timings.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / timings.len() as f64;
    let stddev = variance.sqrt();

    // Percentiles
    let p95_idx = ((timings.len() as f64 * 0.95) as usize).min(timings.len() - 1);
    let p99_idx = ((timings.len() as f64 * 0.99) as usize).min(timings.len() - 1);
    let p95 = timings[p95_idx];
    let p99 = timings[p99_idx];

    // Throughput (MB/s based on mean time)
    let throughput_mbs = if mean > 0.0 {
        (input_size as f64 / 1_000_000.0) / (mean / 1000.0)
    } else {
        0.0
    };

    // Print results
    println!("{}", heading("Results"));
    println!("{}", "─".repeat(40));
    println!(
        "  {:12} {}",
        label("Iterations:"),
        number(&format_with_commas(iterations as usize))
    );
    println!("  {:12} {}", label("Total time:"), format_duration(total));
    println!();
    println!(
        "  {:12} {}",
        label("Mean:"),
        highlight(&format_duration(mean))
    );
    println!("  {:12} {}", label("Median:"), format_duration(median));
    println!("  {:12} {}", label("Std dev:"), format_duration(stddev));
    println!();
    println!("  {:12} {}", label("Min:"), format_duration(min));
    println!("  {:12} {}", label("Max:"), format_duration(max));
    println!("  {:12} {}", label("p95:"), format_duration(p95));
    println!("  {:12} {}", label("p99:"), format_duration(p99));
    println!();
    println!("  {:12} {:.2} MB/s", label("Throughput:"), throughput_mbs);

    // Show histogram if enough samples
    if iterations >= 10 {
        println!();
        println!("{}", heading("Distribution"));
        println!("{}", "─".repeat(40));
        print_histogram(&timings, use_color);
    }

    println!();

    Ok(())
}

/// Format duration in appropriate units
fn format_duration(ms: f64) -> String {
    if ms < 0.001 {
        format!("{:.3} ns", ms * 1_000_000.0)
    } else if ms < 1.0 {
        format!("{:.3} µs", ms * 1000.0)
    } else if ms < 1000.0 {
        format!("{:.3} ms", ms)
    } else {
        format!("{:.3} s", ms / 1000.0)
    }
}

/// Print a simple ASCII histogram
fn print_histogram(timings: &[f64], use_color: bool) {
    const BUCKETS: usize = 10;
    const BAR_WIDTH: usize = 30;

    let min = timings.first().copied().unwrap_or(0.0);
    let max = timings.last().copied().unwrap_or(0.0);
    let range = max - min;

    if range == 0.0 {
        println!("  (all samples identical)");
        return;
    }

    let bucket_size = range / BUCKETS as f64;
    let mut buckets = [0usize; BUCKETS];

    for &t in timings {
        let idx = ((t - min) / bucket_size) as usize;
        let idx = idx.min(BUCKETS - 1);
        buckets[idx] += 1;
    }

    let max_count = *buckets.iter().max().unwrap_or(&1);

    for (i, &count) in buckets.iter().enumerate() {
        let lower = min + (i as f64 * bucket_size);
        let upper = lower + bucket_size;
        let bar_len = (count * BAR_WIDTH) / max_count.max(1);
        let bar: String = "█".repeat(bar_len);

        let bar_display = if use_color {
            bar.cyan().to_string()
        } else {
            bar
        };

        println!(
            "  {:>8} - {:>8} │{:<width$}│ {}",
            format!("{:.2}", lower),
            format!("{:.2}", upper),
            bar_display,
            count,
            width = BAR_WIDTH
        );
    }
}
