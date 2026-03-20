// ============================================================================
// Search tag pill management
// ============================================================================
(function () {
    const searchInput = document.getElementById('search-input');
    const searchTagsContainer = document.getElementById('search-tags');
    const searchForm = document.getElementById('search-form');
    const hiddenInput = document.getElementById('search-query-hidden');

    if (!searchInput) return;

    let tags = [];
    let textQuery = '';

    // ---- Tag operations ----

    function addTag(tag) {
        if (!tags.includes(tag)) {
            tags.push(tag);
            renderTags();
            updateHidden();
        }
    }

    function removeTag(tag) {
        tags = tags.filter(t => t !== tag);
        renderTags();
        updateHidden();
    }

    function renderTags() {
        searchTagsContainer.innerHTML = tags.map(tag => `
            <div class="search-tag-pill">
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58s1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41s-.23-1.06-.59-1.42M5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7"></path>
                </svg>
                ${tag}
                <button type="button" class="remove-tag" data-tag="${tag}" title="Remove">×</button>
            </div>
        `).join('');

        // Attach remove listeners
        searchTagsContainer.querySelectorAll('.remove-tag').forEach(btn => {
            btn.addEventListener('click', () => removeTag(btn.dataset.tag));
        });
    }

    function updateHidden() {
        const tagPart = tags.map(t => `/${t}`).join(' ');
        const textPart = searchInput.value.trim();
        hiddenInput.value = [tagPart, textPart].filter(Boolean).join(' ');
    }

    // ---- Keyboard handling ----

    searchInput.addEventListener('keydown', function (e) {
        const value = this.value.trim();

        // Space or Enter while typing a /tag → create pill
        if ((e.key === ' ' || e.key === 'Enter') && value.startsWith('/')) {
            e.preventDefault();
            const tag = value.slice(1).trim();
            if (tag) addTag(tag);
            this.value = '';
            if (e.key === 'Enter') submitSearch();
            return;
        }

        // Backspace on empty input removes last tag
        if (e.key === 'Backspace' && !this.value && tags.length > 0) {
            removeTag(tags[tags.length - 1]);
            return;
        }

        // Enter with plain text submits
        if (e.key === 'Enter') {
            e.preventDefault();
            submitSearch();
        }
    });

    searchInput.addEventListener('input', updateHidden);

    function submitSearch() {
        updateHidden();
        if (hiddenInput.value.trim()) searchForm.submit();
    }

    // ---- Populate from URL on page load ----
    // (so the search page shows pills for the current query)
    const urlParams = new URLSearchParams(window.location.search);
    const query = urlParams.get('q');

    if (query) {
        query.split(' ').forEach(part => {
            if (part.startsWith('/')) {
                const tag = part.slice(1);
                if (tag && !tags.includes(tag)) tags.push(tag);
            } else if (part) {
                textQuery += (textQuery ? ' ' : '') + part;
            }
        });
        renderTags();
        searchInput.value = textQuery;
        updateHidden();
    }
})();

// ============================================================================
// Scroll to top (same as feed.js — can be shared later)
// ============================================================================
(function () {
    const scrollBtn = document.getElementById('scroll-to-top');
    if (!scrollBtn) return;
    let lastScroll = 0;

    window.addEventListener('scroll', () => {
        const cur = window.pageYOffset;
        if (cur < lastScroll && cur > 500) {
            scrollBtn.classList.add('visible');
        } else {
            scrollBtn.classList.remove('visible');
        }
        lastScroll = cur;
    });

    scrollBtn.addEventListener('click', () => {
        window.scrollTo({ top: 0, behavior: 'smooth' });
    });
})();

// ============================================================================
// Settings dropdown
// ============================================================================
(function () {
    const dropdown = document.querySelector('.settings-dropdown');
    const btn = document.getElementById('settings-btn');
    if (!btn) return;

    btn.addEventListener('click', e => {
        e.stopPropagation();
        dropdown.classList.toggle('open');
        btn.setAttribute('aria-expanded', dropdown.classList.contains('open'));
    });

    dropdown.addEventListener('click', e => e.stopPropagation());

    document.addEventListener('click', () => {
        dropdown.classList.remove('open');
        btn.setAttribute('aria-expanded', 'false');
    });
})();

document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        document.getElementById('search-input')?.focus();
    }
});