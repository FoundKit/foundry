// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
  site: 'https://foundkit.github.io',
  base: process.env.BASE_PATH ?? '/foundry',
  integrations: [
    starlight({
      title: 'Foundry',
      description: 'Build complete multi-tenant systems from a shared foundation.',
      defaultLocale: 'root',
      locales: {
        root: {
          label: 'English',
          lang: 'en',
        },
        zh: {
          label: '简体中文',
          lang: 'zh-CN',
        },
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/FoundKit/foundry',
        },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          translations: {
            'zh-CN': '快速入门',
          },
          link: '/getting-started/',
        },
        {
          label: 'Architecture',
          translations: {
            'zh-CN': '架构设计',
          },
          items: [
            {
              label: 'Architecture Blueprint',
              translations: {
                'zh-CN': '架构设计蓝图',
              },
              link: '/architecture/blueprint/',
            },
          ],
        },
        {
          label: 'Guides',
          translations: {
            'zh-CN': '开发指南',
          },
          items: [
            {
              label: 'Subsystems & Extensions',
              translations: {
                'zh-CN': '子系统与扩展开发',
              },
              link: '/guides/extensions/',
            },
          ],
        },
        {
          label: 'Roadmap',
          translations: {
            'zh-CN': '路线图',
          },
          link: '/roadmap/',
        },
      ],
    }),
  ],
});
