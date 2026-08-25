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

  // --- Modal & Pan-Zoom Viewer Controller ---
  var modalEl = null;
  var viewportEl = null;
  var canvasEl = null;
  var scaleDisplayEl = null;
  var state = {
    scale: 1,
    translateX: 0,
    translateY: 0,
    isDragging: false,
    startX: 0,
    startY: 0,
    naturalWidth: 0,
    naturalHeight: 0,
  };
  var activePointers = new Map();
  var initialPinchDistance = null;
  var initialPinchScale = 1;

  function ensureModal() {
    if (document.getElementById('mermaid-modal')) {
      modalEl = document.getElementById('mermaid-modal');
      viewportEl = document.getElementById('mermaid-viewport');
      canvasEl = document.getElementById('mermaid-canvas');
      scaleDisplayEl = document.getElementById('mermaid-scale-badge');
      return;
    }
    
    var modal = document.createElement('div');
    modal.id = 'mermaid-modal';
    modal.className = 'mermaid-modal';
    modal.setAttribute('aria-hidden', 'true');
    modal.innerHTML = 
      '<div class="mermaid-modal-backdrop"></div>' +
      '<div class="mermaid-modal-container">' +
        '<div class="mermaid-viewport" id="mermaid-viewport">' +
          '<div class="mermaid-canvas" id="mermaid-canvas"></div>' +
        '</div>' +
        '<div class="mermaid-modal-toolbar">' +
          '<div class="mermaid-toolbar-group">' +
            '<button type="button" class="mermaid-tb-btn" data-action="zoom-in" title="放大 (快捷键: +)">' +
              '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line><line x1="11" y1="8" x2="11" y2="14"></line><line x1="8" y1="11" x2="14" y2="11"></line></svg>' +
            '</button>' +
            '<button type="button" class="mermaid-tb-btn" data-action="zoom-out" title="缩小 (快捷键: -)">' +
              '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line><line x1="8" y1="11" x2="14" y2="11"></line></svg>' +
            '</button>' +
            '<span class="mermaid-scale-badge" id="mermaid-scale-badge">100%</span>' +
            '<button type="button" class="mermaid-tb-btn" data-action="fit" title="自适应居中 (快捷键: 0)">' +
              '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path><path d="M3 3v5h5"></path></svg>' +
              '<span>自适应</span>' +
            '</button>' +
            '<button type="button" class="mermaid-tb-btn" data-action="100" title="100% 原始大小 (快捷键: 1)">' +
              '<span>1:1</span>' +
            '</button>' +
          '</div>' +
          '<button type="button" class="mermaid-tb-btn mermaid-close-btn" data-action="close" title="关闭 (ESC)">' +
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>' +
          '</button>' +
        '</div>' +
        '<div class="mermaid-modal-tips">' +
          '<span>💡 鼠标滚轮缩放 · 按住拖拽平移 · 双击自适应 · ESC 退出</span>' +
        '</div>' +
      '</div>';

    document.body.appendChild(modal);

    modalEl = modal;
    viewportEl = document.getElementById('mermaid-viewport');
    canvasEl = document.getElementById('mermaid-canvas');
    scaleDisplayEl = document.getElementById('mermaid-scale-badge');

    // Backdrop click to close
    modal.querySelector('.mermaid-modal-backdrop').addEventListener('click', closeModal);

    // Toolbar click handler
    modal.querySelector('.mermaid-modal-toolbar').addEventListener('click', function(e) {
      var btn = e.target.closest('button');
      if (!btn) return;
      var action = btn.dataset.action;
      if (action === 'zoom-in') zoomAtViewportCenter(1.25);
      else if (action === 'zoom-out') zoomAtViewportCenter(1 / 1.25);
      else if (action === 'fit') fitToViewport();
      else if (action === '100') setScaleActual();
      else if (action === 'close') closeModal();
    });

    // Double click to fit
    viewportEl.addEventListener('dblclick', function(e) {
      if (e.target.closest('.mermaid-modal-toolbar')) return;
      fitToViewport();
    });

    // Wheel zoom
    viewportEl.addEventListener('wheel', function(e) {
      e.preventDefault();
      var rect = viewportEl.getBoundingClientRect();
      var mouseX = e.clientX - rect.left;
      var mouseY = e.clientY - rect.top;
      var factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
      zoomAtPoint(mouseX, mouseY, factor);
    }, { passive: false });

    // Pointer events for Drag and Multi-touch Pinch
    viewportEl.addEventListener('pointerdown', function(e) {
      if (e.button !== 0 && e.pointerType === 'mouse') return;
      activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
      viewportEl.setPointerCapture(e.pointerId);

      if (activePointers.size === 1) {
        state.isDragging = true;
        state.startX = e.clientX - state.translateX;
        state.startY = e.clientY - state.translateY;
        viewportEl.classList.add('is-dragging');
      } else if (activePointers.size === 2) {
        state.isDragging = false;
        var pts = Array.from(activePointers.values());
        initialPinchDistance = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
        initialPinchScale = state.scale;
      }
    });

    viewportEl.addEventListener('pointermove', function(e) {
      if (!activePointers.has(e.pointerId)) return;
      activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });

      if (activePointers.size === 1 && state.isDragging) {
        state.translateX = e.clientX - state.startX;
        state.translateY = e.clientY - state.startY;
        applyTransform();
      } else if (activePointers.size === 2 && initialPinchDistance) {
        var pts = Array.from(activePointers.values());
        var dist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
        var midX = (pts[0].x + pts[1].x) / 2;
        var midY = (pts[0].y + pts[1].y) / 2;
        var rect = viewportEl.getBoundingClientRect();
        var factor = dist / initialPinchDistance;
        var targetScale = initialPinchScale * factor;
        zoomToScaleAtPoint(midX - rect.left, midY - rect.top, targetScale);
      }
    });

    function onPointerEnd(e) {
      if (activePointers.has(e.pointerId)) {
        activePointers.delete(e.pointerId);
        try { viewportEl.releasePointerCapture(e.pointerId); } catch (_) {}
      }
      if (activePointers.size === 0) {
        state.isDragging = false;
        viewportEl.classList.remove('is-dragging');
        initialPinchDistance = null;
      } else if (activePointers.size === 1) {
        var remaining = Array.from(activePointers.values())[0];
        state.isDragging = true;
        state.startX = remaining.x - state.translateX;
        state.startY = remaining.y - state.translateY;
      }
    }

    viewportEl.addEventListener('pointerup', onPointerEnd);
    viewportEl.addEventListener('pointercancel', onPointerEnd);

    // Keyboard controls
    window.addEventListener('keydown', function(e) {
      if (!modalEl || !modalEl.classList.contains('is-open')) return;
      if (e.key === 'Escape') {
        closeModal();
      } else if (e.key === '+' || e.key === '=') {
        zoomAtViewportCenter(1.25);
      } else if (e.key === '-' || e.key === '_') {
        zoomAtViewportCenter(1 / 1.25);
      } else if (e.key === '0') {
        fitToViewport();
      } else if (e.key === '1') {
        setScaleActual();
      }
    });
  }

  function applyTransform() {
    if (!canvasEl) return;
    canvasEl.style.transform = 'translate3d(' + state.translateX + 'px, ' + state.translateY + 'px, 0) scale(' + state.scale + ')';
    if (scaleDisplayEl) {
      scaleDisplayEl.textContent = Math.round(state.scale * 100) + '%';
    }
  }

  function zoomAtPoint(px, py, factor) {
    var newScale = Math.min(Math.max(state.scale * factor, 0.05), 30);
    if (newScale === state.scale) return;
    state.translateX = px - (px - state.translateX) * (newScale / state.scale);
    state.translateY = py - (py - state.translateY) * (newScale / state.scale);
    state.scale = newScale;
    applyTransform();
  }

  function zoomToScaleAtPoint(px, py, targetScale) {
    var newScale = Math.min(Math.max(targetScale, 0.05), 30);
    if (newScale === state.scale) return;
    state.translateX = px - (px - state.translateX) * (newScale / state.scale);
    state.translateY = py - (py - state.translateY) * (newScale / state.scale);
    state.scale = newScale;
    applyTransform();
  }

  function zoomAtViewportCenter(factor) {
    if (!viewportEl) return;
    var rect = viewportEl.getBoundingClientRect();
    zoomAtPoint(rect.width / 2, rect.height / 2, factor);
  }

  function fitToViewport() {
    if (!viewportEl || !state.naturalWidth || !state.naturalHeight) return;
    var vRect = viewportEl.getBoundingClientRect();
    var vWidth = vRect.width || window.innerWidth;
    var vHeight = vRect.height || window.innerHeight;
    
    var padding = 48;
    var availW = Math.max(vWidth - padding * 2, 100);
    var availH = Math.max(vHeight - padding * 2, 100);

    var scaleW = availW / state.naturalWidth;
    var scaleH = availH / state.naturalHeight;
    var fitScale = Math.min(scaleW, scaleH);
    
    if (fitScale > 1.5) fitScale = 1.5;

    state.scale = fitScale;
    state.translateX = (vWidth - state.naturalWidth * fitScale) / 2;
    state.translateY = (vHeight - state.naturalHeight * fitScale) / 2;
    applyTransform();
  }

  function setScaleActual() {
    if (!viewportEl || !state.naturalWidth || !state.naturalHeight) return;
    var vRect = viewportEl.getBoundingClientRect();
    var vWidth = vRect.width || window.innerWidth;
    var vHeight = vRect.height || window.innerHeight;

    state.scale = 1.0;
    state.translateX = (vWidth - state.naturalWidth) / 2;
    state.translateY = (vHeight - state.naturalHeight) / 2;
    applyTransform();
  }

  function openModal(svgElement) {
    ensureModal();
    if (!modalEl || !canvasEl || !viewportEl) return;

    canvasEl.innerHTML = '';
    var clone = svgElement.cloneNode(true);
    clone.removeAttribute('id');
    clone.style.maxWidth = 'none';
    clone.style.width = '100%';
    clone.style.height = '100%';
    canvasEl.appendChild(clone);

    var viewBox = clone.getAttribute('viewBox');
    var w = 0, h = 0;
    if (viewBox) {
      var parts = viewBox.split(/[\s,]+/).map(parseFloat);
      if (parts.length === 4 && parts[2] > 0 && parts[3] > 0) {
        w = parts[2];
        h = parts[3];
      }
    }
    if (!w || !h) {
      w = parseFloat(clone.getAttribute('width')) || svgElement.getBoundingClientRect().width || 800;
      h = parseFloat(clone.getAttribute('height')) || svgElement.getBoundingClientRect().height || 600;
    }

    state.naturalWidth = w;
    state.naturalHeight = h;
    canvasEl.style.width = w + 'px';
    canvasEl.style.height = h + 'px';

    modalEl.classList.add('is-open');
    modalEl.setAttribute('aria-hidden', 'false');
    document.body.classList.add('mermaid-modal-open');

    requestAnimationFrame(function() {
      fitToViewport();
    });
  }

  function closeModal() {
    if (!modalEl) return;
    modalEl.classList.remove('is-open');
    modalEl.setAttribute('aria-hidden', 'true');
    document.body.classList.remove('mermaid-modal-open');
    if (canvasEl) canvasEl.innerHTML = '';
  }

  function attachChartInteractions(chartDiv) {
    if (chartDiv.dataset.interactiveBound === 'true') return;
    chartDiv.dataset.interactiveBound = 'true';

    if (!chartDiv.querySelector('.mermaid-actions')) {
      var actions = document.createElement('div');
      actions.className = 'mermaid-actions';
      actions.innerHTML = 
        '<button type="button" class="mermaid-action-btn" title="点击全屏放大、拖拽平移与滚轮缩放">' +
          '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/></svg>' +
          '<span>放大 / 拖拽查看</span>' +
        '</button>';
      chartDiv.appendChild(actions);
    }

    chartDiv.addEventListener('click', function(e) {
      var svg = chartDiv.querySelector('svg');
      if (!svg) return;
      var selection = window.getSelection();
      if (selection && selection.toString().length > 0) return;
      openModal(svg);
    });
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
          attachChartInteractions(chartDiv);
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
          var actions = chart.querySelector('.mermaid-actions');
          chart.innerHTML = themeRes.svg;
          if (actions) chart.appendChild(actions);
          chart.dataset.interactiveBound = 'false';
          attachChartInteractions(chart);
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
              label: 'Subsystems & Custom Features',
              translations: {
                'zh-CN': '子系统与自定义功能开发',
              },
              link: '/guides/extensions/',
            },
            {
              label: 'Database & Custom Storage',
              translations: {
                'zh-CN': '数据库与自定义存储开发',
              },
              link: '/guides/database/',
            },
            {
              label: 'CLI Tooling Reference',
              translations: {
                'zh-CN': 'CLI 命令行工具指南',
              },
              link: '/guides/cli/',
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
