import { readFileSync } from 'fs';
import { join } from 'path';

const { repo, owner } = context.repo;

function logJsonData(title, data) {
  core.startGroup(title);
  core.info(JSON.stringify(data, null, 2));
  core.endGroup();
}

let latest_dfx_release = await github.rest.repos.getLatestRelease({ owner, repo });
logJsonData('The latest dfx release', latest_dfx_release);

const re = new RegExp('\n.* [Rr]eplica .* elected commit ([a-f0-9]{40})');
let replica_version_used_in_latest_dfx_release;
try {
  replica_version_used_in_latest_dfx_release = re.exec(latest_dfx_release.data.body)[1];
  core.info(`Replica version used in the latest dfx release: ${replica_version_used_in_latest_dfx_release}`);
} catch {
  replica_version_used_in_latest_dfx_release = "";
  core.warning("The phrase \"replica elected commit: <SHA>\" was not found in the latest GitHub Release. The proposed list of executed proposals will be incorrect.");
}

let elected_replicas = await github.request("GET ${{ env.IC_RELEASES_API }}");
logJsonData('Elected replicas fetched from ic-api.internetcomputer.org', elected_replicas);

let idx_start = elected_replicas.data.data.findIndex(el => el.replica_version_id === "${{ env.REPLICA_VERSION }}");
let idx_end = elected_replicas.data.data.findIndex(el => el.replica_version_id === latest_release_replica_version);
let new_proposals_since_last_release = elected_replicas.data.data.slice(idx_start, idx_end);
logJsonData('New proposals since last release', `{ idx_start: ${idx_start}, idx_end:${idx_end}, selected_proposals:${new_proposals_since_last_release} }`);

const new_replica_sha__short = "${{ env.REPLICA_VERSION }}".substring(0, 8);
const templatePath = join(process.env.GITHUB_WORKSPACE, '.github/PULL_REQUEST_TEMPLATE_FOR_UPDATING_REPLICA.md');
let template = readFileSync(templatePath, 'utf8');
template = template.replace(/{{REPO_OWNER}}/g, owner);
template = template.replace(/{{REPO_NAME}}/g, repo);
template = template.replace(/{{REPLICA_VERSION}}/g, '${{ env.REPLICA_VERSION }}');
template = template.replace(/{{NEW_PROPOSALS}}/g, new_proposals_since_last_release.map(el => `- [${el.proposal_id}](https://dashboard.internetcomputer.org/proposal/${el.proposal_id})`).join('\n'));
template = template.replace(/{{PREVIOUS_RELEASE_REPLICA_VERSION}}/g, replica_version_used_in_latest_dfx_release);

const pr_create_result = await github.rest.pulls.create({
  title: `chore: update replica version to ${new_replica_sha__short}`,
  owner,
  repo,
  head: 'chore-update-replica-${{ env.REPLICA_VERSION }}',
  base: 'master',
  body: template
});
github.rest.issues.addLabels({
  owner,
  repo,
  issue_number: pr_create_result.data.number,
  labels: ['chore', 'automerge-squash', 'replica-update']
});
logJsonData('New PR JSON object', pr_create_result);
