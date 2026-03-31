mod bench_completion;
mod bench_format;
mod bench_parser;
mod bench_startup;

use criterion::criterion_main;

criterion_main!(
    bench_startup::cli_benches,
    bench_startup::config_benches,
    bench_parser::parser_benches,
    bench_format::format_benches,
    bench_completion::completion_benches,
);
