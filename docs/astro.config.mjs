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
      head: [
        {
          tag: 'script',
          attrs: {
            type: 'module',
          },
          content: `
import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';

function getMermaidConfig() {
  const isDark = document.documentElement.dataset.theme === 'dark';
  return {
    startOnLoad: false,
    theme: isDark ? 'dark' : 'default',
    themeVariables: {
      darkMode: isDark,
      primaryColor: isDark ? '#1e293b' : '#ffedd5',
      primaryTextColor: isDark ? '#f8fafc' : '#0f172a',
      primaryBorderColor: '#ea580c',
      lineColor: isDark ? '#38bdf8' : '#0284c7',
      secondaryColor: isDark ? '#0f172a' : '#f8fafc',
      tertiaryColor: isDark ? '#1e293b' : '#f1f5f9',
      background: isDark ? '#0b0f19' : '#ffffff',
      mainBkg: isDark ? '#0f172a' : '#ffffff',
      nodeBorder: '#ea580c',
      clusterBkg: isDark ? 'rgba(15, 23, 42, 0.7)' : 'rgba(248, 250, 252, 0.8)',
      clusterBorder: isDark ? 'rgba(234, 88, 12, 0.4)' : 'rgba(234, 88, 12, 0.3)',
      titleColor: isDark ? '#f8fafc' : '#0f172a',
      edgeLabelBackground: isDark ? '#0b0f19' : '#ffffff',
    },
    fontFamily: 'inherit',
    securityLevel: 'loose',
  };
}

async function renderMermaidCharts() {
  mermaid.initialize(getMermaidConfig());

  // 1. Initial conversion of <pre data-language="mermaid">
  const pres = document.querySelectorAll('pre[data-language="mermaid"]');
  for (let i = 0; i < pres.length; i++) {
    const pre = pres[i];
    const code = pre.textContent?.trim();
    if (!code) continue;
    const container = pre.closest('.expressive-code') || pre;
    const chartDiv = document.createElement('div');
    chartDiv.className = 'mermaid-chart not-content';
    chartDiv.dataset.mermaidSource = code;
    
    const id = 'mermaid-' + i + '-' + Math.random().toString(36).slice(2, 7);
    try {
      const { svg } = await mermaid.render(id, code);
      chartDiv.innerHTML = svg;
      container.parentNode.replaceChild(chartDiv, container);
    } catch (err) {
      console.error('Mermaid render error:', err);
    }
  }

  // 2. Re-rendering of existing .mermaid-chart on theme change
  const existingCharts = document.querySelectorAll('.mermaid-chart');
  for (let i = 0; i < existingCharts.length; i++) {
    const chart = existingCharts[i];
    const code = chart.dataset.mermaidSource;
    if (!code) continue;
    const id = 'mermaid-theme-' + i + '-' + Math.random().toString(36).slice(2, 7);
    try {
      const { svg } = await mermaid.render(id, code);
      chart.innerHTML = svg;
    } catch (err) {
      console.error('Mermaid theme re-render error:', err);
    }
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', renderMermaidCharts);
} else {
  renderMermaidCharts();
}
document.addEventListener('astro:page-load', renderMermaidCharts);

const observer = new MutationObserver((mutations) => {
  for (const m of mutations) {
    if (m.type === 'attributes' && m.attributeName === 'data-theme') {
      renderMermaidCharts();
    }
  }
});
observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
          `,
        },
      ],
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
      customCss: ['./src/styles/custom.css'],
    }),
  ],
});
