import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

export const collections = {
	docs: defineCollection({
		loader: docsLoader(),
		schema: docsSchema({
			extend: z.object({
				/** Diátaxis information type, required on every page. */
				type: z.enum(['tutorial', 'how-to', 'reference', 'explanation', 'overview']),
			}),
		}),
	}),
};
