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
          content: `
(function() {
  let mermaidPromise = null;
  function loadMermaid() {
    if (!mermaidPromise) {
      mermaidPromise = import('https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs')
        .then(function(m) { return m.default || m; });
    }
    return mermaidPromise;
  }

  function getMermaidConfig() {
    var isDark = document.documentElement.dataset.theme === 'dark';
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

  function extractCode(pre) {
    var lines = pre.querySelectorAll('.ec-line');
    if (lines.length > 0) {
      return Array.from(lines)
        .map(function(line) { return line.textContent || ''; })
        .join('\\n')
        .trim();
    }
    return (pre.innerText || pre.textContent || '').trim();
  }

  var isRendering = false;
  async function renderMermaidCharts() {
    var pres = document.querySelectorAll('pre[data-language="mermaid"]');
    var existingCharts = document.querySelectorAll('.mermaid-chart');
    if (pres.length === 0 && existingCharts.length === 0) return;
    if (isRendering) return;
    isRendering = true;

    try {
      var mermaid = await loadMermaid();
      mermaid.initialize(getMermaidConfig());

      // 1. Initial conversion of <pre data-language="mermaid">
      for (var i = 0; i < pres.length; i++) {
        var pre = pres[i];
        var code = extractCode(pre);
        if (!code) continue;
        var container = pre.closest('.expressive-code') || pre;
        var chartDiv = document.createElement('div');
        chartDiv.className = 'mermaid-chart not-content';
        chartDiv.dataset.mermaidSource = code;
        
        var id = 'mermaid_' + i + '_' + Math.random().toString(36).slice(2, 9);
        try {
          var res = await mermaid.render(id, code);
          chartDiv.innerHTML = res.svg;
          container.parentNode.replaceChild(chartDiv, container);
        } catch (err) {
          console.error('Mermaid render error for index ' + i + ':', err);
        }
      }

      // 2. Re-rendering of existing .mermaid-chart on theme change
      for (var j = 0; j < existingCharts.length; j++) {
        var chart = existingCharts[j];
        var chartCode = chart.dataset.mermaidSource;
        if (!chartCode) continue;
        var themeId = 'mermaid_theme_' + j + '_' + Math.random().toString(36).slice(2, 9);
        try {
          var themeRes = await mermaid.render(themeId, chartCode);
          chart.innerHTML = themeRes.svg;
        } catch (err) {
          console.error('Mermaid theme re-render error for index ' + j + ':', err);
        }
      }
    } catch (e) {
      console.error('Mermaid load/render failure:', e);
    } finally {
      isRendering = false;
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', renderMermaidCharts);
  } else {
    renderMermaidCharts();
  }
  window.addEventListener('load', renderMermaidCharts);
  document.addEventListener('astro:page-load', renderMermaidCharts);

  var observer = new MutationObserver(function(mutations) {
    for (var i = 0; i < mutations.length; i++) {
      var m = mutations[i];
      if (m.type === 'attributes' && m.attributeName === 'data-theme') {
        renderMermaidCharts();
      }
    }
  });
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
})();
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
