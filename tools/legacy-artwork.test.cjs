const {test} = require('node:test');
const assert = require('node:assert/strict');
const {selectCaches, selectBundles, run} = require('./legacy-artwork.cjs');

test('only the retired Studio cache namespace is selected', () => {
  assert.deepEqual(selectCaches([{key:'studio-art-v1-abc'},{key:'samba-build'},{key:'studio-art-v2-def'},{key:'studio-art-other'}]).map(c=>c.key), ['studio-art-v1-abc','studio-art-v2-def']);
});
test('only completed Pages upload bundles are selected', () => {
  const runs = new Map([[1,{status:'completed',path:'.github/workflows/pages.yml@main'}],[2,{status:'in_progress',path:'.github/workflows/pages.yml'}],[3,{status:'completed',path:'.github/workflows/release.yml'}]]);
  const artifacts = [{id:1,name:'github-pages',workflow_run:{id:1}},{id:2,name:'github-pages',workflow_run:{id:2}},{id:3,name:'github-pages',workflow_run:{id:3}},{id:4,name:'release-downloads',workflow_run:{id:1}},{id:5,name:'github-pages',workflow_run:{id:99}}];
  assert.deepEqual(selectBundles(artifacts,runs).map(a=>a.id), [1]);
});
test('default audit makes no deletion calls', async () => {
  const calls = [];
  const actions = {
    getActionsCacheList:()=>{},listArtifactsForRepo:()=>{},
    deleteActionsCacheById:args=>calls.push(args),deleteArtifact:args=>calls.push(args),
    getWorkflowRun:async()=>({data:{status:'completed',path:'.github/workflows/pages.yml'}}),
  };
  await run({github:{rest:{actions},paginate:async method=>method===actions.getActionsCacheList?[{id:1,key:'studio-art-v1-old'}]:[{id:2,name:'github-pages',workflow_run:{id:3}}]},context:{repo:{owner:'jgraving',repo:'slot.sp'}},core:{info:()=>{},summary:{addRaw(){return this;},async write(){}}}});
  assert.deepEqual(calls, []);
});
test('cleanup deletes only selected storage and protects a Pages run that restarted', async () => {
  const cleared = [], reads = new Map();
  const actions = {
    getActionsCacheList:()=>{}, listArtifactsForRepo:()=>{},
    deleteActionsCacheById:async ({cache_id})=>cleared.push(['cache',cache_id]),
    deleteArtifact:async ({artifact_id})=>cleared.push(['artifact',artifact_id]),
    getWorkflowRun:async ({run_id})=>{
      const count = (reads.get(run_id) || 0)+1; reads.set(run_id,count);
      return {data:{status:run_id===3 && count>1?'in_progress':'completed',path:'.github/workflows/pages.yml'}};
    },
  };
  const github = {rest:{actions},paginate:async method=>method===actions.getActionsCacheList
    ? [{id:10,key:'studio-art-v1-old'},{id:11,key:'samba-build'}]
    : [{id:20,name:'github-pages',workflow_run:{id:3}},{id:21,name:'release-downloads',workflow_run:{id:4}},{id:22,name:'github-pages',workflow_run:{id:5}}]};
  await run({github,context:{repo:{owner:'jgraving',repo:'slot.sp'}},dryRun:false,
    core:{info:()=>{},summary:{addRaw(){return this;},async write(){}}}});
  assert.deepEqual(cleared,[['cache',10],['artifact',22]]);
});
test('another repository is rejected before making API calls', async () => {
  await assert.rejects(run({context:{repo:{owner:'someone',repo:'other'}}}), /Unexpected repository/);
});
