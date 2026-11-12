import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';

export default await defineDocsConfig({
  llmsTxt: {},
  changelog: { repo: 'elcoosp/rsmack' },
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
                                        label: 'GitHub', translations: { fr: "GitHub" },
                                        href: 'https://github.com/elcoosp/rsmack'
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/rsmack/edit/main/'
                        },
                        // Zero-config Pagefind search ships with Starlight and stays enabled.

                        sidebar: [
                                {
                                        label: 'Start', translations: { fr: "Commencer" },
                                        items: [
                                                { label: 'Overview', translations: { fr: 'Aperçu' }, link: '/' },
                                                { label: 'Getting started', translations: { fr: "Commencer" }, slug: 'tutorials/getting-started' },
                                        ]
                                },
                                {
                                        label: 'Guides', translations: { fr: 'Guides' },
                                        items: [
                                                { label: 'Docs from constants (edoc)', translations: { fr: "Docs à partir de constantes (edoc)" }, slug: 'guides/docs-from-constants' },
                                                { label: 'Wrap field types', translations: { fr: "Envelopper les types de champs" }, slug: 'guides/wrap-field-types' },
                                                { label: 'SeaORM enums (seanum)', translations: { fr: "Énumérations SeaORM (seanum)" }, slug: 'guides/seaorm-enums' },
                                                { label: 'Mirror a folder', translations: { fr: "Refléter un dossier" }, slug: 'guides/mirror-a-folder' },
                                                { label: 'Write your own macro', translations: { fr: "Écrire votre propre macro" }, slug: 'guides/write-your-own-macro' },
                                                { label: 'Handle macro errors', translations: { fr: "Gérer les erreurs de macro" }, slug: 'guides/handle-errors' },
                                                { label: 'Preview with the playground', translations: { fr: "Aperçu avec le terrain de jeu" }, slug: 'guides/preview-with-playground' },
                                        ]
                                },
                                {
                                        label: 'Reference', translations: { fr: 'Référence' },
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
                                        label: 'Explanation', translations: { fr: 'Explication' },
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
