//! Ordering and delta-check tests for the depth record reader (child module of `depth_record.rs`).

use super::*;

/// B.7's serialization of a value for the purpose of ordering, written here and not taken from the
/// crate: members by name, no whitespace, strings as `serde_json` writes them.
fn canon(v: &Value) -> String {
    match v {
        Value::Object(members) => {
            let mut names: Vec<_> = members.keys().collect();
            names.sort();
            let members: Vec<_> = names
                .iter()
                .map(|name| {
                    format!(
                        "{}:{}",
                        Value::String((*name).clone()),
                        canon(&members[*name])
                    )
                })
                .collect();
            format!("{{{}}}", members.join(","))
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(canon).collect::<Vec<_>>().join(",")
        ),
        other => other.to_string(),
    }
}

/// B.6 rule 1: every list outside `enrollment` and `acceptance_entries` is a set, written without
/// repeats in ascending order of its entries' serialization, nested lists included. A row names a
/// list and, for a list that holds one entry, a leaf of that entry to change into a second entry.
#[test]
fn every_set_list_is_written_in_ascending_order_without_repeats() {
    let lists: [(&str, &str, Value); 19] = [
        ("/policy/base", "", json!(null)),
        ("/policy/templates", "", json!(null)),
        ("/policy/templates/0/obligations", "", json!("B1")),
        ("/policy/templates/1/obligations", "", json!(null)),
        ("/policy/debt_baseline", "/row_id", json!("row-9")),
        ("/obligations/targets", "", json!(null)),
        ("/obligations/acquired_inputs", "/path", json!("x/y.json")),
        ("/obligations/operations", "/id", json!("run2")),
        (
            "/obligations/operations/0/surfaces",
            "/surface_id",
            json!("--other"),
        ),
        ("/obligations/operations/0/obligations", "", json!(null)),
        (
            "/obligations/classification_required",
            "/operation",
            json!("run2"),
        ),
        (
            "/obligations/acceptance_work",
            "/surface/surface_id",
            json!("--other"),
        ),
        (
            "/obligations/acceptance_work/0/targets",
            "",
            json!("darwin-arm64"),
        ),
        ("/execution/results", "", json!(null)),
        (
            "/execution/results/2/evidence",
            "/path",
            json!("a/other.rs"),
        ),
        (
            "/execution/reuse_bindings",
            "/obligation/target",
            json!("darwin-arm64"),
        ),
        ("/execution/debt_grants", "/row_id", json!("row-9")),
        ("/execution/debt_retired", "", json!("row-3")),
        ("/acceptance_entries/0/targets", "", json!("darwin-arm64")),
    ];
    let base: Value = serde_json::from_str(&sealed(record())).unwrap();
    for (list, leaf, second) in lists {
        let mut entries = base.pointer(list).unwrap().as_array().unwrap().clone();
        if !second.is_null() {
            let mut other = entries[0].clone();
            *other.pointer_mut(leaf).unwrap() = second;
            entries.push(other);
        }
        entries.sort_by_key(canon);
        let with = |entries: Vec<Value>| {
            let mut doc = base.clone();
            *doc.pointer_mut(list).unwrap() = Value::Array(entries);
            parse(&sealed(doc))
        };
        with(entries.clone()).unwrap_or_else(|e| panic!("{list} in ascending order: {e}"));

        let reversed = entries.iter().rev().cloned().collect();
        let err = with(reversed).expect_err(list);
        assert!(err.contains("ascending order"), "{list} reversed: {err}");

        let mut repeated = entries.clone();
        repeated.push(entries.last().unwrap().clone());
        let err = with(repeated).expect_err(list);
        assert!(
            ["ascending order", "must not repeat", "policy.templates"]
                .iter()
                .any(|m| err.contains(m)),
            "{list} repeated: {err}"
        );
    }
}

#[test]
fn a_predicate_that_changes_is_another_mode() {
    let earlier = parse(&sealed(record())).unwrap();
    assert_eq!(
        rec_with(|e| e["modes"][0]["predicates"][0]["value"] = json!("yaml"))
            .lacking_from(&earlier),
        ["mode `run-json`"]
    );
    assert_eq!(
        rec_with(|e| e["modes"][0]["predicates"][0] = json!({ "flag": "-q" }))
            .lacking_from(&earlier),
        ["mode `run-json`"]
    );
}

#[test]
fn one_foreign_baseline_row_among_the_agents_rows_is_refused() {
    let err = sealed_with(|r| {
        let mut other = r["policy"]["debt_baseline"][0].clone();
        other["row_id"] = json!("row-3");
        other["agent_id"] = json!("codex");
        r["policy"]["debt_baseline"]
            .as_array_mut()
            .unwrap()
            .push(other);
    })
    .unwrap_err();
    assert!(err.contains("rows of agent `aider` only"), "{err}");
}

#[test]
fn a_retired_row_among_several_grants_is_refused() {
    let err = sealed_with(|r| {
        let mut other = r["execution"]["debt_grants"][0].clone();
        other["row_id"] = json!("row-3");
        r["execution"]["debt_grants"]
            .as_array_mut()
            .unwrap()
            .push(other);
        r["execution"]["debt_retired"] = json!(["row-3"]);
    })
    .unwrap_err();
    assert!(err.contains("granted or retired, never both"), "{err}");
}

#[test]
fn a_debt_row_that_is_not_the_records_is_refused_wherever_it_stands() {
    // Another agent, the same agent in another case, a command path rooted elsewhere or not
    // rooted at all, in either list, before or after the record's own row.
    type Edit = fn(&mut Value);
    let rows: [(&str, Edit); 5] = [
        ("agent", |row| row["agent_id"] = json!("codex")),
        ("case", |row| row["agent_id"] = json!("Aider")),
        ("path", |row| row["command_path"] = json!("aiderx run")),
        ("blank root", |row| {
            row["command_path"] = json!(" aider run")
        }),
        ("tab", |row| row["command_path"] = json!("aider\trun")),
    ];
    for (part, list) in [("policy", "debt_baseline"), ("execution", "debt_grants")] {
        for (what, edit) in rows {
            for at in [0, 1] {
                let err = sealed_with(|r| {
                    let mut other = r[part][list][0].clone();
                    other["row_id"] = json!("row-0");
                    edit(&mut other);
                    r[part][list].as_array_mut().unwrap().insert(at, other);
                })
                .unwrap_err();
                let expected = format!("{part}.{list} must hold rows of agent `aider` only");
                assert!(err.contains(&expected), "{list}, {what}, at {at}: {err}");
            }
        }
    }
    // A grant of another version, before or after the record's own.
    for at in [0, 1] {
        let err = sealed_with(|r| {
            let mut other = r["execution"]["debt_grants"][0].clone();
            other["row_id"] = json!("row-0");
            other["authorized_at_version"] = json!("1.2.2");
            r["execution"]["debt_grants"]
                .as_array_mut()
                .unwrap()
                .insert(at, other);
        })
        .unwrap_err();
        let expected = "execution.debt_grants must authorize version `1.2.3` only";
        assert!(err.contains(expected), "at {at}: {err}");
    }
}
