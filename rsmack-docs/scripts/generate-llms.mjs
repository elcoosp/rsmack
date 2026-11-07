/**
 * Generates llms.txt and llms-full.txt at the site root (dist/) after `astro build`.
 *
 * llms.txt       — one section per page: title, URL, one-line description.
 * llms-full.txt  — the same index plus the full readable text of every page.
 *
 * Runs as the `postbuild` step so the files always match the built routes.
 */
import { parse } from 'node-html-parser';
import { readdir, readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';

const DIST = 'dist';

async function walk(dir) {
        const entries = await readdir(dir, { withFileTypes: true });
        const files = [];
        for (const e of entries) {
                const p = path.join(dir, e.name);
                if (e.isDirectory()) files.push(...(await walk(p)));
                else if (e.name === 'index.html') files.push(p);
        }
        return files;
}

function toRoute(htmlPath) {
        const rel = path.relative(DIST, path.dirname(htmlPath));
        return rel === '.' ? '/' : `/${rel.replaceAll(path.sep, '/')}/`;
}

function cleanText(root) {
        // Drop scripts/styles and collapse whitespace.
        root.querySelectorAll('script,style,nav,aside').forEach((n) => n.remove());
        return (root.structuredText ?? root.text ?? '')
                .replace(/\r/g, '')
                .split('\n')
                .map((l) => l.trim())
                .filter((l, i, arr) => l.length > 0 || (arr[i - 1] ?? '').length > 0)
                .join('\n')
                .trim();
}

const files = await walk(DIST);
const pages = [];
for (const f of files) {
        const route = toRoute(f);
        const html = await readFile(f, 'utf8');
        const root = parse(html);
        const title = (root.querySelector('title')?.text ?? route).trim().replace(/\s+/g, ' ');
        const main = root.querySelector('main') ?? root;
        const desc =
                root
                        .querySelector('meta[name="description"]')
                        ?.getAttribute('content')
                        ?.trim() ?? '';
        const text = cleanText(main);
        pages.push({ route, title, desc, text });
}

pages.sort((a, b) => {
        if (a.route === '/') return -1;
        if (b.route === '/') return 1;
        return a.route.localeCompare(b.route);
});

await mkdir(DIST, { recursive: true });

const header = [
        '# rsmack',
        '',
        '> The Rust proc-macro factory: write the exec function, get a documented macro.',
        '> edoc, wrap, seanum, folder_iso_struct, and megamac in one dependency.',
        '',
        'Routes are relative to the site root. Rendered pages are plain HTML.',
        '',
].join('\n');

let llms = header;
let full = header + '\n';
for (const p of pages) {
        llms += `- [${p.title}](${p.route})${p.desc ? `: ${p.desc}` : ''}\n`;
        full += `# ${p.title}\n\nURL: ${p.route}\n\n${p.text}\n\n---\n\n`;
}

await writeFile(path.join(DIST, 'llms.txt'), llms, 'utf8');
await writeFile(path.join(DIST, 'llms-full.txt'), full, 'utf8');
// Keep baseline copies in public/ so a fresh clone ships them too; the dist
// copies are always regenerated from the actual build and never drift.
await mkdir('public', { recursive: true });
await writeFile(path.join('public', 'llms.txt'), llms, 'utf8');
await writeFile(path.join('public', 'llms-full.txt'), full, 'utf8');
console.log(`llms.txt: ${pages.length} pages; llms-full.txt: ${full.length} chars`);
