import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';


export default await defineDocsConfig({
  site: 'https://rsmack.vercel.app',
  domain: 'rsmack.vercel.app',
  title: "rsmack",
  description: "The Rust proc-macro factory: write the exec function, get a documented macro. edoc, wrap, seanum, folder_iso_struct, and megamac.",
  sidebar: [],
  extraStarlight: {
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark']
                        },
                        title: 'rsmack',
                        description:
                                'The Rust proc-macro factory: write the exec function, get a documented macro. edoc, wrap, seanum, folder_iso_struct, and megamac.',
                        favicon: '/favicon.png',
                        social: [
                                {
                                        icon: 'github',
                                        label: 'GitHub',
                                        href: 'https://github.com/elcoosp/rsmack'
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/rsmack/edit/main/'
                        },
                        // Zero-config Pagefind search ships with Starlight and stays enabled.
                        defaultLocale: 'en',
                        sidebar: [
                                {
                                        label: 'Start',
                                        items: [
                                                { label: 'Overview', link: '/' },
                                                { label: 'Getting started', slug: 'tutorials/getting-started' },
                                        ]
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
                                        ]
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
                                        ]
                                },
                                {
                                        label: 'Explanation',
                                        items: [
                                                { slug: 'explanation/architecture' },
                                                { slug: 'explanation/payload-protocol' },
                                                { slug: 'explanation/diagnostics-philosophy' },
                                                { slug: 'explanation/performance' },
                                                { slug: 'explanation/roadmap' },
                                        ]
                                },
                        ],
                        plugins: [starlightLinksValidator(), starlightImageZoom(), starlightHeadingBadges()]
                },
});
