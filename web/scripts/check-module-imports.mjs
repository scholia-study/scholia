#!/usr/bin/env node
/**
 * Enforces module encapsulation: code outside `src/modules/<x>/` may only
 * import from `src/modules/<x>` (the index entry), not from internal files.
 *
 * Walks every .ts/.tsx file under web/src, parses static `from "..."` imports,
 * resolves each relative or `#/`-aliased specifier to a file path, and flags
 * any import that lands on `src/modules/<x>/<not-index>` from a file outside
 * that same module.
 *
 * Run via `pnpm check:modules`. Wired into pre-commit through lefthook.
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "..", "src");
const MODULES_DIR = join(ROOT, "modules");

// Files that biome itself excludes — keep this in sync with biome.json's
// `files.includes` exclusions.
const EXCLUDED = new Set([join(ROOT, "routeTree.gen.ts")]);

function* walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = join(dir, entry.name);
        if (entry.isDirectory()) {
            yield* walk(full);
        } else if (
            (entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) &&
            !EXCLUDED.has(full)
        ) {
            yield full;
        }
    }
}

const IMPORT_REGEX = /(?:^|\s)(?:import|export)[^"';]*?["']([^"']+)["']/g;

/** Resolve a TypeScript import specifier to an absolute file path, if it
 *  refers to source within `web/src`. Returns null for external packages or
 *  unresolvable paths. */
function resolveImport(specifier, fromFile) {
    let base;
    if (specifier.startsWith(".")) {
        base = resolve(dirname(fromFile), specifier);
    } else if (specifier.startsWith("#/")) {
        base = join(ROOT, specifier.slice(2));
    } else {
        return null;
    }
    const candidates = [
        `${base}.ts`,
        `${base}.tsx`,
        join(base, "index.ts"),
        join(base, "index.tsx"),
    ];
    for (const candidate of candidates) {
        try {
            if (statSync(candidate).isFile()) return candidate;
        } catch {}
    }
    return null;
}

/** If `absPath` lives inside `src/modules/<x>/...`, return the path of that
 *  module's root directory (e.g. `/abs/web/src/modules/reader`). Otherwise null. */
function moduleRootOf(absPath) {
    const rel = relative(MODULES_DIR, absPath);
    if (!rel || rel.startsWith("..") || rel.startsWith(sep)) return null;
    const moduleName = rel.split(sep)[0];
    if (!moduleName) return null;
    return join(MODULES_DIR, moduleName);
}

function isModuleIndex(absPath, moduleRoot) {
    return (
        absPath === join(moduleRoot, "index.ts") ||
        absPath === join(moduleRoot, "index.tsx")
    );
}

const violations = [];

for (const file of walk(ROOT)) {
    const content = readFileSync(file, "utf8");
    const fromModule = moduleRootOf(file);

    IMPORT_REGEX.lastIndex = 0;
    for (
        let match = IMPORT_REGEX.exec(content);
        match !== null;
        match = IMPORT_REGEX.exec(content)
    ) {
        const specifier = match[1];
        const resolved = resolveImport(specifier, file);
        if (!resolved) continue;

        const toModule = moduleRootOf(resolved);
        if (!toModule) continue; // import target is not inside a module
        if (fromModule === toModule) continue; // same module — internal access fine
        if (isModuleIndex(resolved, toModule)) continue; // public surface

        violations.push({
            from: relative(process.cwd(), file),
            to: relative(process.cwd(), resolved),
            specifier,
            module: relative(process.cwd(), toModule),
        });
    }
}

if (violations.length > 0) {
    for (const v of violations) {
        console.error(
            `${v.from}: imports "${v.specifier}" → ${v.to} (use ${v.module}/index instead)`,
        );
    }
    console.error(`\n${violations.length} module-encapsulation violation(s).`);
    process.exit(1);
}

console.info("✓ Module imports clean.");
