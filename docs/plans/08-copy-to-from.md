# Sub-Plan SP8: COPY TO/FROM

> Parent: [high-level-design.md](high-level-design.md) | Phase: 4
> **Status: PARTIALLY COMPLETE** — COPY TO implemented with all 17 options. COPY FROM not started (19 options, parallel workers, rate limiting). Deferred to Phase 4.

## Objective

Implement the COPY TO and COPY FROM commands with 100% option compatibility, parallel processing, rate limiting, progress reporting, and error handling matching Python cqlsh behavior.

---

## Research Phase

### Tasks

1. **Python cqlsh COPY implementation** — Read `cqlshlib/copyutil.py` (largest file in cqlsh)
2. **COPY TO algorithm** — How it pages through data, token ranges, concurrent fetching
3. **COPY FROM algorithm** — How it chunks CSV, batches inserts, handles errors
4. **All options** — Default values, validation rules, interaction between options
5. **Progress reporting** — Format, frequency, content
6. **Error handling** — Parse errors, insert errors, error files, retry logic
7. **Performance characteristics** — How Python achieves throughput with multiprocessing

### Research Deliverables

- [ ] COPY TO algorithm specification (page-by-page export)
- [ ] COPY FROM algorithm specification (chunk-batch-insert pipeline)
- [ ] Option interaction rules (which options affect which)
- [ ] Progress report format specification
- [ ] Error handling flowchart
- [ ] Performance comparison targets (Python multiprocessing vs Tokio tasks)

---

## Execution Phase

### COPY TO Implementation

| Step | Description | Tests |
|------|-------------|-------|
| 1 | Basic COPY TO (SELECT all, write CSV) | Integration: small table |
| 2 | DELIMITER option | Unit: custom delimiter |
| 3 | QUOTE, ESCAPE options | Unit: quoting behavior |
| 4 | HEADER option | Unit: header row |
| 5 | NULL option | Unit: null representation |
| 6 | DATETIMEFORMAT option | Unit: timestamp format |
| 7 | ENCODING option | Unit: file encoding |
| 8 | FLOATPRECISION, DOUBLEPRECISION | Unit: precision |
| 9 | DECIMALSEP, THOUSANDSSEP | Unit: number format |
| 10 | BOOLSTYLE | Unit: boolean format |
| 11 | PAGESIZE option | Integration: page fetching |
| 12 | PAGETIMEOUT option | Integration: timeout behavior |
| 13 | MAXREQUESTS (concurrent page fetching) | Integration: parallelism |
| 14 | BEGINTOKEN, ENDTOKEN (token range) | Integration: range export |
| 15 | MAXOUTPUTSIZE (row limit) | Unit: row limiting |
| 16 | REPORTFREQUENCY (progress reporting) | Manual: progress display |
| 17 | Stdout output (COPY TO STDOUT) | Unit: stdout mode |

### COPY FROM Implementation

| Step | Description | Tests |
|------|-------------|-------|
| 1 | Basic COPY FROM (read CSV, INSERT rows) | Integration: small file |
| 2 | All shared options (DELIMITER, QUOTE, etc.) | Unit: parsing options |
| 3 | CHUNKSIZE (rows per read chunk) | Unit: chunk reading |
| 4 | MAXBATCHSIZE, MINBATCHSIZE (batch sizing) | Unit: batch formation |
| 5 | PREPAREDSTATEMENTS option | Integration: prepared vs unprepared |
| 6 | TTL option | Integration: TTL on insert |
| 7 | NUMPROCESSES (parallel workers) | Integration: parallel inserts |
| 8 | INGESTRATE (rate limiting) | Unit: rate control |
| 9 | MAXATTEMPTS (retry per batch) | Integration: retry behavior |
| 10 | MAXPARSEERRORS (error tolerance) | Unit: parse error counting |
| 11 | MAXINSERTERRORS (error tolerance) | Integration: insert error counting |
| 12 | ERRFILE (error logging) | Unit: error file writing |
| 13 | REPORTFREQUENCY (progress reporting) | Manual: progress display |
| 14 | Stdin input (COPY FROM STDIN) | Unit: stdin mode |

### Acceptance Criteria

- [ ] COPY TO exports all data types correctly
- [ ] COPY FROM imports all data types correctly
- [ ] All 30+ options work correctly
- [ ] Progress reporting matches Python cqlsh format
- [ ] Error handling matches Python cqlsh behavior
- [ ] Large dataset (1M rows) performance is comparable to Python cqlsh
- [ ] Token-range export works for parallel data extraction
- [ ] STDOUT/STDIN modes work for piping

> Note: This is the most complex sub-plan. COPY FROM alone is comparable to a standalone tool.

---

## Skills Required

- Async Rust / Tokio (S2) — parallel workers, channels, rate limiting
- CSV processing (S7, C7)
- CQL type parsing from strings (D1, S3)
- Performance optimization (S10)
- Error handling patterns (S1)

---

## Key Decisions

| Decision | Options | Recommendation |
|----------|---------|---------------|
| Parallel strategy | a) Tokio tasks, b) OS threads, c) Rayon | (a) Tokio tasks (consistent with rest of app) |
| Rate limiting | a) Token bucket, b) Leaky bucket, c) Simple sleep | (a) Token bucket for smooth throughput |
| CSV parser | a) `csv` crate, b) Custom parser | (a) `csv` crate with custom configuration |
| Batch insertion | a) CQL BATCH, b) Individual prepared stmts | (b) Individual prepared stmts (matches Python cqlsh) |
