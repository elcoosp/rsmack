// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import astroTakumi from 'astro-takumi';
import fs from 'node:fs';
import { makeRender } from './src/og-render.mjs';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';

// https://astro.build/config
const __logoB64 = fs.readFileSync("./src/assets/logo.png").toString("base64");
const __render = makeRender({
  logoDataUrl: "data:image/png;base64," + __logoB64,
  accentHex: "#743e3e",
  accentHexA12: "#743e3e1a",
  domain: "rsmack.vercel.app",
  projectName: "rsmack",
});

export default defineConfig({
	vite: {
		ssr: { external: ["jsdom", "astro-takumi"] },
	},
        // Canonical deployment URL — Starlight 0.42 always registers the sitemap
        // integration, which requires `site`. Update when the real host is known
        // (recorded as an assumption in docs/decisions/0006-site-and-plugins.md).
        site: 'https://rsmack.vercel.app',
        integrations: [
                starlight({
			routeMiddleware: "./src/routeData.ts",
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark'],
                        },
                        title: 'rsmack',
                        description:
                                'The Rust proc-macro factory: write the exec function, get a documented macro. edoc, wrap, seanum, folder_iso_struct, and megamac.',

                        logo: { src: './src/assets/logo.png' },
                        favicon: '/favicon.png',
                        customCss: ['./src/styles/custom.css'],
                        social: [
                                {
                                        icon: 'github',
                                        label: 'GitHub',
                                        href: 'https://github.com/elcoosp/rsmack',
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/rsmack/edit/main/',
                        },
                        // Zero-config Pagefind search ships with Starlight and stays enabled.
                        defaultLocale: 'en',
                        sidebar: [
                                {
                                        label: 'Start',
                                        items: [
                                                { label: 'Overview', link: '/' },
                                                { label: 'Getting started', slug: 'tutorials/getting-started' },
                                        ],
                                },
                                {
                                        label: 'Guides',
                                        items: [
                                                { label: 'Docs from constants (edoc)', slug: 'guides/docs-from-constants' },
                                                { label: 'Wrap field types', slug: 'guides/wrap-field-types' },
                                                { label: 'SeaORM enums (seanum)', slug: 'guides/seaorm-enums' },
                                                { label: 'Mirror a folder', slug: 'guides/mirror-a-folder' },
                                                { label: 'Write your own macro', slug: 'guides/write-your-own-macro' },
                                                { label: 'Handle macro errors', slug: 'guides/handle-errors' },
                                                { label: 'Preview with the playground', slug: 'guides/preview-with-playground' },
                                        ],
                                },
                                {
                                        label: 'Reference',
                                        items: [
                                                { slug: 'reference/edoc' },
                                                { slug: 'reference/wrap' },
                                                { slug: 'reference/seanum' },
                                                { slug: 'reference/folder-iso-struct' },
                                                { slug: 'reference/megamac' },
                                                { slug: 'reference/rsmack-utils' },
                                                { slug: 'reference/error-messages' },
                                                { slug: 'reference/benchmarks' },
                                        ],
                                },
                                {
                                        label: 'Explanation',
                                        items: [
                                                { slug: 'explanation/architecture' },
                                                { slug: 'explanation/payload-protocol' },
                                                { slug: 'explanation/diagnostics-philosophy' },
                                                { slug: 'explanation/performance' },
                                                { slug: 'explanation/roadmap' },
                                        ],
                                },
                        ],
                        plugins: [starlightLinksValidator(), starlightImageZoom(), starlightHeadingBadges()],
                }),
        
		astroTakumi({
			options: {
				domBackend: "linkedom",
				fonts: [fs.readFileSync("./node_modules/@fontsource/roboto/files/roboto-latin-400-normal.woff2")],
				width: 1200,
				height: 630,
				format: "png",
			},
			render: __render,
		}),
	],
});
