window.BENCHMARK_DATA = {
  "lastUpdate": 1773610425238,
  "repoUrl": "https://github.com/fruch/cqlsh-rs",
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
      }
    ]
  }
}