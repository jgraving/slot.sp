// Remove only retired Studio cache keys and completed Pages upload bundles.
// Release downloads, source files, ordinary review artifacts and active runs stay intact.
function selectCaches(caches) {
  return caches.filter(cache => /^studio-art-v\d+-/.test(cache.key));
}
function selectBundles(artifacts, runs) {
  return artifacts.filter(artifact => {
    const run = runs.get(artifact.workflow_run?.id);
    return artifact.name === 'github-pages' && run?.status === 'completed'
      && run.path?.split('@')[0] === '.github/workflows/pages.yml';
  });
}
async function run({github, context, core, dryRun = true}) {
  const {owner, repo} = context.repo;
  if (owner !== 'jgraving' || repo !== 'slot.sp') throw new Error('Unexpected repository');
  const caches = await github.paginate(github.rest.actions.getActionsCacheList, {owner, repo, per_page:100});
  const artifacts = await github.paginate(github.rest.actions.listArtifactsForRepo, {owner, repo, per_page:100});
  const runs = new Map();
  for (const id of new Set(artifacts.filter(a => a.name === 'github-pages').map(a => a.workflow_run?.id).filter(Boolean))) {
    const {data} = await github.rest.actions.getWorkflowRun({owner, repo, run_id:id});
    runs.set(id, data);
  }
  const selected = selectCaches(caches);
  let bundles = 0;
  for (const cache of selected) {
    core.info(`${dryRun ? 'Would clear' : 'Clear'} legacy cache ${cache.id}`);
    if (!dryRun) await github.rest.actions.deleteActionsCacheById({owner, repo, cache_id:cache.id});
  }
  for (const artifact of selectBundles(artifacts, runs)) {
    // A rerun may have started since the initial listing.
    const {data} = await github.rest.actions.getWorkflowRun({owner, repo, run_id:artifact.workflow_run.id});
    if (data.status !== 'completed') continue;
    core.info(`${dryRun ? 'Would clear' : 'Clear'} completed Pages upload ${artifact.id}`);
    if (!dryRun) await github.rest.actions.deleteArtifact({owner, repo, artifact_id:artifact.id});
    bundles++;
  }
  await core.summary.addRaw(`${dryRun ? 'Audit' : 'Cleanup'}: ${selected.length} legacy artwork caches and ${bundles} completed Pages uploads. This does not change source branches, releases, the deployed site or handheld files.\n`).write();
}
module.exports = {selectCaches, selectBundles, run};
