(function () {
  var root = document.documentElement;

  /* ---------- THEME TOGGLE ---------- */
  var themeBtn = document.getElementById('theme-toggle');
  var themeLabel = document.getElementById('theme-label');
  themeBtn.addEventListener('click', function () {
    var next = root.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    root.setAttribute('data-theme', next);
    themeLabel.textContent = next === 'dark' ? 'Dark' : 'Light';
  });

  /* ---------- SIDEBAR GROUP EXPAND/COLLAPSE ---------- */
  document.querySelectorAll('.nav-toggle').forEach(function (btn) {
    btn.addEventListener('click', function () {
      var group = btn.closest('.nav-group');
      var open = group.classList.toggle('open');
      btn.setAttribute('aria-expanded', open ? 'true' : 'false');
    });
  });

  /* ---------- CLOSE MOBILE NAV ON LINK TAP ---------- */
  document.querySelectorAll('.nav-link').forEach(function (link) {
    link.addEventListener('click', function () {
      if (window.innerWidth <= 900) closeNav();
    });
  });

  /* ---------- MOBILE HAMBURGER ---------- */
  var backdrop = document.getElementById('backdrop');
  function closeNav() { document.body.classList.remove('nav-open'); }
  document.getElementById('hamburger').addEventListener('click', function () {
    document.body.classList.toggle('nav-open');
  });
  backdrop.addEventListener('click', closeNav);

  /* ---------- CLASSIC / EMBEDDED SEGMENTED TOGGLE ---------- */
  var segClassic = document.getElementById('seg-classic');
  var segEmbedded = document.getElementById('seg-embedded');
  if (segClassic && segEmbedded) {
    var setFlavor = function (embedded) {
      document.body.classList.toggle('embedded', embedded);
      segEmbedded.classList.toggle('on', embedded);
      segClassic.classList.toggle('on', !embedded);
      segEmbedded.setAttribute('aria-selected', embedded ? 'true' : 'false');
      segClassic.setAttribute('aria-selected', embedded ? 'false' : 'true');
      var article = document.querySelector('article.page');
      if (article) article.scrollIntoView({ behavior: 'smooth', block: 'start' });
    };
    segClassic.addEventListener('click', function () { setFlavor(false); });
    segEmbedded.addEventListener('click', function () { setFlavor(true); });
  }

  /* ---------- SCENARIO APPROACH PICKER ---------- */
  // Scenarios with community-contributed approaches render an "Approach:"
  // dropdown (Classic first) and one `.approach-panel` per option. Options
  // and panels pair by explicit value/data-idx — not DOM position — so the
  // vote code below can reorder options by like-count without breaking the
  // pairing. Cards never nest, so closest() scoping is safe.
  document.querySelectorAll('.approach-select').forEach(function (select) {
    var card = select.closest('.card');
    var panels = Array.prototype.slice.call(card.querySelectorAll('.approach-panel'));
    select.addEventListener('change', function () {
      panels.forEach(function (p) {
        p.classList.toggle('on', p.getAttribute('data-idx') === select.value);
      });
    });
  });

  /* ---------- APPROACH LIKES (GitHub reactions) ---------- */
  // Each community approach maps to a GitHub issue (label `approach-vote`,
  // title = the option's data-vote-key). One unauthenticated API call
  // fetches every issue's 👍 count; we sort approaches by likes (Classic
  // always stays first) and reveal a like chip (with count) linking to the
  // issue. Any failure (offline, rate limit, no issue yet) silently leaves
  // the page exactly as rendered.
  var VOTES_REPO = 'NGDeveloper125/Rust_Wiki';
  if (document.querySelector('.approach-select option[data-vote-key]')) {
    fetchVotes().then(applyVotes).catch(function () { /* graceful no-op */ });
  }

  // Fetch fresh counts on every page load so a reload always reflects the
  // current votes. `cache: 'no-store'` also stops the browser's own HTTP
  // cache from serving a stale API response. Anonymous GitHub API calls are
  // limited to 60/hr per IP; if that's ever hit the fetch just fails and the
  // page keeps its rendered (authored) order — no breakage.
  function fetchVotes() {
    var url = 'https://api.github.com/repos/' + VOTES_REPO +
      '/issues?labels=approach-vote&state=open&per_page=100';
    return fetch(url, { cache: 'no-store' }).then(function (res) {
      if (!res.ok) throw new Error('votes fetch failed: ' + res.status);
      return res.json();
    }).then(function (issues) {
      var votes = {};
      issues.forEach(function (issue) {
        if (issue.pull_request) return; // the issues API also returns PRs
        votes[issue.title] = {
          count: (issue.reactions && issue.reactions['+1']) || 0,
          url: issue.html_url
        };
      });
      return votes;
    });
  }

  function applyVotes(votes) {
    document.querySelectorAll('.approach-select').forEach(function (select) {
      var card = select.closest('.card');
      var voted = false;
      select.querySelectorAll('option[data-vote-key]').forEach(function (opt) {
        var vote = votes[opt.getAttribute('data-vote-key')];
        if (!vote) return;
        voted = true;
        var panel = card.querySelector('.approach-panel[data-idx="' + opt.value + '"]');
        var chip = panel && panel.querySelector('.approach-like');
        if (chip) {
          chip.href = vote.url;
          chip.querySelector('.like-n').textContent = vote.count;
          chip.removeAttribute('hidden');
        }
      });
      if (!voted) return;
      // Re-append options sorted by likes, Classic (value "0") always first;
      // ties keep authored order. Selection stays on Classic.
      var options = Array.prototype.slice.call(select.querySelectorAll('option[data-vote-key]'));
      options
        .map(function (opt, i) {
          var vote = votes[opt.getAttribute('data-vote-key')];
          return { opt: opt, count: vote ? vote.count : 0, i: i };
        })
        .sort(function (a, b) { return b.count - a.count || a.i - b.i; })
        .forEach(function (entry) { select.appendChild(entry.opt); });
    });
  }

  /* ---------- STICKY SECTION-TABS BAR ---------- */
  // Classic and embedded each render their own full set of sections
  // (`.flavor-classic` / `.flavor-embedded`), sharing tab labels via a
  // `data-tab` attribute. Only one flavor is visible (display) at a time,
  // so a tab click/scroll-spy just needs to operate on whichever copy of
  // a `data-tab` is currently visible.
  var tabsBar = document.getElementById('section-tabs');
  if (tabsBar) {
    var tabs = Array.prototype.slice.call(tabsBar.querySelectorAll('.tab'));
    var allTabSections = Array.prototype.slice.call(document.querySelectorAll('[data-tab]'));

    function visibleSection(target) {
      return allTabSections.filter(function (s) {
        return s.dataset.tab === target && s.offsetParent !== null;
      })[0];
    }

    tabs.forEach(function (t) {
      t.addEventListener('click', function () {
        var el = visibleSection(t.dataset.target);
        if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
      });
    });

    function setActiveTab(target) {
      tabs.forEach(function (t) { t.classList.toggle('on', t.dataset.target === target); });
    }

    if ('IntersectionObserver' in window && allTabSections.length) {
      var topOffset = (document.querySelector('.topbar') || {}).offsetHeight || 56;
      var tabsOffset = tabsBar.offsetHeight || 46;
      var spy = new IntersectionObserver(function (entries) {
        entries.forEach(function (entry) {
          if (entry.isIntersecting) setActiveTab(entry.target.dataset.tab);
        });
      }, { rootMargin: '-' + (topOffset + tabsOffset + 1) + 'px 0px -70% 0px', threshold: 0 });
      allTabSections.forEach(function (s) { spy.observe(s); });
    }
  }

  /* ---------- SEARCH ---------- */
  var PAGES = window.SEARCH_INDEX || [];
  var ROOT = window.SITE_ROOT || '';
  var input = document.getElementById('search-input');
  var dd = document.getElementById('search-dropdown');
  var searchWrap = document.getElementById('search');
  var hlIndex = -1, current = [];

  function esc(s){ return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }

  function render(list, q) {
    current = list; hlIndex = -1;
    if (!list.length) { dd.innerHTML = '<div class="sd-empty">No pages match &ldquo;' + esc(q) + '&rdquo;</div>'; return; }
    var html = '<div class="sd-head">' + list.length + ' page' + (list.length>1?'s':'') + '</div>';
    list.forEach(function (p, i) {
      var label = p.isToken ? '<span class="sd-tok">' + esc(p.title) + '</span>' : esc(p.title);
      html += '<button class="sd-item" data-i="' + i + '">' +
        '<span class="sd-badge">' + p.kind + '</span>' +
        '<span class="sd-title">' + label + '</span>' +
        '<span class="sd-crumb">' + esc(p.crumb) + '</span>' +
      '</button>';
    });
    dd.innerHTML = html;
    dd.querySelectorAll('.sd-item').forEach(function (el) {
      el.addEventListener('click', function () { pick(list[+el.dataset.i]); });
    });
  }
  function open() { dd.classList.add('open'); }
  function close() { dd.classList.remove('open'); hlIndex = -1; }
  function pick(p) { window.location.href = ROOT + p.href; }

  function query() {
    var q = input.value.trim().toLowerCase();
    if (!q) {
      // show a default set on focus
      render(PAGES.slice(0, 5), '');
    } else {
      render(PAGES.filter(function (p) {
        return p.title.toLowerCase().indexOf(q) !== -1 ||
               (p.kw || '').indexOf(q) !== -1;
      }), q);
    }
    open();
  }
  input.addEventListener('focus', query);
  input.addEventListener('input', query);
  input.addEventListener('keydown', function (e) {
    var items = dd.querySelectorAll('.sd-item');
    if (e.key === 'ArrowDown') { e.preventDefault(); hlIndex = Math.min(hlIndex + 1, items.length - 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); hlIndex = Math.max(hlIndex - 1, 0); }
    else if (e.key === 'Enter') { if (current[hlIndex]) pick(current[hlIndex]); return; }
    else if (e.key === 'Escape') { close(); input.blur(); return; }
    else return;
    items.forEach(function (el, i) { el.classList.toggle('hl', i === hlIndex); });
  });
  document.addEventListener('click', function (e) { if (!searchWrap.contains(e.target)) close(); });
  document.addEventListener('keydown', function (e) {
    if (e.key === '/' && document.activeElement !== input) { e.preventDefault(); input.focus(); }
  });

  /* ---------- LIGHTWEIGHT RUST SYNTAX HIGHLIGHTER ---------- */
  var KW = new Set(['as','async','await','break','const','continue','crate','dyn','else','enum','extern','false','fn','for','if','impl','in','let','loop','match','mod','move','mut','pub','ref','return','self','static','struct','super','trait','true','type','unsafe','use','where','while']);
  function hl(code) {
    var re = /(\/\/[^\n]*)|("(?:\\.|[^"\\])*")|('(?:\\.|[^'\\])')|(\b\d[\d_]*(?:\.\d+)?(?:f32|f64|u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)?\b)|([A-Za-z_][A-Za-z0-9_]*!)|([A-Za-z_][A-Za-z0-9_]*)/g;
    var out = '', last = 0, m;
    function e(s){ return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
    while ((m = re.exec(code))) {
      out += e(code.slice(last, m.index));
      last = re.lastIndex;
      if (m[1]) out += '<span class="tok-comment">' + e(m[1]) + '</span>';
      else if (m[2]) out += '<span class="tok-string">' + e(m[2]) + '</span>';
      else if (m[3]) out += '<span class="tok-string">' + e(m[3]) + '</span>';
      else if (m[4]) out += '<span class="tok-number">' + e(m[4]) + '</span>';
      else if (m[5]) out += '<span class="tok-macro">' + e(m[5]) + '</span>';
      else {
        var w = m[6];
        if (KW.has(w)) out += '<span class="tok-keyword">' + e(w) + '</span>';
        else if (/^[A-Z]/.test(w) || w === 'Self') out += '<span class="tok-type">' + e(w) + '</span>';
        else if (code[re.lastIndex] === '(') out += '<span class="tok-macro">' + e(w) + '</span>';
        else out += e(w);
      }
    }
    out += e(code.slice(last));
    return out;
  }
  document.querySelectorAll('code.rust').forEach(function (el) {
    // el.textContent already has entities decoded by the browser
    el.innerHTML = hl(el.textContent);
  });
})();
