#!/usr/bin/env node
// Requirement registry linter (BA-REQ-LINT-001, BA-REQ-LINT-002, Appendix D).
//
// `requirements-registry.yaml` is the machine-readable source of truth for the
// stable requirement IDs; Appendix D of DESIGN-FINAL-LOCKED.md is generated
// from it. This script enforces, and CI fails on:
//
//   * a registry that does not parse, or whose top-level shape is wrong;
//   * duplicate, malformed or orphan requirement IDs;
//   * status values outside the documented `status_semantics` vocabulary;
//   * a missing field from the Appendix D.1 evidence-matrix field list;
//   * a `source_section` that does not resolve to a real section of the design;
//   * a release-critical requirement with no test ID;
//   * an inline `[BA-...]` ID in the design with no registry entry, or a
//     registry entry the design never states (orphan/unknown IDs);
//   * Appendix D disagreeing with the registry on IDs, order, source section,
//     summary text or release gate (a stale generated table);
//   * a `PASS` status whose recorded commit, time, platform or evidence
//     artifacts are missing, or whose evidence artifacts are not on disk.
//
// It also runs the advisory requirement-density check required by
// BA-REQ-LINT-001: normative paragraphs carrying more than one stable ID are
// reported for human review. Advisory output never fails the run.

import { readFileSync, existsSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import { load } from 'js-yaml';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const registryPath = join(repoRoot, 'requirements-registry.yaml');
const designPath = join(repoRoot, 'DESIGN-FINAL-LOCKED.md');

const ID_PATTERN = /^BA-[A-Z0-9]+(?:-[A-Z0-9]+)*$/;
const REQUIRED_FIELDS = [
  'id',
  'source_section',
  'summary',
  'priority',
  'owner_module',
  'test_ids',
  'evidence_artifacts',
  'status',
  'last_verified_commit',
  'last_verified_at',
  'platform',
  'approved_deviation_id',
  'notes',
];
// Appendix D.2/BA-REQ-LINT-001: these gates must name a test ID.
const RELEASE_CRITICAL_PRIORITIES = new Set(['critical', 'merge-blocking', 'release-blocking']);

const violations = [];
const advisories = [];

const fail = (message) => violations.push(message);
const advise = (message) => advisories.push(message);

const readText = (path) => readFileSync(path, 'utf8');

// ---------------------------------------------------------------- design doc

const design = readText(designPath);

/** Section numbers that exist as headings in the design document. */
const designSections = new Set(
  [...design.matchAll(/^#{2,6}\s+(\d+(?:\.\d+)*)[.\s]/gm)].map((match) => match[1]),
);
/** Appendix letters that exist in the design document. */
const designAppendices = new Set(
  [...design.matchAll(/^#{2,6}\s+Appendix\s+([A-Z])\b/gm)].map((match) => match[1]),
);
/** ADR identifiers the design document references. */
const designAdrs = new Set([...design.matchAll(/\bADR-\d{3}\b/g)].map((match) => match[0]));

const normaliseSource = (value) =>
  String(value)
    .replace(/`/g, '')
    .replace(/[–—]/g, '-')
    .replace(/\s*,\s*/g, ',')
    .trim();

/**
 * Resolves one `source_section` string against the design document, returning
 * the tokens that do not resolve.
 */
function unresolvedSourceTokens(raw) {
  const normalised = normaliseSource(raw);
  if (normalised === '') return ['(empty)'];

  const unresolved = [];
  for (const part of normalised.split(',')) {
    const token = part.trim();
    if (token === '') continue;

    const range = token.split('-');
    if (
      range.length === 2 &&
      /^\d+(?:\.\d+)*$/.test(range[0]) &&
      /^\d+(?:\.\d+)*$/.test(range[1])
    ) {
      for (const endpoint of range) if (!designSections.has(endpoint)) unresolved.push(endpoint);
      continue;
    }

    const appendix = /^Appendix\s+([A-Z])$/i.exec(token);
    if (appendix) {
      if (!designAppendices.has(appendix[1].toUpperCase())) unresolved.push(token);
      continue;
    }

    if (/^ADR-\d{3}$/.test(token)) {
      if (!designAdrs.has(token)) unresolved.push(token);
      continue;
    }

    if (/^\d+(?:\.\d+)*$/.test(token)) {
      if (!designSections.has(token)) unresolved.push(token);
      continue;
    }

    unresolved.push(token);
  }
  return unresolved;
}

// ------------------------------------------------------------ Appendix D rows

/** Parses the Appendix D registry table, preserving row order. */
function parseAppendixD() {
  const start = design.indexOf('## Appendix D —');
  if (start < 0) {
    fail('DESIGN-FINAL-LOCKED.md has no "## Appendix D —" section');
    return [];
  }
  const end = design.indexOf('\n## Appendix E', start);
  const body = design.slice(start, end < 0 ? undefined : end);

  const rows = [];
  for (const line of body.split('\n')) {
    const cells = line.split('|').slice(1, -1);
    if (cells.length !== 4) continue;
    const id = cells[0].trim().replace(/`/g, '');
    if (!ID_PATTERN.test(id)) continue;
    rows.push({
      id,
      source: cells[1].trim(),
      summary: cells[2].trim(),
      releaseGate: cells[3].trim(),
    });
  }
  return rows;
}

// ------------------------------------------------------------- inline IDs

/** Stable inline IDs the design document states, in first-appearance order. */
function inlineIds() {
  const seen = new Set();
  const ordered = [];
  for (const match of design.matchAll(/\[(BA-[A-Z0-9-]+)\]/g)) {
    if (!seen.has(match[1])) {
      seen.add(match[1]);
      ordered.push(match[1]);
    }
  }
  return ordered;
}

/** Normative paragraphs (non-table, non-heading) carrying more than one ID. */
function densityReport() {
  const paragraphs = design.split(/\n\s*\n/);
  const flagged = [];
  for (const paragraph of paragraphs) {
    const trimmed = paragraph.trim();
    if (trimmed === '' || trimmed.startsWith('#') || trimmed.startsWith('|')) continue;
    const ids = [...new Set([...trimmed.matchAll(/\[(BA-[A-Z0-9-]+)\]/g)].map((m) => m[1]))];
    if (ids.length > 1) {
      const heading = trimmed.split('\n')[0].slice(0, 60).replace(/\s+/g, ' ');
      flagged.push(`${ids.join(', ')} — paragraph starting "${heading}…"`);
    }
  }
  return flagged;
}

// --------------------------------------------------------------- registry

let registry;
try {
  registry = load(readText(registryPath), { filename: registryPath });
} catch (error) {
  fail(`requirements-registry.yaml does not parse: ${error.message}`);
}

if (registry) {
  const statusVocabulary = Object.keys(registry.status_semantics ?? {});
  if (statusVocabulary.length === 0) {
    fail('requirements-registry.yaml has no `status_semantics` vocabulary');
  }
  const requirements = registry.requirements;
  if (!Array.isArray(requirements) || requirements.length === 0) {
    fail('requirements-registry.yaml has no `requirements` list');
  } else {
    const byId = new Map();
    for (const [index, entry] of requirements.entries()) {
      const where = `requirements[${index}]`;
      if (entry === null || typeof entry !== 'object') {
        fail(`${where} is not a mapping`);
        continue;
      }

      for (const field of REQUIRED_FIELDS) {
        if (!(field in entry))
          fail(`${where} (${entry.id ?? 'no id'}) is missing field \`${field}\``);
      }

      const id = entry.id;
      if (typeof id !== 'string' || !ID_PATTERN.test(id)) {
        fail(`${where} has malformed id ${JSON.stringify(id)}`);
        continue;
      }
      if (byId.has(id)) fail(`duplicate requirement id ${id}`);
      byId.set(id, entry);

      if (!statusVocabulary.includes(entry.status)) {
        fail(`${id} has status ${JSON.stringify(entry.status)} outside the documented vocabulary`);
      }

      for (const field of ['source_section', 'summary', 'priority', 'owner_module', 'notes']) {
        const value = entry[field];
        if (typeof value !== 'string' || value.trim() === '') {
          fail(`${id} has empty \`${field}\``);
        }
      }

      if (typeof entry.source_section === 'string') {
        const unresolved = unresolvedSourceTokens(entry.source_section);
        if (unresolved.length > 0) {
          fail(
            `${id} cites source section(s) ${unresolved.join(', ')} that do not exist in the design`,
          );
        }
      }

      const testIds = entry.test_ids;
      if (!Array.isArray(testIds)) {
        fail(`${id} has a non-list \`test_ids\``);
      } else {
        if (testIds.some((testId) => typeof testId !== 'string' || testId.trim() === '')) {
          fail(`${id} has an empty test ID`);
        }
        if (RELEASE_CRITICAL_PRIORITIES.has(entry.priority) && testIds.length === 0) {
          fail(`${id} is release-critical (${entry.priority}) but names no test ID`);
        }
      }

      const artifacts = entry.evidence_artifacts;
      if (!Array.isArray(artifacts)) {
        fail(`${id} has a non-list \`evidence_artifacts\``);
      }

      if (entry.status === 'PASS') {
        if (
          typeof entry.last_verified_commit !== 'string' ||
          entry.last_verified_commit.trim() === ''
        ) {
          fail(`${id} is PASS without a \`last_verified_commit\``);
        }
        if (typeof entry.last_verified_at !== 'string' || entry.last_verified_at.trim() === '') {
          fail(`${id} is PASS without a \`last_verified_at\``);
        }
        if (!Array.isArray(entry.platform) || entry.platform.length === 0) {
          fail(`${id} is PASS without a recorded \`platform\``);
        }
        if (!Array.isArray(artifacts) || artifacts.length === 0) {
          fail(`${id} is PASS without evidence artifacts`);
        }
        if (typeof entry.notes !== 'string' || entry.notes.trim() === '') {
          fail(`${id} is PASS without notes describing what was actually run`);
        }
        if (Array.isArray(artifacts)) {
          for (const artifact of artifacts) {
            if (typeof artifact !== 'string') {
              fail(`${id} has a non-string evidence artifact`);
              continue;
            }
            const artifactPath = join(repoRoot, artifact);
            if (!existsSync(artifactPath)) {
              fail(`${id} cites evidence artifact ${artifact} which does not exist`);
            } else if (statSync(artifactPath).isDirectory()) {
              fail(`${id} cites evidence artifact ${artifact} which is a directory, not a file`);
            }
          }
        }
      }

      if (entry.approved_deviation_id !== null && entry.approved_deviation_id !== undefined) {
        if (
          typeof entry.approved_deviation_id !== 'string' ||
          entry.approved_deviation_id.trim() === ''
        ) {
          fail(`${id} has a non-string \`approved_deviation_id\``);
        }
      }
    }

    // ---------------------------------------------------- registry vs design

    const inline = inlineIds();
    for (const id of inline) {
      if (!byId.has(id)) fail(`design states inline ID ${id} that the registry does not contain`);
    }
    for (const id of byId.keys()) {
      if (!inline.includes(id))
        fail(`registry contains ${id} which the design never states inline`);
    }

    // ------------------------------------------------- Appendix D parity

    const rows = parseAppendixD();
    const rowIds = rows.map((row) => row.id);
    if (rows.length !== requirements.length) {
      fail(
        `Appendix D lists ${rows.length} requirements but the registry has ${requirements.length}`,
      );
    }
    for (const [index, row] of rows.entries()) {
      if (byId.has(row.id) === false) {
        fail(`Appendix D lists ${row.id} which the registry does not contain`);
        continue;
      }
      const entry = byId.get(row.id);
      const registryIndex = requirements.indexOf(entry);
      if (registryIndex !== index) {
        fail(
          `Appendix D row ${index + 1} is ${row.id} but the registry has it at position ${registryIndex + 1}`,
        );
      }
      if (normaliseSource(row.source) !== normaliseSource(entry.source_section)) {
        fail(
          `Appendix D source for ${row.id} is ${JSON.stringify(row.source)} but the registry says ` +
            JSON.stringify(entry.source_section),
        );
      }
      if (row.summary !== entry.summary) {
        fail(`Appendix D summary for ${row.id} differs from the registry summary`);
      }
      if (row.releaseGate !== entry.release_gate) {
        fail(
          `Appendix D release gate for ${row.id} is ${JSON.stringify(row.releaseGate)} but the registry says ` +
            JSON.stringify(entry.release_gate),
        );
      }
    }

    // ------------------------------------------------------------ coverage

    const total = requirements.length;
    const critical = requirements.filter((entry) =>
      RELEASE_CRITICAL_PRIORITIES.has(entry?.priority),
    );
    const criticalWithTests = critical.filter(
      (entry) => Array.isArray(entry.test_ids) && entry.test_ids.length > 0,
    );
    const passed = requirements.filter((entry) => entry?.status === 'PASS');
    console.log(
      `registry: ${total} requirements, ${critical.length} release-critical, ` +
        `${criticalWithTests.length}/${critical.length} with test IDs, ${passed.length} PASS`,
    );
  }
}

// ----------------------------------------------------------------- advisory

for (const line of densityReport()) {
  advise(`requirement density (human review): ${line}`);
}

// ------------------------------------------------------------------- report

for (const line of advisories) console.warn(`advisory: ${line}`);

if (violations.length > 0) {
  console.error(`\nregistry check FAILED with ${violations.length} violation(s):`);
  for (const line of violations) console.error(`  - ${line}`);
  process.exit(1);
}

console.log('registry check passed');
