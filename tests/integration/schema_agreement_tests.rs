//! Integration tests for schema version agreement check (PR #113).

use predicates::prelude::*;

use super::helpers::*;

#[test]
#[ignore = "requires Docker"]
fn single_node_has_schema_agreement() {
    let scylla = get_scylla();
    // A single-node cluster should always have schema agreement.
    // The warning should NOT appear in stderr for a healthy single node.
    cqlsh_cmd(scylla)
        .args(["-e", "SELECT key FROM system.local"])
        .assert()
        .success()
        .stderr(predicate::str::contains("schema version mismatch").not());
}
