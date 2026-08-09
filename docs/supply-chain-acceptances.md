# The supply chain self audit, and what is accepted

The repository scores itself against the OpenSSF Scorecard checks and publishes
what it finds. This page is the other half of that: every check the audit
reported, and for each one either the change that answers it or the reason it is
accepted as it stands.

The score is not a target. Several of these checks reward a practice that is
wrong for this repository, and where that is the case the acceptance says so
instead of the practice being adopted to move a number. A score quoted without
its triage is a number that means whatever the reader wants it to mean, and an
untouched finding is the same as no audit at all.

## Where the audit runs and what it publishes

`.github/workflows/scorecard.yml` is the authority for the triggers, and it is
read from the reference a reader will have rather than from a working tree:

    $ git fetch origin && git rev-parse origin/main
    21e1c80e29426280e38ac6f57291d9564654ad65
    $ git show origin/main:.github/workflows/scorecard.yml | sed -n '/^on:/,/^permissions:/p'
    on:
      # Branch-Protection check only reads the default branch's ruleset.
      branch_protection_rule:
      # Keeps the Maintained check current and re-scores against upstream drift.
      schedule:
        - cron: "27 4 * * 1"
      push:
        branches: [main]

    # Read-only by default; the analysis job adds only the scopes it needs. A
    # read-only top-level declaration is itself one of the Token-Permissions checks
    # Scorecard rewards. Declared as an explicit scope rather than read-all so it is
    # minimal and specific (the job overrides this block with its own scopes).
    permissions:

The job is guarded on the default branch, because the Branch-Protection check
reads the rule on that branch and because publication only works from there.

Findings reach the code scanning tab, so they sit where a person will meet them
rather than only in a job log:

    $ gh api repos/iderex/drehbank/code-scanning/alerts --paginate \
        --jq '.[] | select(.tool.name == "Scorecard") | "\(.rule.id) \(.state)"' | sort
    BranchProtectionID open
    CIIBestPracticesID open
    CodeReviewID open
    DependencyUpdateToolID open
    FuzzingID open
    LicenseID open
    MaintainedID open
    SASTID fixed
    SecurityPolicyID open

That view carries the failing checks only. The full result, which is what a
triage has to work from, is the one the run publishes:

    $ curl -sS https://api.securityscorecards.dev/projects/github.com/iderex/drehbank \
        | python -c "import json,sys; d=json.load(sys.stdin); print(d['date'], d['repo']['commit'], d['scorecard']['version'], d['score']); [print('%-24s %3d  %s' % (c['name'], c['score'], c['reason'])) for c in sorted(d['checks'], key=lambda c: c['name'])]"
    2026-08-09T11:37:42Z 21e1c80e29426280e38ac6f57291d9564654ad65 v5.5.0 5.3
    Binary-Artifacts          10  no binaries found in the repo
    Branch-Protection          3  branch protection is not maximal on development and all release branches
    CI-Tests                  10  15 out of 15 merged PRs checked by a CI test -- score normalized to 10
    CII-Best-Practices         0  no effort to earn an OpenSSF best practices badge detected
    Code-Review                0  Found 0/15 approved changesets -- score normalized to 0
    Contributors               0  project has 0 contributing companies or organizations -- score normalized to 0
    Dangerous-Workflow        10  no dangerous workflow patterns detected
    Dependency-Update-Tool     0  no update tool detected
    Fuzzing                    0  project is not fuzzed
    License                    0  license file not detected
    Maintained                 0  project was created within the last 90 days. Please review its contents carefully
    Packaging                 -1  packaging workflow not detected
    Pinned-Dependencies       10  all dependencies are pinned
    SAST                      10  SAST tool is run on all commits
    Security-Policy            4  security policy file detected
    Signed-Releases           -1  no releases found
    Token-Permissions         10  GitHub workflow tokens follow principle of least privilege
    Vulnerabilities           10  0 existing vulnerabilities detected

## The run this triage was made against

Commit `21e1c80e29426280e38ac6f57291d9564654ad65`, scanned at 2026-08-09T11:37:42Z
by Scorecard v5.5.0, aggregate 5.3 out of 10. Eighteen checks were reported.

The run is on the tracker as well as in the output above:

    $ gh run list --workflow=scorecard.yml --limit 1 \
        --json databaseId,headSha,event,conclusion,createdAt \
        --jq '.[] | "\(.databaseId) \(.event) \(.conclusion) \(.createdAt) \(.headSha)"'
    31311253678 push success 2026-08-09T11:37:32Z 21e1c80e29426280e38ac6f57291d9564654ad65

A later run moves these numbers. Re-run the command above rather than reading
the figures here as current, and where a number has moved, the entry it belongs
to is re-triaged in the change that notices it.

## What passes, and what that is worth

Seven checks scored ten: Binary-Artifacts, CI-Tests, Dangerous-Workflow,
Pinned-Dependencies, SAST, Token-Permissions and Vulnerabilities. Each of them
is a property this repository holds on purpose and each has a home in the tree
rather than being an accident of the score. Actions are pinned to a commit and
the dependency graph is one dev-dependency, which is what
`docs/dependencies.md` argues. Workflow permissions are declared read-only at
the top and widened only on the job that needs them, which is the rule
`.github/workflows/zizmor.yml` audits every workflow against.

Nothing is accepted for these and nothing is owed. They are listed so that a
reader can see the triage covers the whole run and not only its complaints.

## What the audit could not evaluate

Packaging and Signed-Releases both scored minus one, which is Scorecard's value
for a check it could not run rather than a failure. There is no publishing
workflow and there are no releases, so neither check had anything to read.

Both become real checks the day the first release exists, and both are inside
what #63 builds. They are not accepted here, because there is nothing yet to
accept; they are re-triaged when the release artefacts land.

## The accepted findings

Nine checks reported a finding and none of them is fixed by this change. Each
entry below says why, and each names what would end the acceptance, so that an
entry cannot quietly become permanent by nobody looking at it again.

### Branch-Protection, 3

Accepted. The rule on the default branch refuses deletion, refuses a non fast
forward push, requires a pull request and has no bypass actors, so it holds for
everyone including the maintainer:

    $ gh api repos/iderex/drehbank/rulesets --jq '.[] | select(.target=="branch") | .id'
    20521226
    $ gh api repos/iderex/drehbank/rulesets/20521226 \
        --jq '{enforcement, bypass: .bypass_actors, rules: [.rules[].type]}'
    {"bypass":[],"enforcement":"active","rules":["deletion","non_fast_forward","pull_request"]}

What the check is missing is a required status check, a required approver,
codeowners review and stale review dismissal, and the parameters say the same:

    $ gh api repos/iderex/drehbank/rulesets/20521226 \
        --jq '.rules[] | select(.type=="pull_request") | .parameters'
    {"allowed_merge_methods":["merge","squash","rebase"],"dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,"require_last_push_approval":false,"required_approving_review_count":0,"required_review_thread_resolution":false,"required_reviewers":[]}


The required status checks are the missing half that this project agrees with,
and #27 holds the list and the order to attach it in. It is deliberately not
attached yet: a check is added to the rule only once it has been observed red at
least once, because a required check that has never failed can be required and
broken at the same time. The approver and codeowners halves depend on whether
outside contributions are accepted, which is an open entry in #2.

Ends when #27 attaches its list and the review posture in #2 is answered.

### Code-Review, 0

Accepted. The check found no approved changeset in the last fifteen, which is
accurate: there is no second reviewer on this repository today, so an approval
would be the author approving their own change.

What stands in place of a review here is the evidence a change carries in its
own body, which is the rule the contribution guide states. That is weaker than a
second reader and this entry does not pretend otherwise.

Ends when the review posture in #2 is answered and a second reader exists.

### License, 0

Accepted as standing rather than as correct. There is no LICENSE file, so today
nobody may lawfully use, copy or modify anything in this repository, and that is
the single worst finding in the run. It is not fixed here because which license
applies is the first open entry in #2 and is not a choice any change can take on
its own.

Ends when #2 answers the license entry and #21 puts the file in.

### Fuzzing, 0

Accepted. Nothing in the tree is fuzzed yet. The parsing surface this check is
about is the file reader, which the security policy already names as the place
input somebody else produced enters the package, and #56 is the issue that
fuzzes it together with the constructors.

Ends when #56 lands.

### Dependency-Update-Tool, 0

Accepted. No bot opens dependency updates here. A dependency in this workspace
arrives with a written argument on `docs/dependencies.md` covering what it is
for, what removing it would cost, its license read from the crate's own metadata
and whether it reaches the numbers, and a bot cannot write any of those four.

The half of this check that is worth having is the pinned actions, which move
without anybody in this project touching a line, and today those are raised by
hand in their own change. Whether that becomes automatic is a question about how
a change may arrive rather than about the graph, and it sits with the
contribution route entry in #2.

Ends when the contribution route in #2 is answered, at which point this is
re-triaged rather than assumed.

### Security-Policy, 4

Accepted, and this is one of the checks that rewards the wrong thing here. The
policy exists, states a disclosure route and says what a reporter can expect.
The points it is missing are for linked content, which Scorecard counts as an
absolute URL or an email address in the file.

The route this project wants a reporter to take is the platform's private
reporting form, which is neither of those, and a public address exists nowhere
in the tree on purpose. Adding a URL so that a string matcher finds one would
move the score and would not give a reporter anything they did not already have.

Ends if the reporting route ever changes to one that has an address, in which
case the address goes in the policy because a reporter needs it, not because
this check counts it.

### CII-Best-Practices, 0

Accepted. The badge is a self-assessment filed on a separate site, so earning it
is an account action outside this tree and it certifies claims this repository
already makes in documents a reader can check.

Ends if the badge is ever wanted for a reason other than the score, which would
be a decision taken in the open like any other.

### Contributors, 0

Accepted, and it is not a finding this project can act on. The check counts
contributing companies or organizations, and there is one author and no
organization.

Ends by itself if the project ever has contributors from more than one
organization, which is downstream of the contribution route in #2 rather than of
anything a change can do.

### Maintained, 0

Accepted, and it retires itself. The check reports that the repository was
created within the last ninety days, which is true and is a statement about the
calendar rather than about the work.

Ends ninety days after creation, with no change owed.

## What this page is not

It is not a claim that the repository is secure. It is a record that every check
one automated audit reported was read and answered, on one commit, on one date,
by one reading. The audit reads what it can see from outside, and the failures
this package most fears, a wrong number that looks like a right one, are
invisible to it.
