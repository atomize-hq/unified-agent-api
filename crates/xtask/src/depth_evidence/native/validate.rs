use super::*;
use crate::depth_evidence::validate as v;
fn revision(n: u64) -> Result<(), String> {
    if n != 1 {
        Err("unsupported native payload revision".into())
    } else {
        Ok(())
    }
}
pub(super) fn repository(r: &Repository) -> Result<(), String> {
    v::repository(&r.name)?;
    v::positive(r.id)?;
    v::positive(r.owner_id)
}
pub(super) fn comment(r: &CommentRef) -> Result<(), String> {
    v::positive(r.id)?;
    v::sha256("native host body", &r.body_sha256)
}
pub(super) fn receipt(r: &NativeReceipt) -> Result<(), String> {
    if r.schema_version != 2 {
        return Err("unsupported native receipt revision".into());
    }
    repository(&r.producer.repository)?;
    v::positive(r.producer.pull_request)?;
    comment(&r.producer.registration)?;
    comment(&r.producer.completion)?;
    if r.producer.registration.id >= r.producer.completion.id {
        return Err("native completion precedes registration".into());
    }
    v::common(r.claims())
}
pub(super) fn start(r: &Start) -> Result<(), String> {
    revision(r.schema_version)?;
    v::text(&r.profile)?;
    repository(&r.repository)?;
    v::positive(r.pull_request)?;
    v::git_identity(&r.packet_head)?;
    v::scope(&r.scope)?;
    v::file(&r.plan)
}
pub(super) fn finish(r: &Finish) -> Result<(), String> {
    revision(r.schema_version)?;
    comment(&r.registration)?;
    v::git_identity(&r.evidence_commit)?;
    v::file(&r.bundle)?;
    v::text(&r.executor_identity)?;
    v::positive(r.population_through_id)?;
    v::unique(&r.registrations, |i| i)?;
    for id in &r.registrations {
        v::positive(*id)?;
        if *id > r.population_through_id {
            return Err("registration beyond completion snapshot".into());
        }
    }
    if !r.registrations.contains(&r.registration.id) {
        return Err("completion omits its registration".into());
    }
    Ok(())
}
pub(super) fn plan(r: &Plan) -> Result<(), String> {
    revision(r.schema_version)?;
    v::scope(&r.scope)?;
    v::git_identity(&r.packet_head)?;
    v::build(&r.build)?;
    let version =
        semver::Version::parse(&r.native.version).map_err(|_| "invalid native version")?;
    if !version.pre.is_empty()
        || !version.build.is_empty()
        || version.to_string() != r.native.version
    {
        return Err("noncanonical native version".into());
    }
    v::text(&r.native.target)?;
    v::sha256("native executable", &r.native.executable_sha256)?;
    v::file(&r.native.acquisition)?;
    v::file(&r.context)?;
    v::file(&r.collector)?;
    v::unique(&r.requested, |c| c)?;
    for c in &r.requested {
        v::case_identity(c)?;
    }
    Ok(())
}
pub(super) fn bundle(r: &Bundle) -> Result<(), String> {
    revision(r.schema_version)?;
    v::file(&r.plan)?;
    v::files(&r.claims)?;
    v::files(&r.files)?;
    if r.claims.is_empty()
        || !r.files.contains(&r.plan)
        || r.claims.iter().any(|f| !r.files.contains(f))
    {
        return Err("bundle omits plan or claims".into());
    }
    Ok(())
}
pub(super) fn claim(r: &Claim) -> Result<(), String> {
    v::scope(&r.scope)?;
    v::build(&r.build)?;
    v::provenance(&r.provenance)?;
    v::cases(&r.cases)?;
    v::mapping(&r.mapping)
}
