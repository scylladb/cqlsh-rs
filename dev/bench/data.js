window.BENCHMARK_DATA = {
  "lastUpdate": 1776715294625,
  "repoUrl": "https://github.com/scylladb/cqlsh-rs",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "c4b474f33d9eb3e39a1e2904323ad78072ec882f",
          "message": "docs(plans): add SP17 — embedded AI assistant help plan\n\nAdd comprehensive plan for embedding Qwen2.5-Coder-0.5B into cqlsh-rs\nfor offline CQL error diagnostics via --ai-help flag. Covers model\nselection, inference engine (llama-cpp-2), distribution/caching strategy,\nprompt engineering, UX, resource management, and phased implementation.\n\nhttps://claude.ai/code/session_016AR2GzCPyLuko2smjcishH",
          "timestamp": "2026-03-15T01:32:16+02:00",
          "tree_id": "96fd860a5ec5e96ee1eb69e6098b26c1975d6c55",
          "url": "https://github.com/fruch/cqlsh-rs/commit/c4b474f33d9eb3e39a1e2904323ad78072ec882f"
        },
        "date": 1773531538254,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16673,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18325,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20223,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19298,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19001,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 34928,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2872,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6222,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44177,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5500,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12584,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51961,
            "range": "± 942",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 94113,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 528,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 591,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1112,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1330,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14662,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 52925,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21805,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 100495,
            "range": "± 635",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "81bfcab2b79848fc231896f34f82ea1afdd9f824",
          "message": "fix(driver): add debug logging for value conversion and remove redundant error context\n\n- Add tracing::debug! in row conversion loop to log each ScyllaCqlValue\n  variant before conversion, aiding diagnosis of empty column values\n- Add tracing::warn! for unhandled ScyllaCqlValue variants in catch-all\n- Remove redundant .context(\"executing CQL query\") from execute_unpaged\n  — the driver error from ScyllaDB is already descriptive\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-15T01:32:25+02:00",
          "tree_id": "643329880aa01fdae382e1540331d1de4100f050",
          "url": "https://github.com/fruch/cqlsh-rs/commit/81bfcab2b79848fc231896f34f82ea1afdd9f824"
        },
        "date": 1773531539622,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 14854,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 16940,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 18915,
            "range": "± 602",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 17326,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 17458,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 31390,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2772,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6082,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 43747,
            "range": "± 1328",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5108,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12847,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51956,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 103126,
            "range": "± 898",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 291,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 341,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1104,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 519,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 9089,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 45937,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 18650,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 90539,
            "range": "± 308",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "4c6cfed451fb02e8175b37eb440044903e760819",
          "message": "fix(ci): save current ref before orphan checkout for gh-pages creation\n\ngit checkout - fails after --orphan because the previous branch ref is\nlost. Save HEAD before and checkout the commit SHA after pushing.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-15T01:51:29+02:00",
          "tree_id": "f77d6921eebfe8ad03f5394d5f212e0658c8a960",
          "url": "https://github.com/fruch/cqlsh-rs/commit/4c6cfed451fb02e8175b37eb440044903e760819"
        },
        "date": 1773532671059,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 14828,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 16774,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 18823,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 17171,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 17320,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 33990,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2690,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 5840,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 41849,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5141,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12814,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 52247,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 97385,
            "range": "± 728",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 294,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 345,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1101,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 520,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 8932,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 45304,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 18414,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 89166,
            "range": "± 316",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "ad69387e832a5db060547c8758df443a64de964f",
          "message": "feat(hooks): add SessionStart hook to install gh CLI for Claude web\n\nAdds a Claude Code SessionStart hook that automatically installs the\nGitHub CLI (gh) when it's not available in the environment. This ensures\ngh commands work in Claude Code web sessions.\n\nhttps://claude.ai/code/session_01EmWYLFqv1L5jVYs9hu8RtU",
          "timestamp": "2026-03-15T22:05:13+02:00",
          "tree_id": "6ddd70b1de0e52a333b54d36d056cb9af6125942",
          "url": "https://github.com/fruch/cqlsh-rs/commit/ad69387e832a5db060547c8758df443a64de964f"
        },
        "date": 1773605508269,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16850,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 19873,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20502,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19403,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19923,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 38339,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2859,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6087,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 43686,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5371,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12412,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 50595,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 93656,
            "range": "± 2418",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 524,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 583,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1107,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1326,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14522,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 52267,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 22571,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 98039,
            "range": "± 1087",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "d83e720ce15250ec46033eadffa740115ae753b5",
          "message": "fix(parser): address review findings — O(n) incremental scanning and missing tests\n\nRewrites the parser to fix all P0/P1 issues identified in code review:\n\nP0: O(n²) re-scanning → truly incremental via scan_offset tracking.\n  - State (LexState, block_comment_depth) is preserved across feed_line\n    calls instead of being reset and re-scanned from position 0.\n  - Total work is now O(n) over the parser's lifetime.\n\nP1: Eliminated buffer.clone() and Vec<char> allocations.\n  - Uses byte-offset scanning with decode_char_at() instead of\n    collecting into Vec<char> (4x memory inflation for ASCII).\n  - No buffer clone; scans in-place and compacts only on Complete.\n\nP1: strip_comments now handles nested block comments.\n  - Consistent with the main lexer's block_comment_depth tracking.\n\nAPI improvements:\n  - #[must_use] on ParseResult, InputKind, new(), is_empty(), is_shell_command\n  - #[default] on LexState::Normal, derive Default for StatementParser\n  - Added remaining() accessor (replaces direct buffer field access)\n  - classify_input delegates to is_shell_command (removes duplication)\n  - is_shell_command now handles trailing semicolons\n\nNew tests (14 added, 52 total):\n  - nested_block_comments, nested_block_comments_stripped\n  - block_comment_across_feed_lines\n  - line_comment_then_statement_across_lines\n  - reuse_after_complete, reuse_after_complete_multiline\n  - unterminated_string_blocks_semicolon\n  - unterminated_block_comment_blocks_semicolon\n  - backslash_in_string_is_literal\n  - empty_dollar_quote\n  - classify_shell_command_with_semicolon\n  - shell_command_with_semicolon\n  - parse_batch_only_comments\n  - incremental_scan_preserves_state_across_lines\n\nhttps://claude.ai/code/session_01Y7qFwwx57pSsc9FnZYvgnZ",
          "timestamp": "2026-03-15T23:27:32+02:00",
          "tree_id": "fc58e1bbdb21218aac80c5d0cbb7ebfb45964a27",
          "url": "https://github.com/fruch/cqlsh-rs/commit/d83e720ce15250ec46033eadffa740115ae753b5"
        },
        "date": 1773610424974,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16817,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18714,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20033,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19224,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19641,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37928,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2887,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6077,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44332,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5447,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12555,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51932,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 96660,
            "range": "± 1399",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 534,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 581,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1110,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1320,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14670,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 52534,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21671,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 98712,
            "range": "± 612",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "67e0952d36b31a991c950a5866db1fc33734f9a8",
          "message": "fix: address all 9 bugs from PR22 manual testing\n\nBUG-1: DESCRIBE TABLES lists all keyspaces with grouped tables\nBUG-2: Tabular output with proper column widths and separators\nBUG-3: EXPAND ON/OFF with vertical format matching Python cqlsh\nBUG-4: SOURCE command dispatches shell commands correctly\nBUG-5: Multi-line paste splits lines before processing\nBUG-6: Error messages use category-aware formatting (SyntaxError, etc.)\nBUG-7: Inline comment after semicolon no longer enters continuation\nBUG-8: Bare ;; no longer enters continuation prompt loop\nBUG-9: Banner shows ScyllaDB version when connected to ScyllaDB\n\nAdds error module, ScyllaDB version detection in driver/session,\nand rewrites REPL output formatting. 35 new unit tests.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-17T02:16:48+02:00",
          "tree_id": "6a822422f74569459c960e0c57ec3d34d66d51f1",
          "url": "https://github.com/fruch/cqlsh-rs/commit/67e0952d36b31a991c950a5866db1fc33734f9a8"
        },
        "date": 1773707033803,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16595,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18884,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 19898,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19100,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19536,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37747,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2782,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6157,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 43860,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5382,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12527,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51002,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 95188,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 559,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 612,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1154,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1326,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14785,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 52796,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 22240,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 98995,
            "range": "± 621",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "e5a23cfbc4d451addd2083bb5054595d67aea2e3",
          "message": "chore: add .worktrees/ to .gitignore\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-25T17:28:32+02:00",
          "tree_id": "4dfba55ed64572e64963e56fd68e54425d5b4fbf",
          "url": "https://github.com/fruch/cqlsh-rs/commit/e5a23cfbc4d451addd2083bb5054595d67aea2e3"
        },
        "date": 1774452966028,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16514,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18532,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 19621,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 18911,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19294,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37703,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2781,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6114,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44388,
            "range": "± 2502",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5382,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12623,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51823,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 94796,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 543,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 602,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1124,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1331,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14544,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 53086,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21556,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 97783,
            "range": "± 425",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "48891954bf323e6ff378844edc132cf01da76e93",
          "message": "feat(pager): replace MORE prompt with sapling-streampager\n\nReplace the simple ---MORE--- forward-only pagination with\nsapling-streampager, a pure-Rust pager using termwiz for ANSI-aware\nrendering. Features:\n- Vertical scrolling (up/down arrows)\n- Horizontal scrolling for wide tables (left/right arrows)\n- Regex search with /pattern\n- Correct ANSI color handling during scroll\n- Hybrid mode: prints directly if output fits the screen\n- Column names shown in pager title for context while scrolling\n- Max column width (40 chars) with cell wrapping for readability\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-25T23:33:58+02:00",
          "tree_id": "caf5987311d0955e63a6244e9340cd8775d5482b",
          "url": "https://github.com/fruch/cqlsh-rs/commit/48891954bf323e6ff378844edc132cf01da76e93"
        },
        "date": 1774474911112,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 14723,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 16903,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 18212,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 17226,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 17346,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 34372,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2659,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 5882,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 42477,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5146,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12902,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51893,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 97371,
            "range": "± 1092",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 291,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 341,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1092,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 528,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 8955,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 45967,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 18502,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 90139,
            "range": "± 269",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "e9719b363afef84fff2aa5a2e160d56c7e7b6584",
          "message": "docs(progress): update progress tracking and integrate into development workflow\n\n- Update progress.json to reflect reality: Phase 1 (15/15), Phase 2\n  (22/22), Phase 3 (21/21) all completed — was stuck at Phase 1 (5/15)\n- Fix progress.yml GitHub Action: trigger on push to main instead of\n  PR merge, since this repo uses direct pushes (all PRs show merged=false)\n- Add Step 5b to development-process skill: mandatory progress.json\n  update after completing tasks, with detailed instructions\n- Add progress scope and tracking reminder to conventional-commit skill\n- Add progress tracking section to CLAUDE.md as a key convention\n- Add progress.json reminder to create-implementation-plan skill\n- Regenerate progress-roadmap.svg (now shows 54% complete)\n\nhttps://claude.ai/code/session_01Sks9pNgaYb8jBb8oNGbfee",
          "timestamp": "2026-03-25T23:51:45+02:00",
          "tree_id": "25297d5804f6978a2cd40a53813ae422b025b29f",
          "url": "https://github.com/fruch/cqlsh-rs/commit/e9719b363afef84fff2aa5a2e160d56c7e7b6584"
        },
        "date": 1774475977857,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16763,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18613,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20973,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19093,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19074,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 35942,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2859,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6317,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 42629,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5657,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12494,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 50710,
            "range": "± 839",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 94372,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 604,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 659,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1211,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1467,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 16316,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 53178,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 22173,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 93031,
            "range": "± 401",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "c4b474f33d9eb3e39a1e2904323ad78072ec882f",
          "message": "docs(plans): add SP17 — embedded AI assistant help plan\n\nAdd comprehensive plan for embedding Qwen2.5-Coder-0.5B into cqlsh-rs\nfor offline CQL error diagnostics via --ai-help flag. Covers model\nselection, inference engine (llama-cpp-2), distribution/caching strategy,\nprompt engineering, UX, resource management, and phased implementation.\n\nhttps://claude.ai/code/session_016AR2GzCPyLuko2smjcishH",
          "timestamp": "2026-03-15T01:32:16+02:00",
          "tree_id": "96fd860a5ec5e96ee1eb69e6098b26c1975d6c55",
          "url": "https://github.com/fruch/cqlsh-rs/commit/c4b474f33d9eb3e39a1e2904323ad78072ec882f"
        },
        "date": 1774476320740,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16980,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18783,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20318,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 18893,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19157,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37913,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2873,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6103,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 43926,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5319,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12343,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51234,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 93478,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 532,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 586,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1121,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1340,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14604,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 52234,
            "range": "± 530",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21991,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 98411,
            "range": "± 762",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "651b1fde4a6a57a1654ced9e352e1b2da85fc7d0",
          "message": "docs: add benchmark links to GitHub Actions workflow in README\n\nAdd clickable links for benchmark workflow runs and Criterion report\nartifacts so users can easily navigate to CI benchmark results.\n\nhttps://claude.ai/code/session_01HwgGQJeLayBZbniRhAXKBB",
          "timestamp": "2026-03-26T00:09:54+02:00",
          "tree_id": "28d74436bdea8fd6243c1a7eb6ceeb077094f22d",
          "url": "https://github.com/fruch/cqlsh-rs/commit/651b1fde4a6a57a1654ced9e352e1b2da85fc7d0"
        },
        "date": 1774477073334,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 15995,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 17801,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 19410,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 18372,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 18178,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37046,
            "range": "± 1030",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2778,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6189,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44901,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5545,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12705,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51880,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 97116,
            "range": "± 697",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 555,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 607,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1146,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1327,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14718,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 53784,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 20963,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 100241,
            "range": "± 1423",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f8f2d8a38ae7a8f3e5c0168507f6e1973e223a84",
          "message": "feat(cli): integrate --tty, --encoding, --cqlversion, --protocol-version flags\n\n- --tty: forces TTY behavior (pager, color) even when stdout is piped\n- --encoding: displayed in UNICODE command, logged in debug mode\n- --cqlversion: logged in debug mode (scylla-rs auto-negotiates)\n- --protocol-version: logged in debug mode (scylla-rs auto-negotiates)\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-26T08:32:54+02:00",
          "tree_id": "ba30a191afcd4fad712592d5a7c4187cbb184561",
          "url": "https://github.com/fruch/cqlsh-rs/commit/f8f2d8a38ae7a8f3e5c0168507f6e1973e223a84"
        },
        "date": 1774507245634,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16203,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18283,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 19516,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 18837,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 18797,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37519,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2781,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6094,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44222,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5415,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12620,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51579,
            "range": "± 1787",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 95465,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 494,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 547,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1086,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1317,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14738,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 52898,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21429,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 98380,
            "range": "± 552",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "303eec32718f6cd0e1e09cba2eeaca17c7cb0361",
          "message": "docs(plan): add SP19 test gap analysis — Python cqlsh vs cqlsh-rs\n\nComprehensive comparison of Python cqlsh test suites (scylla-cqlsh\nunit tests + scylla-dtest integration tests) against cqlsh-rs coverage.\n\nKey findings:\n- 0 integration tests in cqlsh-rs (Python has 80+)\n- Missing: escape sequences (25 tests), Unicode (5), auth (17),\n  output comparison (25+), COPY integration (35+), LOGIN (4)\n- 8-phase implementation plan to close gaps (Phases 1-8)\n- Target: 500+ tests (from current 327)\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-26T13:24:24+02:00",
          "tree_id": "d2797d7bf03f5925f6bc846577aca5ca00690505",
          "url": "https://github.com/fruch/cqlsh-rs/commit/303eec32718f6cd0e1e09cba2eeaca17c7cb0361"
        },
        "date": 1774524744558,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16257,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18522,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 19207,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19104,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 18929,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37653,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2896,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6094,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 45155,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5374,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12670,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 52276,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 96880,
            "range": "± 873",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 539,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 593,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1124,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1318,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14531,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 53280,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21193,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 96943,
            "range": "± 552",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "139b8a09fd8e01f9b869955ecc7b6d372674ab82",
          "message": "test(SP19): add integration test infrastructure with testcontainers-rs\n\nSet up SP19 Phase 1-2: integration test infrastructure using\ntestcontainers-rs with ScyllaDB and 19 core integration tests covering\nCRUD, DDL, BATCH, USE, UUID, TRUNCATE, data types, collections,\nnull/empty values, SHOW commands, and --no-color flag verification.\n\n- Add testcontainers (blocking) dev-dependency\n- Create tests/integration/ with shared helpers (container singleton,\n  cqlsh command execution, keyspace management)\n- Add 19 integration tests (all #[ignore = \"requires Docker\"])\n- Add integration test CI job to GitHub Actions\n- Update progress.json for Phase 5 start\n\nhttps://claude.ai/code/session_01FuJ98bXntPXqoZqyTanq4c",
          "timestamp": "2026-03-26T13:46:49+02:00",
          "tree_id": "9b99cf8796e19887cb61d9f0f922f67049132aa6",
          "url": "https://github.com/fruch/cqlsh-rs/commit/139b8a09fd8e01f9b869955ecc7b6d372674ab82"
        },
        "date": 1774526161491,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 16444,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18272,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20543,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19194,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19011,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37678,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2787,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6134,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 45027,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5403,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12625,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 52600,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 97785,
            "range": "± 644",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 528,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 582,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1117,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1084,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 12879,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 51636,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21661,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 96726,
            "range": "± 1123",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "b2d4fbd282764815bcea7468111269e119ff09b8",
          "message": "feat(copy): COPY TO/FROM with CSV export and import\n\nCOPY TO: export table data to CSV file or STDOUT with 15 options\n(delimiter, quote, escape, header, null, datetime format, precision,\nbool style, page size, max output, progress reporting).\n\nCOPY FROM: import CSV data with 14 options (chunk size, batch size,\nprepared statements, TTL, max attempts, error limits, error file,\ningest rate, progress reporting, STDIN mode).\n\nAlso fix colorizer: don't highlight words after dot as keywords\n(e.g., test_ks.users no longer highlights 'users' as keyword).\n\n24 unit tests for COPY parsing and value formatting.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-26T13:47:44+02:00",
          "tree_id": "b070a6d45fdb550bfb2e3deddbdee35ff38402fd",
          "url": "https://github.com/fruch/cqlsh-rs/commit/b2d4fbd282764815bcea7468111269e119ff09b8"
        },
        "date": 1774526235644,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 15993,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 17854,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20394,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 18560,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 19291,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 36843,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2779,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6207,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44493,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5459,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12776,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 51149,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 95863,
            "range": "± 663",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 545,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 596,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1130,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1339,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14650,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 53347,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 22000,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 99515,
            "range": "± 599",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "noreply@anthropic.com",
            "name": "Claude",
            "username": "claude"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "1fc728166ba46c158815a33b3a5eb04c3be5fbd3",
          "message": "docs: update inline status markers across all SP01-SP19 plan documents\n\n- SP1-SP4: Change \"IN PROGRESS\" → \"COMPLETED\" with completion context\n- SP5-SP6: Add missing status lines, mark as COMPLETED\n- SP7: Add \"PARTIALLY COMPLETE\" status (19/25+ commands done)\n- SP8: Add \"PARTIALLY COMPLETE\" status (COPY TO done, COPY FROM pending)\n- SP9: Add \"COMPLETED\" status, check off all acceptance criteria\n- SP10: Add \"IN PROGRESS\" status (327 tests, infra pending)\n- SP11: Add \"IN PROGRESS\" status (startup benchmarks done)\n- SP12: Add \"NOT STARTED\" status\n- SP14-SP15: Add \"NOT STARTED\" status\n- SP16: Add \"COMPLETED\" status (fixes incorporated)\n- SP17-SP18: Add \"NOT STARTED\" status (post-v1)\n- SP19: Add \"IN PROGRESS\" status (analysis done, execution pending)\n- Check off completed acceptance criteria and research deliverables\n\nhttps://claude.ai/code/session_01QhVfCKDR6KRjKPdsJdEoxm",
          "timestamp": "2026-03-26T19:38:40+02:00",
          "tree_id": "9bf23f34e71d9da89a800d24a1c6ea0de20e15b0",
          "url": "https://github.com/fruch/cqlsh-rs/commit/1fc728166ba46c158815a33b3a5eb04c3be5fbd3"
        },
        "date": 1774547325344,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 15940,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 17965,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 20165,
            "range": "± 460",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 19462,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 18917,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37873,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2807,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6115,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44856,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5434,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12416,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 50857,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 94628,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 511,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 576,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1100,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1340,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14592,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 53627,
            "range": "± 911",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21329,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 99546,
            "range": "± 8259",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "254ffc632b606b591819e1c6bbe4587f14b04f79",
          "message": "fix(clippy): fix Rust 1.94 clippy warnings in copy.rs\n\n- Use .is_multiple_of() instead of manual % == 0 checks\n- Remove needless & in slice comparison\n- Collapse nested if statements\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-26T20:03:14+02:00",
          "tree_id": "34afe2b46fe3e27d746b34aa05c0c8a37589ba20",
          "url": "https://github.com/fruch/cqlsh-rs/commit/254ffc632b606b591819e1c6bbe4587f14b04f79"
        },
        "date": 1774548787808,
        "tool": "cargo",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 15985,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_only",
            "value": 18078,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/host_and_port",
            "value": 19515,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/execute_mode",
            "value": 18370,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/file_mode",
            "value": 18606,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "cli_parse_args/full_connection",
            "value": 37500,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cli_validate/valid_minimal",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2790,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6156,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 45443,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/0",
            "value": 5494,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/10",
            "value": 12876,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/50",
            "value": 53342,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_parse_scaling/certfiles/100",
            "value": 99214,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/all_defaults",
            "value": 532,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/cli_overrides_only",
            "value": 586,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1114,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/nonexistent_file",
            "value": 1340,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/minimal_file",
            "value": 14812,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "cqlshrc_load_file/full_file",
            "value": 54914,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/minimal",
            "value": 21962,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 100636,
            "range": "± 382",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "5913bfc3006a1f3db24dd8cd34b0189ee895badf",
          "message": "fix(ci): remove unsupported --output-format bencher flag\n\nCriterion 0.5 doesn't support --output-format bencher. Run cargo bench\nwithout format flags and update the summary script to parse both bencher\nand Criterion's default output format.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-28T23:43:19+03:00",
          "tree_id": "dff02b0662efbf0ab344c5379924745c2b9d9200",
          "url": "https://github.com/fruch/cqlsh-rs/commit/5913bfc3006a1f3db24dd8cd34b0189ee895badf"
        },
        "date": 1774731848117,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 52830,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 510300,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5090300,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6698,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 3854,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16187,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2848,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6117,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44339,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1103,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 97638,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "db826b3601a7019366593c0ec12738518103e3dd",
          "message": "feat(shell): Phase 4.1 non-interactive mode & shell improvements\n\n- Add stdin pipe/redirect detection: when stdin is not a TTY and --tty\n  is not set, skip the REPL and read CQL statements line-by-line\n- --tty flag now forces interactive REPL even when stdin is piped\n- Fix connection error exit code: 1 → 2 (distinct from CQL error = 1)\n- Suppress connection banner in stdin pipe mode\n- Extract execute_cql_reader<R: BufRead> generic, shared by -f and stdin\n- Add DEBUG/UNICODE handlers to non-interactive execution path so they\n  work correctly in -e, -f, and piped-stdin modes\n- Add 4 no-Docker CLI tests (exit code 2, --tty, stdin pipe)\n- Add 8 Docker integration tests for DEBUG, UNICODE, and stdin pipe mode\n- Update docs/progress.json: Phase 4 in_progress, 5 tasks completed\n\nCloses tasks 4.1.3, 4.1.4, 4.1.8, 4.1.9, 4.1.10\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-29T21:24:57+03:00",
          "tree_id": "144ff4c8d46ad37d8db6b435a53a4b0d6a0ea70a",
          "url": "https://github.com/fruch/cqlsh-rs/commit/db826b3601a7019366593c0ec12738518103e3dd"
        },
        "date": 1774809953859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 51831,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 501280,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5027600,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6667,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 3786,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16497,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2797,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6025,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 43810,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1104,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 97567,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "a3f35e6d11eba2304d1d1d5aed43630de6604c2a",
          "message": "docs(SP11): update benchmarking plan with actual CI results and optimization roadmap\n\nReplace approximate formatter/completion baseline values with actual CI measurements\n(criterion 0.5, GitHub Actions ubuntu-latest). Add Performance Analysis & Optimization\nRoadmap section with summary table and P1–P5 proposals. Mark 200× startup improvement\nacceptance criterion as confirmed.\n\nKey findings:\n- format_table/rows/100: 510 µs ✅ (under 1 ms target)\n- format_table_colored/rows/100: 1.215 ms ❌ (P1: fix color overhead)\n- Parser O(n) confirmed: 630–660 ns/stmt across 10–500 statements\n- All completion ops <40 µs ✅ (P3: phf/trie opportunity)\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-29T23:08:19+03:00",
          "tree_id": "ee79388c77df36cf1b0a8c9c18bf79baf8085b2f",
          "url": "https://github.com/fruch/cqlsh-rs/commit/a3f35e6d11eba2304d1d1d5aed43630de6604c2a"
        },
        "date": 1774816215904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 47037,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 479100,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 4880700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6000,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 34442,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 347440,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3618400,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 25643,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 25007,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 59213,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 2737,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 4,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 14784,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2725,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 5900,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 42355,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1104,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 90188,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "ccbc5c0f45a7a6c7a524d0c202cf24bf255b45e8",
          "message": "ci: fix integration tests by disabling Docker userland-proxy\n\nDocker's userland-proxy binds a host port via listen() to forward\ntraffic into the container. On GitHub Actions runners this port is\noften already in use, causing container startup to fail with\n\"address already in use\".\n\nDisabling the userland-proxy makes Docker use kernel NAT (iptables)\ninstead, eliminating the bind conflict. Also switches from nextest\nto cargo test with --test-threads=1 to guarantee the OnceLock\nsingleton container is used correctly (nextest's parallel runner\nwould spin up multiple containers).\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-29T23:53:31+03:00",
          "tree_id": "17fe2436cd21d5fbcd79cf38d5ac7c9948b7db06",
          "url": "https://github.com/fruch/cqlsh-rs/commit/ccbc5c0f45a7a6c7a524d0c202cf24bf255b45e8"
        },
        "date": 1774818926003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 52263,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 509600,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5004900,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6832,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 38382,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 360940,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3672300,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 29805,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 26797,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 63433,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 3731,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16303,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2848,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6277,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 45074,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1124,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 96533,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "cfbc051d5a36f16eecd0af1b8c5d9916567f743d",
          "message": "fix(test): cache container start failure in OnceLock to prevent retry storms\n\nOnceLock::get_or_init does not cache a panic — if the init closure\npanics, the lock stays uninitialized and the next test retries from\nscratch. With --test-threads=1 this caused every one of the 63 tests\nto attempt its own container start in sequence, each leaving partial\niptables rules that blocked the next attempt with \"address already\nin use\".\n\nSwitching to OnceLock<Result<…>> ensures the error is stored on the\nfirst failure, so all subsequent tests see the cached Err immediately\nwithout touching Docker again.\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-29T23:58:48+03:00",
          "tree_id": "c777239acdedb4daa418991e99c6bca48f1d5923",
          "url": "https://github.com/fruch/cqlsh-rs/commit/cfbc051d5a36f16eecd0af1b8c5d9916567f743d"
        },
        "date": 1774819249592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 52064,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 506660,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5049100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6834,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 38092,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 367410,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3686800,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 28413,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 26915,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 63873,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 3690,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16190,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2794,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6108,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44387,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1145,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 98421,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f0774c5c18de49a0553224cc1c9ebdd435e3bbb3",
          "message": "ci(integration): work around testcontainers Docker networking failure\n\ntestcontainers-rs fails on GitHub Actions (Ubuntu 24.04 / Docker 27)\nwith \"address already in use\" when Docker's networking driver tries\nto program external connectivity, even with userland-proxy disabled.\nRoot cause not yet identified.\n\nTemporary workaround: start ScyllaDB via plain docker run in CI and\npass SCYLLA_TEST_HOST/PORT env vars; helpers.rs bypasses testcontainers\nwhen those vars are set, falling back to testcontainers for local dev.\n\nTODO: restore testcontainers-managed lifecycle once the Docker\nnetworking issue on GHA runners is understood and fixed.\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-30T00:12:44+03:00",
          "tree_id": "140e9c33a88305c434eb894d267e78ce7c961a29",
          "url": "https://github.com/fruch/cqlsh-rs/commit/f0774c5c18de49a0553224cc1c9ebdd435e3bbb3"
        },
        "date": 1774820092798,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 53177,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 516240,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5070800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6911,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 37913,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 369780,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3727500,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 29211,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 26859,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 62885,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 3774,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16224,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2774,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6051,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44881,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1120,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 97159,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "7815b75cb482aa5da98ab4d1ef53a7fba69cc70d",
          "message": "fix(ci): fix integration test failures and CI infrastructure\n\nCode fixes:\n- src/main.rs: print banner in -e/-f mode (Python cqlsh compat); add\n  trailing semicolon to -e input so parse_batch doesn't silently drop it\n- src/parser.rs: track BEGIN BATCH…APPLY BATCH blocks so internal\n  semicolons don't split the statement\n\nIntegration test fixes:\n- tests/integration/helpers.rs: add with_semicolon() helper so all\n  execute_cql calls normalise input; change OnceLock<ScyllaContainer> to\n  OnceLock<Result<…,String>> so a failed container start is cached and\n  never retried (prevents Docker networking storm on subsequent tests)\n- tests/integration/unicode_tests.rs: escape apostrophe as '' in CQL;\n  add graceful skip when ScyllaDB rejects non-ASCII table names\n- tests/integration/output_tests.rs: new banner and stdin-pipe tests\n- tests/integration/escape_tests.rs: minor fixes\n\nCI infrastructure:\n- .github/workflows/ci.yml: switch integration job from nextest to\n  cargo test (shared OnceLock across all tests); add --test-threads=1\n  to serialise tests; disable Docker userland-proxy to fix \"address\n  already in use\" port-forwarding errors on ubuntu-latest runners\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-30T00:19:28+03:00",
          "tree_id": "999b8b51bc804966ecea9045a2ffff0322983203",
          "url": "https://github.com/fruch/cqlsh-rs/commit/7815b75cb482aa5da98ab4d1ef53a7fba69cc70d"
        },
        "date": 1774820500130,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 54335,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 527690,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5186700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 6966,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 38871,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 364200,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3721400,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 27914,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 28039,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 63809,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 4192,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 4,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16186,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2797,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6076,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44047,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1148,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 96046,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "940f113d8da90e2cd372b28e903e67051eaa5091",
          "message": "test(SP19): add DESCRIBE, COPY TO/FROM, and SSL integration tests (Phase 6)\n\nExtends the integration test suite with three new test modules:\n\n- describe_tests.rs — 18 tests covering DESCRIBE KEYSPACE(S), TABLE(S),\n  INDEX, TYPE(S), CLUSTER, SCHEMA, FULL SCHEMA, MATERIALIZED VIEW, FUNCTION,\n  AGGREGATE, not-found errors, and tables with keyword column names.\n  Merged with main's Phase 4 describe tests; duplicate test_describe_index\n  renamed to test_describe_index_on_non_pk_column.\n\n- copy_tests.rs — 7 tests for COPY TO, COPY FROM, round-trip, HEADER,\n  NULL indicator, collection types, and wrong column count. Each test probes\n  whether COPY is dispatched in -e mode (Phase 4); skips gracefully with\n  eprintln! when the server returns SyntaxException.\n\n- ssl_tests.rs — 4 tests verifying --ssl fails gracefully on a non-TLS\n  server, plain connection baseline, --ssl + auth no-panic, and\n  connect-timeout bounded.\n\nAlso updates docs/progress.json (Phase 5: 6 → 9 completed tasks).\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-30T00:34:20+03:00",
          "tree_id": "013a8e0fae1031b5700a0ef9c0b70084132412f1",
          "url": "https://github.com/fruch/cqlsh-rs/commit/940f113d8da90e2cd372b28e903e67051eaa5091"
        },
        "date": 1774821402014,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 52844,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 519760,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5169100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 7000,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 38938,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 370060,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3744700,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 28182,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 28059,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 63740,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 4087,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16552,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2810,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6068,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 44577,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1146,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 97069,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "e29ddd0d74995db4c3104dc7b66ec7fdddd06b33",
          "message": "feat(copy): implement COPY FROM with full type conversion and advanced options\n\nImplements Phase 1 (core) and Phase 2 (advanced) of COPY FROM:\n\n- csv_str_to_cql_value(): type-aware CSV→CQL conversion for all 25 CQL\n  types (ascii, text, varchar, int, bigint, smallint, tinyint, float,\n  double, boolean, uuid, timeuuid, timestamp, date, time, inet, blob,\n  varint, decimal, duration; collections fall back to Text passthrough)\n- cql_value_to_insert_literal(): proper CQL literal formatting for the\n  unprepared INSERT string path\n- Dual execution path: PREPAREDSTATEMENTS=true (default) uses prepared\n  statements with Vec<Option<ScyllaCqlValue>> binding; false uses string\n  INSERT\n- insert_row_with_retry(): exponential backoff up to MAXATTEMPTS\n- NUMPROCESSES: parallel inserts via futures::StreamExt::buffer_unordered\n  (no Arc/threading needed — futures borrow &CqlSession cooperatively)\n- INGESTRATE: TokenBucket rate limiter with async tokio::time::sleep\n- CHUNKSIZE buffering, TTL clause on INSERT, elapsed timing in output\n- MAXPARSEERRORS / MAXINSERTERRORS counting, ERRFILE option parsed\n- Fix ScyllaDriver::execute_prepared to actually bind values (was a stub)\n  via internal_to_scylla_cql() reverse mapping for all 25 types\n- Expose CqlSession::execute_prepared delegation\n- 41 unit tests covering csv_str_to_cql_value, option parsing, types\n- 10 integration test stubs in tests/integration/copy_from_tests.rs\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-30T00:42:59+03:00",
          "tree_id": "71e195fd0eb0e8e699747f24d4b4d4d2dff29391",
          "url": "https://github.com/fruch/cqlsh-rs/commit/e29ddd0d74995db4c3104dc7b66ec7fdddd06b33"
        },
        "date": 1774821908109,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 53481,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 520990,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5158700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 7085,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 38260,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 362590,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 3680600,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 28634,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 26593,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 63653,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 4028,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 4,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 16315,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2806,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 6084,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 43901,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1143,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 96660,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "ee928b4e53701b96a57c3a8ed2f4090e612abc56",
          "message": "fix(ci): restore customSmallerIsBetter tool for benchmark action\n\nThe Node.js 24 upgrade commit (3728dc6) regressed the benchmark-action\ntool from 'customSmallerIsBetter' back to 'criterion', which is no\nlonger a valid value in github-action-benchmark@v1.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T00:11:18+03:00",
          "tree_id": "78d16d2587c5bdf0f4d07b82ed81c8b9bc6a09ce",
          "url": "https://github.com/fruch/cqlsh-rs/commit/ee928b4e53701b96a57c3a8ed2f4090e612abc56"
        },
        "date": 1774906404547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 83749,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 797700,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7946500,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9982,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 60226,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 565950,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5751700,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 38152,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 36431,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 98889,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 10238,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28925,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3546,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9257,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64556,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1218,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153560,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "00a464f6159b2200737161334fffb3ac2e88a76e",
          "message": "ci: remove beta toolchain from test and build matrices\n\nDrop the Rust beta toolchain from the CI matrix, keeping only stable.\nThis halves the test and build job count (6 → 3 each) without losing\nmeaningful coverage for this project.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T00:20:25+03:00",
          "tree_id": "a189c7987794944537548281fa97af109d38ff03",
          "url": "https://github.com/fruch/cqlsh-rs/commit/00a464f6159b2200737161334fffb3ac2e88a76e"
        },
        "date": 1774906951321,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 83738,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 800840,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7957200,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10032,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 60236,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 558920,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5690000,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 37262,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 36423,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 99649,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 10255,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28861,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3664,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9412,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65223,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1234,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154080,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "5376116c548aaa7cfa8df21aeb3598b215720e11",
          "message": "fix(ci): update macOS runner from macos-13 to macos-14\n\nThe macos-13 runner has been deprecated by GitHub, causing the\nx86_64-apple-darwin release build to fail. macOS 14 runners support\ncross-compilation to x86_64 via Rosetta 2.\n\nFixes: https://github.com/fruch/cqlsh-rs/actions/runs/23767706685\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T00:50:37+03:00",
          "tree_id": "6369d39b0a11425d313d807c087d36dc2f791d4d",
          "url": "https://github.com/fruch/cqlsh-rs/commit/5376116c548aaa7cfa8df21aeb3598b215720e11"
        },
        "date": 1774908700076,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 85297,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 790910,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7896700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9818,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 60208,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 563450,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5750200,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 38116,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 36287,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 99737,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 10219,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28915,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3580,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9220,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64952,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1207,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 151260,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "02ba3ab95fdafeea1e1a384c04a9807027e0246f",
          "message": "docs: facelift README with community-standard style\n\nAdd badges (CI, crates.io, license, Docker), feature highlights,\nquickstart section, and structured installation methods (Homebrew,\nCargo, Docker, pre-built binaries, from source). Update project\nstructure to reflect current 19-file layout. Collapse benchmarks\nand testing into details sections. Add contributing section.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T00:52:40+03:00",
          "tree_id": "4e83d6d19919cbdfbf0fc95ec7dda0e7fd8c5fa6",
          "url": "https://github.com/fruch/cqlsh-rs/commit/02ba3ab95fdafeea1e1a384c04a9807027e0246f"
        },
        "date": 1774908805125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 72594,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 704560,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7085400,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 8336,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 54069,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 520179,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5359300,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 33370,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 35118,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 89355,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7233,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 26430,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3623,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9071,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 60793,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1186,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 134500,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f400147df83d08d68cfc1ca259e1f8043e1a3cc3",
          "message": "fix(docs): fix link checker failures in CI\n\n- Rename README.md to index.md in commands/ and configuration/ dirs\n  (mdBook renders README.md as index.html, causing broken README.html links)\n- Update all internal references to use index.md\n- Remove site-url from book.toml to avoid root-relative link in 404.html\n- Exclude 404.html, font license files, and adobe.com from lychee checks\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T01:06:45+03:00",
          "tree_id": "11a47a6046b7f51b0ad77cd983e84611ee02f4ea",
          "url": "https://github.com/fruch/cqlsh-rs/commit/f400147df83d08d68cfc1ca259e1f8043e1a3cc3"
        },
        "date": 1774909688887,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 83506,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 788260,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7890800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9932,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 59950,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 562700,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5741300,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 38025,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 35987,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 98158,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 10244,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28978,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3684,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9388,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65345,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1215,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154470,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "52f4382c6d9dfe856cd62aecb6ddb20fe2131484",
          "message": "ci: configure Renovate for Rust and GitHub Actions dependency updates\n\nAdds renovate.json with:\n- Grouped automerge for Rust minor/patch updates\n- Separate PRs (no automerge) for Rust major updates\n- Grouped automerge for GitHub Actions updates\n- Lock file maintenance enabled\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T01:15:14+03:00",
          "tree_id": "32d392dd9f38a5bd7d782d2b6b3fbebec433f0e7",
          "url": "https://github.com/fruch/cqlsh-rs/commit/52f4382c6d9dfe856cd62aecb6ddb20fe2131484"
        },
        "date": 1774910170523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 83671,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 791190,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7912300,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9808,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 59877,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 564500,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5751900,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 37322,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 35153,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 98491,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 10216,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 29121,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3605,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9359,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65041,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1208,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153500,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "4194ad9c1a43e179cfec1198343ecee128db33ae",
          "message": "chore(deps): update dependency python to 3.14",
          "timestamp": "2026-03-31T08:46:12+03:00",
          "tree_id": "f2475f4f8a190fd3b0ad8ffb64ebc0bd5d4611e9",
          "url": "https://github.com/fruch/cqlsh-rs/commit/4194ad9c1a43e179cfec1198343ecee128db33ae"
        },
        "date": 1774937232111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 84580,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 785620,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7866600,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10093,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 59477,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 557600,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5709700,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 38810,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 35532,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 98139,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 10215,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28643,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3621,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9375,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65122,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1175,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154300,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "3ed7ab1c123d84000930b78add09bf0219cb342c",
          "message": "chore(deps): update alpine docker tag to v3.23",
          "timestamp": "2026-03-31T08:46:28+03:00",
          "tree_id": "6d095ab2232f1278923e51b7080d80059a9a1673",
          "url": "https://github.com/fruch/cqlsh-rs/commit/3ed7ab1c123d84000930b78add09bf0219cb342c"
        },
        "date": 1774937243452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 72262,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 711620,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7094200,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 8315,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 53965,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 519080,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5355200,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 33256,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 35156,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 89474,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7041,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 26431,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3368,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 8654,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 60389,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1163,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 133420,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "e732569f03115b8009e241127e4fa053cf02c0e3",
          "message": "fix(deps): update rust crate rustyline to v18",
          "timestamp": "2026-03-31T08:46:47+03:00",
          "tree_id": "6ca0197dca7248802ea11093f48f03e052a08357",
          "url": "https://github.com/fruch/cqlsh-rs/commit/e732569f03115b8009e241127e4fa053cf02c0e3"
        },
        "date": 1774937348774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 85800,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 806760,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 8023999,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10072,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 59634,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 552310,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5636500,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 37153,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37146,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 95836,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 9861,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28860,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 4,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3584,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9177,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63615,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1219,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154650,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "dba42d17f05d0eb8cdc37ae98869fb4fc12e6c72",
          "message": "fix(ci): fix GitHub Pages docs deployment conflicts\n\nThe docs site at https://fruch.github.io/cqlsh-rs/ wasn't working because\nbench.yml had a competing deploy-pages job that overwrote the mdBook\ndeployment with just benchmark data.\n\nChanges:\n- Remove deploy-pages job from bench.yml (benchmarks still push to\n  gh-pages via auto-push; the docs workflow handles Pages deployment)\n- Add benchmark dashboard merge step to docs.yml so /dev/bench/ is\n  included in the deployed site alongside mdBook and rustdoc\n- Trigger docs.yml on src/** and Cargo.toml changes (for rustdoc updates)\n  and on workflow_dispatch (for manual re-deploy)\n- Add documentation links to README (docs site, API reference, benchmarks)\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T14:42:36+03:00",
          "tree_id": "031feb2d1c47f561c398d63defe9b8aa4f7d607b",
          "url": "https://github.com/fruch/cqlsh-rs/commit/dba42d17f05d0eb8cdc37ae98869fb4fc12e6c72"
        },
        "date": 1774958718561,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "format_table/rows/10",
            "value": 83566,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 789230,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7887800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10123,
            "unit": "ns"
          },
          {
            "name": "format_table/10",
            "value": 60674,
            "unit": "ns"
          },
          {
            "name": "format_table/100",
            "value": 567030,
            "unit": "ns"
          },
          {
            "name": "format_table/1000",
            "value": 5727000,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 37438,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37385,
            "unit": "ns"
          },
          {
            "name": "format_each_type",
            "value": 98009,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 9864,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "cli_parse_args/no_args",
            "value": 28995,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3605,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9196,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63235,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1195,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 152820,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "00c0cd16a3d829651e7f3e864ba1425ca31d9737",
          "message": "perf(ci): reduce benchmark CI time by lowering criterion warmup/measurement\n\n- Pass --warm-up-time 1 --measurement-time 1 --sample-size 20 to\n  criterion in CI, reducing per-benchmark time from ~10s to ~2s\n  (83 benchmarks: ~14min → ~3min)\n- Remove unused CRITERION_PROFILE env var (no config existed for it)\n- Replace deprecated criterion::black_box with std::hint::black_box\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-03-31T19:17:45+03:00",
          "tree_id": "400cebd0337a8511caa6669976b7fd874963b539",
          "url": "https://github.com/fruch/cqlsh-rs/commit/00c0cd16a3d829651e7f3e864ba1425ca31d9737"
        },
        "date": 1774974406042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29077,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3685,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9162,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 62856,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 860,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 147640,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 9733,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 81679,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 734860,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7330400,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9972,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 50524,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 38932,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "copilot-swe-agent[bot]",
            "username": "Copilot"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "b469875c787251c263f928812f5a5b5e4d11b01b",
          "message": "fix(ci): exclude benchmark dashboard URL from link checker\n\nThe benchmark dashboard at scylladb.github.io/cqlsh-rs/dev/bench/ is\ndeployed separately by the benchmark workflow and may not exist until\nthat workflow runs on main. Exclude it from lychee link checking to\nprevent false 404 failures.\n\nAgent-Logs-Url: https://github.com/scylladb/cqlsh-rs/sessions/8f4ddcab-f2a3-4270-bc8a-5af5827d66e1\n\nCo-authored-by: fruch <340979+fruch@users.noreply.github.com>",
          "timestamp": "2026-04-16T22:43:00+03:00",
          "tree_id": "fc71201dd421da5f11c591fa38ce9f1e5341ea35",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/b469875c787251c263f928812f5a5b5e4d11b01b"
        },
        "date": 1776369191945,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29057,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 4556,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 10110,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63839,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 971,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153520,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 9808,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 7,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85957,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 793350,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7884200,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9585,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 47009,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39647,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "copilot-swe-agent[bot]",
            "username": "Copilot"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "d72fc9e6a524db6a84d2df5d01dbed1a56eb1dcd",
          "message": "fix: remove unnecessary .into_iter() calls to fix clippy warnings\n\nRemove redundant .into_iter() calls that trigger clippy::useless_conversion\nlint errors on Rust 1.95.0:\n- src/copy.rs:1433 - futures::stream::iter() accepts IntoIterator directly\n- src/describe.rs:591 - Iterator::zip() accepts IntoIterator directly\n\nAgent-Logs-Url: https://github.com/scylladb/cqlsh-rs/sessions/e302ef36-eb5c-4061-9320-f9ae7bacb9e4\n\nCo-authored-by: fruch <340979+fruch@users.noreply.github.com>",
          "timestamp": "2026-04-19T09:44:14+03:00",
          "tree_id": "31e764502f1b9947d82a8c44cb91bbb2266d9e0b",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/d72fc9e6a524db6a84d2df5d01dbed1a56eb1dcd"
        },
        "date": 1776581614753,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 28950,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3590,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9029,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63741,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 833,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 146460,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 9860,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 82643,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 740780,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7450900,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9533,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46020,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37690,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "6e3bc131cf0511d22503d33d0cf063a13df0af5b",
          "message": "perf(cql_lexer): replace tokenize-based strip_comments with zero-alloc scanner\n\nThe previous strip_comments delegated to tokenize(), which allocates a\nVec<Token> with String copies per token. The parse_multiline/6_lines\nbenchmark regressed from ~10µs to ~25µs (2.4x) as a result.\n\nReplace with a dedicated single-pass scanner that copies input slices\ndirectly into the result String with no intermediate allocations.\nFixes the perf regression while keeping the logic in cql_lexer.\n\nCo-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-04-19T10:03:13+03:00",
          "tree_id": "bce673a2228c1c6cb5cad72825866b311f32cab0",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/6e3bc131cf0511d22503d33d0cf063a13df0af5b"
        },
        "date": 1776582754773,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29377,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3458,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9012,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63029,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 860,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154200,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6761,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 7,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 83207,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 763140,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7607800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9800,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 45899,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 38589,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f902ff4a1862aeb14dfd77c8ec6078cc86f9bbe0",
          "message": "fix: use microsecond precision (6 digits) for timestamp formatting\n\nAligns timestamp output with Python cqlsh by using 6-digit microsecond\nprecision. Also fixes clippy useless_conversion warnings in copy.rs and\ndescribe.rs.",
          "timestamp": "2026-04-19T12:02:53+03:00",
          "tree_id": "d48ab7dae75bc18885ec5d8b5b16a5d2579c6917",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/f902ff4a1862aeb14dfd77c8ec6078cc86f9bbe0"
        },
        "date": 1776589896110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 22799,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 2785,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 7308,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 49662,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 678,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 112200,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 5425,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 6,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 63140,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 571040,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 5722000,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 7706,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 36184,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 29799,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "1380f7aca3fa8563902544a5e07f4be875229798",
          "message": "fix: print connect/request timeout and ssl in --debug mode\n\nDisplay connection timeout, request timeout, and SSL status in the\ndebug output when --debug flag is passed, matching Python cqlsh behavior.\n\nCloses #65",
          "timestamp": "2026-04-19T12:03:09+03:00",
          "tree_id": "b7bb310d6f6f95b1c2a791b45f568a79ef18f9db",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/1380f7aca3fa8563902544a5e07f4be875229798"
        },
        "date": 1776589941598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29712,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3421,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9038,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63320,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 851,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 152890,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6752,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 7,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 83460,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 760280,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7591700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9660,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 44043,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 38863,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "adc9af0cdda50bf58bb87c3bc7b53a3d35ee3be8",
          "message": "fix: emit ANSI clear sequences for CLEAR/CLS in non-interactive mode\n\nCLEAR/CLS was silently ignored in non-interactive mode, but Python cqlsh\nemits ANSI escape sequences (\\x1B[2J\\x1B[1;1H) which dtests assert on.\nMove CLEAR/CLS handling before the ignore block to match Python behavior.\n\nCloses #66",
          "timestamp": "2026-04-19T12:03:23+03:00",
          "tree_id": "cd9f34192397b054b9432e3ac0a6881e479137b9",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/adc9af0cdda50bf58bb87c3bc7b53a3d35ee3be8"
        },
        "date": 1776589961707,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 28613,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3595,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9339,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64799,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 843,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 144660,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6833,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 83709,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 739780,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7460100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9646,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46907,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37880,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "3eff998e18b6f2e723efe20cbc669c8b575e438f",
          "message": "fix: display null values as blank and trim trailing whitespace\n\nMatches Python cqlsh behavior where null column values display as\nempty cells rather than the literal text 'null'. Also trims trailing\nwhitespace from tabular output lines.\n\nCloses #71",
          "timestamp": "2026-04-19T13:40:53+03:00",
          "tree_id": "a665dc559125c2ce96252b6541b57f1836d216fb",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/3eff998e18b6f2e723efe20cbc669c8b575e438f"
        },
        "date": 1776595812523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 28158,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3572,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9160,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63412,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 834,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 144910,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7257,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 83585,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 748820,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7453500,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10495,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 49566,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 38347,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "47e5295767aae56dcadeaac2cf54ba7fafc2aa80",
          "message": "fix: include tables and indexes in DESCRIBE KEYSPACE output\n\nDESCRIBE KEYSPACE previously only returned the CREATE KEYSPACE statement.\nNow it also outputs CREATE TABLE and CREATE INDEX statements for all\nobjects within the keyspace, matching Python cqlsh behavior.\n\nAlso updates DESCRIBE SCHEMA / DESCRIBE FULL SCHEMA to include indexes\nafter each table definition.\n\nCloses #63",
          "timestamp": "2026-04-19T13:41:31+03:00",
          "tree_id": "61caa1bf621be73a4e12b686398e110249f01d9f",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/47e5295767aae56dcadeaac2cf54ba7fafc2aa80"
        },
        "date": 1776595855631,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 28548,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3724,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9443,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63316,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 841,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 152560,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7210,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 82752,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 743510,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7599900,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10200,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 48570,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37795,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "925db352f77f711f8c679351e70c40fec1f04478",
          "message": "fix: suppress tracing output for queries against system_traces\n\nSkip tracing for queries targeting the system_traces keyspace to avoid\nrecursive trace queries, matching Python cqlsh behavior.\n\nCloses #72",
          "timestamp": "2026-04-19T14:01:20+03:00",
          "tree_id": "66e92426d7bf40d7d790720958b0ec197053348d",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/925db352f77f711f8c679351e70c40fec1f04478"
        },
        "date": 1776597036973,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 28784,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3483,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9076,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 62631,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 826,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153740,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6909,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 7,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 84748,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 773050,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7699100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10556,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46527,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39211,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "256dd92291564ea9a2be89b6bfd1faa65e601c71",
          "message": "test: implement COPY FROM integration test stubs with real test logic",
          "timestamp": "2026-04-19T14:01:55+03:00",
          "tree_id": "af5cdf5f858d785ee0f3f241edbc57bb6bdbde6e",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/256dd92291564ea9a2be89b6bfd1faa65e601c71"
        },
        "date": 1776597071079,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29375,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3573,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9272,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63365,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 820,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 144650,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7249,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 83969,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 745230,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7489100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10108,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 48402,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 38875,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "762b13689c541c33ec972e442194eefa431d831a",
          "message": "test: add integration tests for all remaining CLI flags\n\nAdd 22 new integration tests covering every previously untested CLI flag:\n--ssl, --no-file-io, --coverage, -k, -u, -p, --request-timeout,\n--encoding, --cqlversion, --protocol-version, --consistency-level,\n--serial-consistency-level, --no_compact, --disable-history, -b,\n--browser, --generate-man, -C, --no-color, -e, -f, and\nprotocol version out-of-range validation.\n\nTotal CLI integration tests: 22 → 44.",
          "timestamp": "2026-04-19T14:02:08+03:00",
          "tree_id": "c48c86ac9e3e61f06b60b6a9745e17eb85344066",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/762b13689c541c33ec972e442194eefa431d831a"
        },
        "date": 1776597086080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 28716,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3660,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9473,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65281,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 825,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154320,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7415,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 8,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 82796,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 744390,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7483900,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10055,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 49402,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37750,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "copilot-swe-agent[bot]",
            "username": "Copilot"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "ca57214fb07881711c030554d7dd703337ecd5aa",
          "message": "fix(ci): address code review feedback on release pipeline\n\n- Remove CARGO_REGISTRY_TOKEN from release-pr step (only needed for\n  the release/publish step)\n- Add explicit permissions: contents: read to build-release-binaries\n  and generate-release-assets jobs\n\nAgent-Logs-Url: https://github.com/scylladb/cqlsh-rs/sessions/bad47fa5-7ecb-4300-ae35-7f65791d62d8\n\nCo-authored-by: fruch <340979+fruch@users.noreply.github.com>",
          "timestamp": "2026-04-19T14:29:37+03:00",
          "tree_id": "58f7b263b6b1fb8b64d4e77497368a49d0a0f60c",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/ca57214fb07881711c030554d7dd703337ecd5aa"
        },
        "date": 1776598769523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 25621,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3246,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 8513,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 58155,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 987,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 133210,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 5406,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 5,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 78624,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 704480,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7099600,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 8414,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 39354,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 34091,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "851673ee4f3e2bf1cf98c258558d006aff5fc20c",
          "message": "feat: add --cqlversion and --protocol-version compatibility warnings\n\nThe scylla-rust-driver auto-negotiates protocol version and hardcodes\nCQL_VERSION=4.0.0 in the STARTUP frame. Both flags are accepted for CLI\ncompatibility with Python cqlsh but emit warnings explaining the driver\nlimitations.\n\n- --cqlversion: warns when requested version differs from server's CQL spec\n- --protocol-version: warns that the driver auto-negotiates\n- Add CLI tests for both flags\n- Add integration-output.log to .gitignore",
          "timestamp": "2026-04-19T14:29:53+03:00",
          "tree_id": "dd04d6a4c027bd3c670f132373d2a6c0cbc67210",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/851673ee4f3e2bf1cf98c258558d006aff5fc20c"
        },
        "date": 1776598791538,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29322,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3477,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9109,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 62437,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 824,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 152350,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7011,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 7,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85082,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 772990,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7737300,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10578,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46811,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39762,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f4b376f62bb4c46831c17f2c5c6125502e95ddf6",
          "message": "fix: gracefully skip SSL tests when TLS container fails to start",
          "timestamp": "2026-04-19T14:30:17+03:00",
          "tree_id": "c1913676c7a016ba951d08104f510998e8c968c0",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/f4b376f62bb4c46831c17f2c5c6125502e95ddf6"
        },
        "date": 1776598822885,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29706,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3404,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9294,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63049,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 823,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153690,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7061,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 7,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85104,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 783580,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7745700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10037,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46512,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39581,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "438b6caeaa553c2763860907cbde52740b1012fe",
          "message": "fix(deps): update rust dependencies",
          "timestamp": "2026-04-19T14:30:42+03:00",
          "tree_id": "b784fdcf3bcf1839978b7d957d739882226f5b5b",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/438b6caeaa553c2763860907cbde52740b1012fe"
        },
        "date": 1776598846711,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29118,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3731,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9407,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63781,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 816,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 145120,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7187,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 13,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 84076,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 755030,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7627100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10392,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 48645,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39594,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "4e8904aef89940a52884e5a01ed99d745db57707",
          "message": "fix: correct debug output assertion for encoding in ui_encoding_from_cqlshrc test",
          "timestamp": "2026-04-19T14:30:54+03:00",
          "tree_id": "af6945a0ad9ec903ed33efd4d8e5e8b2341dc02f",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/4e8904aef89940a52884e5a01ed99d745db57707"
        },
        "date": 1776598847579,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 26992,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3322,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 8572,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 58118,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 1001,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 132220,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 5398,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 10,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 79564,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 716150,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7213600,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 8335,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 39700,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 36677,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "8503f56e381a8a42b7e2d06d37634b5b5d4be253",
          "message": "fix: handle SOURCE command client-side in non-interactive mode\n\nProcess SOURCE commands locally by reading and executing the referenced\nCQL file, instead of sending SOURCE as a CQL query to the server.\n\nCloses #67",
          "timestamp": "2026-04-19T14:31:04+03:00",
          "tree_id": "a65c3c5f69f4a06e94ed36be40f5f453540e9e41",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/8503f56e381a8a42b7e2d06d37634b5b5d4be253"
        },
        "date": 1776598866888,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29245,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3710,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9524,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65461,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 851,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 145040,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7173,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 13,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 91230,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 760960,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7518200,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10112,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 47268,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39203,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "6fd6efbe567bef9631f586709c13523a86aefa0f",
          "message": "fix: correct DESCRIBE TABLE column ordering, add WITH clause, and include materialized views in DESCRIBE KEYSPACE\n\n- Fix column ordering in DESCRIBE TABLE output to match schema definition\n  order (partition keys → clustering keys → regular columns) instead of\n  alphabetical order (#89)\n- Add WITH clause to DESCRIBE TABLE including CLUSTERING ORDER BY and\n  table properties (compaction, compression, gc_grace_seconds, etc.) (#90)\n- Add materialized view definitions to DESCRIBE KEYSPACE output by\n  querying system_schema.views (#91)\n\nCloses #89, closes #90, closes #91",
          "timestamp": "2026-04-19T15:18:00+03:00",
          "tree_id": "897b76f2609e754a968fe46ed23425d10548d15c",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/6fd6efbe567bef9631f586709c13523a86aefa0f"
        },
        "date": 1776601604417,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29357,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3659,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9250,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64288,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 836,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 149020,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7135,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 13,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 82546,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 745010,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7456700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9790,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 45427,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37730,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "copilot-swe-agent[bot]",
            "username": "Copilot"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "defec268b1f36be972f219d142e14562d6807b51",
          "message": "fix(ci): use RELEASE_PLZ_TOKEN for release-pr step\n\nThe default GITHUB_TOKEN is not permitted to create PRs in this\nrepo (HTTP 403). Instead of removing the release-pr step (which\nwould break CHANGELOG.md auto-updates), use a dedicated PAT\nstored as the RELEASE_PLZ_TOKEN secret.\n\nSetup required:\n  1. Create a fine-grained PAT with Contents + Pull Requests\n     read/write scopes for scylladb/cqlsh-rs\n  2. Add it as a repository secret named RELEASE_PLZ_TOKEN\n\nAgent-Logs-Url: https://github.com/scylladb/cqlsh-rs/sessions/2416f025-e3cf-49d5-81ae-3b9ad67e8c01\n\nCo-authored-by: fruch <340979+fruch@users.noreply.github.com>",
          "timestamp": "2026-04-19T17:45:16+03:00",
          "tree_id": "cf94ba69cd07b91d2e473ceb2789eaad44331265",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/defec268b1f36be972f219d142e14562d6807b51"
        },
        "date": 1776610444356,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29320,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 2,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3634,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9234,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64247,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 827,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 147050,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7011,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 13,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 82875,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 744020,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7455800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9837,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 45830,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 37913,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "d4a38fbb162f6f5009b111b117967d99f86c5a87",
          "message": "chore: release v0.2.0",
          "timestamp": "2026-04-19T18:01:00+03:00",
          "tree_id": "a0918193deb3ea6f4c6cb842d2347cff12879d68",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/d4a38fbb162f6f5009b111b117967d99f86c5a87"
        },
        "date": 1776611389182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 30326,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3448,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9139,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63631,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 836,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 154800,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6770,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 11,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85931,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 787610,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 8078400,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10024,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 44575,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40022,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f0d4947383c8b8be2bf1c7fc6062e0311ff2995d",
          "message": "fix: add WITH clause to DESCRIBE MATERIALIZED VIEW output\n\nAdd CLUSTERING ORDER BY and table properties (bloom_filter_fp_chance,\ncaching, compaction, compression, etc.) to DESCRIBE MATERIALIZED VIEW\noutput, matching Python cqlsh behavior.\n\n- Add properties field to MvDdlParts struct\n- Fetch properties from system_schema.views\n- Always emit CLUSTERING ORDER BY for MVs (matching Python cqlsh)\n- Add format_create_mv_ddl_with_properties test\n\nCloses #90",
          "timestamp": "2026-04-20T14:43:54+03:00",
          "tree_id": "b04eac112c4a7446ffecf4eb3174c24ef47764a2",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/f0d4947383c8b8be2bf1c7fc6062e0311ff2995d"
        },
        "date": 1776685960923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 30053,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3449,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9131,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63458,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 839,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 155720,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6466,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 12,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85639,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 778080,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7810900,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9828,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 48832,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40332,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "9751a4c9c780c0a1187a32493d76b6a6e4a1dae5",
          "message": "fix(describe): restore accidentally deleted format_property_value function\n\nThe function was removed in 4703a88 as collateral damage from an\nunrelated COPY TO/FROM fix, breaking compilation on main.",
          "timestamp": "2026-04-20T14:56:44+03:00",
          "tree_id": "00519f66f799d92ee9f0d2465c805f728f844a01",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/9751a4c9c780c0a1187a32493d76b6a6e4a1dae5"
        },
        "date": 1776686730724,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29503,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3458,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9268,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65917,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 823,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 165760,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6316,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 12,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 86105,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 785640,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7834500,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10215,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46541,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40017,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "9b001dcc40fe6ebc64f534d67400dabebccc152a",
          "message": "chore: release v0.3.0",
          "timestamp": "2026-04-20T15:07:40+03:00",
          "tree_id": "4c236133f2d7386173447bcea2592275c803b6c7",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/9b001dcc40fe6ebc64f534d67400dabebccc152a"
        },
        "date": 1776687397494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29777,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3420,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9150,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64896,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 827,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 155150,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6591,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 12,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 86841,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 798750,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7957000,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9758,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 45609,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40656,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "copilot-swe-agent[bot]",
            "username": "Copilot"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "0690fc25951b223f05b00a0298a6e2b1b4040056",
          "message": "ci(release): require integration tests to pass before creating release PRs\n\nAdd `integration` to the `needs` array of the `release` job in ci.yml.\nPreviously, release-plz could create release PRs even when integration\ntests were failing, since only fmt, clippy, test (unit), and build were\nrequired. This ensures ALL CI checks must pass before version bumps.\n\nCloses the gap identified in PR #99 where a release PR was created\nwhile the main branch had a broken build.\n\nAgent-Logs-Url: https://github.com/scylladb/cqlsh-rs/sessions/0e9ba6c0-d624-4036-8e56-8e7a1ebd2fc4\n\nCo-authored-by: fruch <340979+fruch@users.noreply.github.com>",
          "timestamp": "2026-04-20T16:46:58+03:00",
          "tree_id": "ca8d45f693190673651440088fe3a81f2597e1cd",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/0690fc25951b223f05b00a0298a6e2b1b4040056"
        },
        "date": 1776693342065,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29099,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3619,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9208,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64081,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 828,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 144850,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6422,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 14,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 84561,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 752590,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7548600,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10120,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 47020,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 38063,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "2e9b5533610e67ab033db44fadb0f95e4211271f",
          "message": "feat(cli): include git SHA in --version output\n\nAdd build.rs to embed the git commit hash at compile time. The version\nstring now shows 'cqlsh-rs 0.2.0 (abc1234)' or 'cqlsh-rs 0.2.0 (abc1234-dirty)'\nwhen there are uncommitted changes. Tracks HEAD and git index for\naccurate rebuild triggers.",
          "timestamp": "2026-04-20T18:09:51+03:00",
          "tree_id": "02e90988a7f0302f2311b0e30bb72c58a98d6cce",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/2e9b5533610e67ab033db44fadb0f95e4211271f"
        },
        "date": 1776698312864,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29647,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3465,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9186,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64370,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 844,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 155270,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6625,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 12,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 84555,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 776680,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7778800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9778,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 45738,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 41042,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f97f99bb46d004bd637fb6deb75e3be935b3d8e5",
          "message": "fix: handle BATCH statements without inner semicolons via stdin/pipe\n\nWhen BATCH statements have no semicolons after inner DML statements\n(only APPLY BATCH; has one), the parser entered batch mode on the only\nsemicolon but never emitted the statement. Now detects when a single\nsemicolon both starts and ends a batch, emitting immediately.\n\nCloses #106",
          "timestamp": "2026-04-20T18:44:44+03:00",
          "tree_id": "344ce1e6578a15d2707f54ed8d80f6a18cc5b022",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/f97f99bb46d004bd637fb6deb75e3be935b3d8e5"
        },
        "date": 1776700408911,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29603,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3505,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9065,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63130,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 850,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153220,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6969,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 11,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 86823,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 788830,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7926800,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9862,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46516,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40774,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "f5122508ad43d75c97bdf7d98af8e51f96db8d1f",
          "message": "fix: match Python cqlsh error output format with source prefix and error codes\n\nAdd 'Error from server: code=NNNN [category] message=\"...\"' wrapper to\nserver error messages, matching the Python cassandra-driver's error format.\n\nAdd '<source>:N:' line number prefix to errors in batch/file mode (stdin\npipe, -f flag, SOURCE command), matching Python cqlsh's show_line_nums\nbehavior.\n\nCloses #109",
          "timestamp": "2026-04-20T22:35:40+03:00",
          "tree_id": "5c4f5a7a7fc8b9942185f5ba14486e56a899bc0d",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/f5122508ad43d75c97bdf7d98af8e51f96db8d1f"
        },
        "date": 1776714255212,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29954,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 4,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3487,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9166,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63643,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 842,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 155710,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6961,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 11,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 86314,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 787020,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7888100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9866,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46608,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40963,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "e6cbe189deeb1e52498744855fdef7d79d7264a3",
          "message": "chore: release v0.3.2",
          "timestamp": "2026-04-20T22:46:45+03:00",
          "tree_id": "18adda4d6c10fb527b8420a75550035a23f697aa",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/e6cbe189deeb1e52498744855fdef7d79d7264a3"
        },
        "date": 1776714931425,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 30060,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3477,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9089,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 63348,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 853,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153640,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6958,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 12,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85789,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 790840,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7923100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9925,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 49642,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40573,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "7fb4eff5dee82bfde54429071a16dd83c30fb822",
          "message": "feat: warn on schema version mismatch after connect\n\nQuery system.local and system.peers for schema_version after connecting.\nIf nodes disagree, emit a warning to stderr matching Python cqlsh behavior.\n\nCloses #110",
          "timestamp": "2026-04-20T22:52:12+03:00",
          "tree_id": "e5b4fc76053a51c71cdb520edeed6bd9a995ace7",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/7fb4eff5dee82bfde54429071a16dd83c30fb822"
        },
        "date": 1776715278804,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29976,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3730,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9485,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 65340,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 826,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 147140,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 7280,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 14,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 85710,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 766040,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 7715100,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 10076,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 49665,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 39195,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "fruch@scylladb.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "committer": {
            "email": "israel.fruchter@gmail.com",
            "name": "Israel Fruchter",
            "username": "fruch"
          },
          "distinct": true,
          "id": "90e671cd58e3fcee6840be467386f896f0d60eea",
          "message": "fix: handle LOGIN command in non-interactive mode\n\nPreviously LOGIN was silently ignored in non-interactive/batch mode,\ncausing dtests that send LOGIN via stdin to fail with no error output.\n\nNow LOGIN reconnects with new credentials (matching interactive behavior)\nand outputs auth errors to stderr using the standard error formatter.\n\nCloses #108",
          "timestamp": "2026-04-20T22:52:48+03:00",
          "tree_id": "b1b677ed3c46340c83c38da8a0e3696e405cdbcd",
          "url": "https://github.com/scylladb/cqlsh-rs/commit/90e671cd58e3fcee6840be467386f896f0d60eea"
        },
        "date": 1776715294087,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cli_parse_args/no_args",
            "value": 29725,
            "unit": "ns"
          },
          {
            "name": "cli_validate/valid_full",
            "value": 3,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/empty",
            "value": 3546,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/minimal",
            "value": 9074,
            "unit": "ns"
          },
          {
            "name": "cqlshrc_parse/full",
            "value": 64025,
            "unit": "ns"
          },
          {
            "name": "config_merge/full_merge",
            "value": 847,
            "unit": "ns"
          },
          {
            "name": "end_to_end_startup/full",
            "value": 153990,
            "unit": "ns"
          },
          {
            "name": "parse_multiline/6_lines",
            "value": 6871,
            "unit": "ns"
          },
          {
            "name": "classify_input/empty",
            "value": 12,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/10",
            "value": 87417,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/100",
            "value": 803050,
            "unit": "ns"
          },
          {
            "name": "format_table/rows/1000",
            "value": 8000700,
            "unit": "ns"
          },
          {
            "name": "format_expanded/rows/10",
            "value": 9697,
            "unit": "ns"
          },
          {
            "name": "format_json_100",
            "value": 46755,
            "unit": "ns"
          },
          {
            "name": "format_csv_100",
            "value": 40346,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}