window.BENCHMARK_DATA = {
  "lastUpdate": 1774507246178,
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
      }
    ]
  }
}