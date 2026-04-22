#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FindingSignature {
    pub category_id: String,
    pub surfaces: Vec<String>,
}

impl FindingSignature {
    pub fn new(category_id: impl Into<String>, surfaces: &[String]) -> Self {
        let mut normalized_surfaces = surfaces.to_vec();
        normalized_surfaces.sort();
        normalized_surfaces.dedup();
        Self {
            category_id: category_id.into(),
            surfaces: normalized_surfaces,
        }
    }
}
