use super::*;
use serde_json::json;
struct Pages(BTreeMap<String, Value>);
impl Api for Pages {
    fn json(&self, _: &form::Repository, path: &str) -> Result<Value, String> {
        self.0.get(path).cloned().ok_or("missing page".into())
    }
}
#[test]
fn full_pages_are_not_successful_truncation_and_duplicate_ids_refuse() {
    let repo = form::Repository {
        name: "owner/proof".into(),
        id: 1,
        owner_id: 2,
    };
    let mut pages = Pages(BTreeMap::new());
    for page in 1..=10 {
        pages.0.insert(
            format!("issues/8/comments?per_page=100&page={page}"),
            json!((1..=100)
                .map(|id| json!({"id":(page-1)*100+id}))
                .collect::<Vec<_>>()),
        );
    }
    // A saturated last page is not a complete population.
    assert_eq!(
        list(&pages, &repo, "issues/8/comments").unwrap_err(),
        "native host pagination bound reached"
    );
    pages.0.insert(
        "issues/8/comments?per_page=100&page=2".into(),
        json!([{"id":101,"body":"unfavorable unselected attempt"}]),
    );
    let rows = list(&pages, &repo, "issues/8/comments").unwrap();
    assert_eq!(rows.len(), 101);
    assert_eq!(rows[100]["body"], "unfavorable unselected attempt");
    pages
        .0
        .get_mut("issues/8/comments?per_page=100&page=2")
        .unwrap()[0]["id"] = json!(1);
    assert_eq!(
        list(&pages, &repo, "issues/8/comments").unwrap_err(),
        "duplicate native host identity"
    );
    pages.0.remove("issues/8/comments?per_page=100&page=2");
    assert!(list(&pages, &repo, "issues/8/comments").is_err());
}
