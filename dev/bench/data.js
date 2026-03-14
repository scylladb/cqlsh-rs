window.BENCHMARK_DATA = {
  "lastUpdate": 1773531538738,
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
      }
    ]
  }
}