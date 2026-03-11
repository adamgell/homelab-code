# CMTrace Open next implementation plan

This document tracks the next implementation slices after the first-pass Intune workflow and baseline source plumbing. The direction is now sample-driven evidence-bundle intake, evidence inventory, parser hardening from real samples, registry state support, and curated adjacent evidence.

## Current position

The following work should be treated as good enough for now:

- First-class log sources: file, folder, and known platform sources.
- Folder browsing in the left sidebar for log viewing.
- Toolbar support for Open Folder.
- Native app menu entries for known Windows log sources.
- Intune analysis support for either a file path or a directory path.
- First-pass IME sidecar analysis, source provenance, and diagnostics summaries.

The main gap is no longer basic Intune folder analysis. The bigger product gap is taking mixed real-world investigation evidence and turning it into a clear, reviewable inventory of what the app recognized, what it could parse, and where evidence is still missing.

## Recommended next slice

The next slice should establish evidence intake and coverage before more specialized parser work.

1. Add sample intake for mixed investigation evidence.
2. Ship an evidence inventory with provenance and parse status.
3. Harden existing parsers and IME rules from real samples.
4. Add registry snapshot support as structured device state.
5. Add curated event-log intake for adjacent evidence.

## Phase 1: Sample intake baseline

Goal: make a local investigation folder usable even when it contains a mix of logs, exports, and partially supported artifacts.

- Accept a mixed evidence bundle instead of assuming one log family.
- Classify inputs into logs, registry snapshots, curated event exports, and unknown artifacts.
- Produce a deterministic intake summary with recognized sources, parse failures, and obvious missing evidence.
- Preserve stable ordering so comparisons across sample revisions stay easy.

Expected outcome:

- A user can point the app at a local evidence bundle and immediately understand what evidence is available and what is not yet supported.

## Phase 2: Evidence inventory and provenance

Goal: make every investigation start with coverage and source traceability.

- Show included artifacts, ignored artifacts, parse status, and basic time coverage.
- Surface provenance throughout the UI so timelines and summaries identify their source artifact.
- Make partial evidence obvious so conclusions are easier to trust and review.

Expected outcome:

- The app can answer which artifact produced each notable event and whether the supplied evidence is broad enough to support a conclusion.

## Phase 3: Parser hardening from real samples

Goal: improve reliability of the parsers and evidence rules already in the product before expanding into more speculative formats.

- Use real samples to tighten detection, multiline handling, timestamp parsing, and severity mapping.
- Track unsupported patterns and unknown source families discovered during intake.
- Keep Intune rule work in maintenance mode: improve it when samples expose repeatable gaps, not as a standalone feature track.

Expected outcome:

- Fewer generic events, fewer parse misses, and better evidence quality on real customer-style samples.

## Phase 4: Registry snapshot support

Goal: ingest registry evidence as structured device state that can be queried and correlated with logs.

- Start with practical snapshot inputs such as exported `.reg` files.
- Normalize hives, keys, and values so enrollment, policy, and app-management views can reference them directly.
- Focus on high-value device state such as MDM enrollment, PolicyManager data, and app-management state.

Expected outcome:

- Registry evidence becomes first-class correlation data rather than an unstructured attachment.

## Phase 5: Curated event-log intake

Goal: add adjacent evidence sources without blocking on a full generic event viewer.

- Start with curated channels relevant to MDM, Autopilot, enrollment, BitLocker, LAPS, and Defender.
- Prefer practical intake paths such as saved exports and offline investigation artifacts first.
- Correlate event IDs, levels, and timestamps into the shared evidence inventory and timelines.

Expected outcome:

- Local investigation bundles can include focused event evidence that complements logs and registry state.

## Follow-on parser expansion

After intake, inventory, and hardening are in place, the next parser work should stay sample-led. Current likely priorities remain:

1. CBS/Panther format family.
2. WindowsUpdate.log high-precision timestamp support.
3. SetupAPI section-based logs.

These formats are still high-value, but they should land in response to real investigation demand rather than as the primary product track.

## Validation

When the next slices are implemented, validate with:

1. Frontend build.
2. Rust test suite.
3. Manual intake against a real mixed local evidence bundle containing logs plus at least one adjacent artifact such as a registry export or curated event-log export.
