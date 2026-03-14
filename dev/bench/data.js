window.BENCHMARK_DATA = {
  "lastUpdate": 1773532671420,
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
      }
    ]
  }
}