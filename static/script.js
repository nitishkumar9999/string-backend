// ============================================================================
// Scroll to top
// ============================================================================

// ============================================================================
// Scroll position restore
// ============================================================================
window.addEventListener('beforeunload', () => {
    sessionStorage.setItem('scrollPos:' + window.location.pathname, window.scrollY);
});

window.addEventListener('DOMContentLoaded', () => {
    const saved = sessionStorage.getItem('scrollPos:' + window.location.pathname);
    if (saved) {
        window.scrollTo(0, parseInt(saved));
        sessionStorage.removeItem('scrollPos:' + window.location.pathname);
    }
});


const scrollBtn = document.getElementById('scroll-to-top');
if (scrollBtn) {
    let lastScroll = 0;

    window.addEventListener('scroll', () => {
        const currentScroll = window.pageYOffset;
        if (currentScroll < lastScroll && currentScroll > 500) {
            scrollBtn.classList.add('visible');
        } else {
            scrollBtn.classList.remove('visible');
        }
        lastScroll = currentScroll;
    });

    scrollBtn.addEventListener('click', () => {
        window.scrollTo({ top: 0, behavior: 'smooth' });
    });
}

// ============================================================================
// Create post box expand/collapse
// ============================================================================
const postBox = document.getElementById('create-post-box');
if (postBox) {
    const postInput = postBox.querySelector('.create-post-input');

    postInput.addEventListener('focus', () => {
        postBox.classList.add('expanded');
    });

    document.addEventListener('mousedown', (e) => {
        if (!postBox.contains(e.target)) {
            postBox.classList.remove('expanded');
            postInput.blur();
        }
    });
}

// ============================================================================
// Refract expand/collapse
// ============================================================================
document.querySelectorAll('[data-refract-toggle]').forEach(button => {
    button.addEventListener('click', () => {
        const refractId = button.dataset.refractToggle;
        const preview = document.getElementById(`refract-preview-${refractId}`);
        const full = document.getElementById(`refract-full-${refractId}`);
        const embedded = document.getElementById(`embedded-post-${refractId}`);
        const expandText = button.querySelector('.expand-text');
        const iconExpand = button.querySelector('.icon-expand');
        const iconCollapse = button.querySelector('.icon-collapse');

        if (full.style.display === 'none') {
            preview.style.display = 'none';
            full.style.display = 'block';
            if (embedded) embedded.style.display = 'block';
            expandText.textContent = 'See less';
            iconExpand.style.display = 'none';
            iconCollapse.style.display = 'block';
        } else {
            preview.style.display = 'block';
            full.style.display = 'none';
            if (embedded) embedded.style.display = 'none';
            expandText.textContent = 'See more';
            iconExpand.style.display = 'block';
            iconCollapse.style.display = 'none';
        }
    });
});

// ============================================================================
// Settings dropdown
// ============================================================================
(function () {
    const dropdown = document.querySelector('.settings-dropdown');
    const btn = document.getElementById('settings-btn');
    if (!btn || !dropdown) return;

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

// ============================================================================
// Feed tabs
// ============================================================================
document.querySelectorAll('.feed-tab').forEach(tab => {
    tab.addEventListener('click', function () {
        document.querySelectorAll('.feed-tab').forEach(t => t.classList.remove('active'));
        this.classList.add('active');
    });
});

// ============================================================================
// Search tag pills
// ============================================================================
(function () {
    const searchInput = document.getElementById('search-input');
    const searchTagsContainer = document.getElementById('search-tags');
    const searchForm = document.getElementById('search-form');
    const hiddenInput = document.getElementById('search-query-hidden');

    if (!searchInput) return;

    let tags = [];
    let textQuery = '';

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

        searchTagsContainer.querySelectorAll('.remove-tag').forEach(btn => {
            btn.addEventListener('click', () => removeTag(btn.dataset.tag));
        });
    }

    function updateHidden() {
        const tagPart = tags.map(t => `/${t}`).join(' ');
        const textPart = searchInput.value.trim();
        hiddenInput.value = [tagPart, textPart].filter(Boolean).join(' ');
    }

    searchInput.addEventListener('keydown', function (e) {
        const value = this.value.trim();

        if ((e.key === ' ' || e.key === 'Enter') && value.startsWith('/')) {
            e.preventDefault();
            const tag = value.slice(1).trim();
            if (tag) addTag(tag);
            this.value = '';
            if (e.key === 'Enter') submitSearch();
            return;
        }

        if (e.key === 'Backspace' && !this.value && tags.length > 0) {
            removeTag(tags[tags.length - 1]);
            return;
        }

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

    // Populate from URL on page load
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
// Shared helpers (used by bio editor and create.js)
// ============================================================================
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function renderMarkdownPreview(text, targetEl) {
    if (!text.trim()) {
        targetEl.innerHTML = '<p>Nothing to preview yet...</p>';
        return;
    }

    if (typeof marked === 'undefined') {
        targetEl.innerHTML = text
            .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.+?)\*/g, '<em>$1</em>')
            .replace(/`(.+?)`/g, '<code>$1</code>')
            .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>')
            .replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>')
            .replace(/^- (.+)$/gm, '<li>$1</li>')
            .replace(/\n/g, '<br>');
        return;
    }

    const renderer = new marked.Renderer();
    renderer.code = function (code, language) {
        const lang = (language && language.trim() && !language.includes('\n'))
            ? language.trim().toLowerCase() : 'text';
        const lines = code.split('\n');
        if (lines[lines.length - 1] === '') lines.pop();
        const numberedLines = lines.map((line, i) =>
            `<span class="line"><span class="line-number">${i + 1}</span>${escapeHtml(line)}</span>`
        ).join('\n');
        return `<div class="code-block-wrapper" data-language="${lang}">
            <div class="code-block-header">
                <span class="code-language">${lang}</span>
                <button class="copy-button" data-copy-code>Copy</button>
            </div>
            <pre class="code-block"><code>${numberedLines}</code></pre>
        </div>`;
    };

    marked.setOptions({
        renderer,
        breaks: true,
        gfm: true,
        mangle: false,
        sanitize: false,
        smartLists: true,
        smartypants: false,
        xhtml: false,
    });

    try {
        targetEl.innerHTML = marked.parse(text);
        targetEl.querySelectorAll('[data-copy-code]').forEach(btn => {
            btn.addEventListener('click', () => copyCodeFromButton(btn));
        });
    } catch (e) {
        targetEl.innerHTML = '<p style="color:#DC2626;">Error rendering preview.</p>';
    }
}

function copyCodeFromButton(button) {
    const wrapper = button.closest('.code-block-wrapper');
    const code = wrapper.querySelector('code');
    const lines = Array.from(code.querySelectorAll('.line'));
    const text = lines.map(line => {
        const lineNum = line.querySelector('.line-number');
        return line.textContent.substring(lineNum.textContent.length);
    }).join('\n');

    navigator.clipboard.writeText(text).then(() => {
        const orig = button.textContent;
        button.textContent = 'Copied!';
        button.style.background = '#059669';
        setTimeout(() => { button.textContent = orig; button.style.background = ''; }, 2000);
    }).catch(() => {
        button.textContent = 'Failed';
        setTimeout(() => { button.textContent = 'Copy'; }, 2000);
    });
}

// ============================================================================
// Bio editor (edit profile page only)
// ============================================================================
const bioTextarea = document.getElementById('bio-textarea');

if (bioTextarea) {
    const bioContainer = document.getElementById('bio-input');
    const bioPreviewPanel = document.getElementById('bio-preview-panel');
    const bioPreviewToggle = document.getElementById('bio-preview-toggle');
    const bioPreviewContent = document.getElementById('bio-preview-content');
    const bioCharCounter = document.getElementById('bio-char-counter');
    const bioPreviewOverlay = document.getElementById('preview-overlay');

    bioTextarea.addEventListener('focus', () => {
        bioContainer.classList.add('expanded');
    });

    bioTextarea.addEventListener('input', () => {
        bioCharCounter.textContent = `${bioTextarea.value.length} / 1000`;
        if (bioPreviewToggle.classList.contains('active')) {
            renderMarkdownPreview(bioTextarea.value, bioPreviewContent);
        }
    });

    const bioMarkdownButtons = bioContainer.querySelector('.markdown-buttons');
    if (bioMarkdownButtons) {
        bioMarkdownButtons.addEventListener('click', (e) => {
            const btn = e.target.closest('[data-action]');
            if (!btn) return;
            e.preventDefault();
            switch (btn.dataset.action) {
                case 'bold':           insertBioMarkdown('**', '**'); break;
                case 'italic':         insertBioMarkdown('*', '*'); break;
                case 'code':           insertBioMarkdown('`', '`'); break;
                case 'link':           insertBioLink(); break;
                case 'bullet-list':    insertBioMarkdown('- ', ''); break;
                case 'numbered-list':  insertBioMarkdown('1. ', ''); break;
                case 'quote':          insertBioMarkdown('> ', ''); break;
            }
        });
    }

    bioPreviewToggle.addEventListener('click', toggleBioPreview);

    if (bioPreviewOverlay) {
        bioPreviewOverlay.addEventListener('click', () => {
            if (bioPreviewToggle.classList.contains('active')) toggleBioPreview();
        });
    }

    bioContainer.addEventListener('click', (e) => e.stopPropagation());

    document.addEventListener('click', (e) => {
        if (!bioContainer.contains(e.target) &&
            bioContainer.classList.contains('expanded') &&
            !bioContainer.classList.contains('split-view')) {
            bioContainer.classList.remove('expanded');
        }
    });

    function toggleBioPreview() {
        const isActive = bioPreviewToggle.classList.contains('active');
        bioPreviewToggle.classList.toggle('active');
        bioPreviewPanel.classList.toggle('active');
        bioContainer.classList.toggle('split-view');
        if (bioPreviewOverlay) bioPreviewOverlay.classList.toggle('active');
        bioPreviewToggle.textContent = isActive ? 'Live Preview' : 'Cancel Preview';
        if (!isActive) renderMarkdownPreview(bioTextarea.value, bioPreviewContent);
    }

    function insertBioMarkdown(before, after) {
        const start = bioTextarea.selectionStart;
        const end = bioTextarea.selectionEnd;
        const text = bioTextarea.value;
        const selected = text.substring(start, end);
        bioTextarea.value = text.substring(0, start) + before + selected + after + text.substring(end);
        const newPos = start + before.length + selected.length;
        bioTextarea.setSelectionRange(newPos, newPos);
        bioTextarea.focus();
        bioTextarea.dispatchEvent(new Event('input'));
    }

    function insertBioLink() {
        const start = bioTextarea.selectionStart;
        const end = bioTextarea.selectionEnd;
        const text = bioTextarea.value;
        const selected = text.substring(start, end) || 'link text';
        const insertion = `[${selected}](url)`;
        bioTextarea.value = text.substring(0, start) + insertion + text.substring(end);
        const urlStart = start + selected.length + 3;
        bioTextarea.setSelectionRange(urlStart, urlStart + 3);
        bioTextarea.focus();
        bioTextarea.dispatchEvent(new Event('input'));
    }
}

// ============================================================================
// Profile forms (edit profile page) — bypass HTMX, use fetch directly
// ============================================================================
// ============================================================================
// Profile forms (edit profile page) — bypass HTMX, use fetch directly
// ============================================================================
(function () {
    async function handleFormSubmit(form, url, method) {
        const csrf = form.dataset.csrf || '';
        const data = new URLSearchParams(new FormData(form));
        const feedback = document.getElementById('profile-feedback');

        try {
            const res = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                    'X-CSRF-Token': csrf,
                },
                body: data.toString(),
            });

            const text = await res.text();

            if (!res.ok) {
                try {
                    const err = JSON.parse(text);
                    if (feedback) feedback.innerHTML = `<div class="toast toast-danger">${err.message || 'Something went wrong.'}</div>`;
                } catch {
                    if (feedback) feedback.innerHTML = `<div class="toast toast-danger">${text}</div>`;
                }
            } else {
                if (feedback) feedback.innerHTML = text;
            }
        } catch (err) {
            console.error('Form submit error:', err);
            if (feedback) feedback.innerHTML = `<div class="toast toast-danger">Network error. Please try again.</div>`;
        }
    }

    const linksForm = document.getElementById('links-form');
    if (linksForm) {
        linksForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            e.stopPropagation();
            await handleFormSubmit(linksForm, '/profile/update_links_bulk', 'PATCH');
        });
    }

    const basicForm = document.getElementById('basic-info-form');
    if (basicForm) {
        basicForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            e.stopPropagation();
            await handleFormSubmit(basicForm, '/profile/update', 'PATCH');
        });
    }

    const usernameForm = document.getElementById('username-form');
    if (usernameForm) {
        usernameForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            e.stopPropagation();
            await handleFormSubmit(usernameForm, '/profile/username', 'PATCH');
        });
    }
})();

// ============================================================================
// Copy code block
// ============================================================================

document.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-action="copy-code"]');
    if (!btn) return;
    e.stopPropagation();
    const wrapper = btn.closest('.code-block-wrapper');
    const pre = wrapper?.querySelector('pre.code-block');
    if (!pre) return;

    // Clone the pre, remove all line-number spans, get remaining text
    const clone = pre.cloneNode(true);
    clone.querySelectorAll('.line-number').forEach(el => el.remove());
    const text = clone.textContent;

    navigator.clipboard.writeText(text).then(() => {
        btn.textContent = 'Copied!';
        btn.style.background = '#059669';
        setTimeout(() => { btn.textContent = 'Copy'; btn.style.background = ''; }, 2000);
    }).catch((err) => {
        console.error('Clipboard failed:', err);
        // Fallback
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.opacity = '0';
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        btn.textContent = 'Copied!';
        btn.style.background = '#059669';
        setTimeout(() => { btn.textContent = 'Copy'; btn.style.background = ''; }, 2000);
    });
});

// ============================================================================
// Card click navigation
// ============================================================================
document.addEventListener('click', (e) => {
    const card = e.target.closest('.feed-card, .activity-item');
    if (!card) return;

    if (e.target.closest('.action-bar') ||
        e.target.closest('.answer-action-bar') ||
        e.target.closest('.answer-body') ||
        e.target.closest('.tags') ||
        e.target.closest('a') ||
        e.target.closest('[hx-get]') ||
        e.target.closest('[hx-post]') ||
        e.target.closest('[data-action]')) return;

    const embeddedPost = e.target.closest('.embedded-post');
    if (embeddedPost) {
        const href = embeddedPost.dataset.href;
        if (href) window.location.href = href;
        return;
    }

    const slug = card.dataset.slug;
    const type = card.dataset.type;
    if (slug && type) {
        window.location.href = `/${type}s/${slug}`;
    }
});

// ============================================================================
// Copy link buttons
// ============================================================================
document.addEventListener('click', (e) => {
    const btn = e.target.closest('.copy-link-btn');
    if (!btn) return;
    e.stopPropagation();
    const link = btn.dataset.copyLink;
    if (link) navigator.clipboard.writeText(window.location.origin + link);
});

// ============================================================================
// Echo button
// ============================================================================
document.addEventListener('click', async (e) => {
    const btn = e.target.closest('.action-echo');
    if (!btn) return;
    e.stopPropagation();

    const type = btn.dataset.echoType;
    const id = parseInt(btn.dataset.echoId);
    const csrf = document.querySelector('meta[name="csrf-token"]')?.content || '';

    if (!type || !id || !csrf) return;

    btn.disabled = true;

    const body = {};
    if (type === 'post') body.post_id = id;
    else if (type === 'question') body.question_id = id;
    else if (type === 'answer') body.answer_id = id;
    else if (type === 'refract') body.refract_id = id;

    try {
        const res = await fetch('/echo', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': csrf,
            },
            body: JSON.stringify(body),
        });

        if (!res.ok) throw new Error('Failed');
        const html = await res.text();
        btn.outerHTML = html;
    } catch (err) {
        console.error('Echo error:', err);
        btn.disabled = false;
    }
});

// ============================================================================
// Avatar picker
// ============================================================================
document.addEventListener('click', (e) => {
    if (e.target.closest('[data-action="open-avatar-picker"]')) {
        document.getElementById('avatar-file-input')?.click();
    }
});

// ============================================================================
// Create post box
// ============================================================================
(function () {
    const box         = document.getElementById('create-post-box');
    if (!box) return;

    const collapsed   = document.getElementById('create-post-collapsed');
    const expanded    = document.getElementById('create-post-expanded');
    const prompt      = document.getElementById('create-post-prompt');
    const textarea    = document.getElementById('create-post-textarea');
    const submitBtn   = document.getElementById('create-post-submit-btn');
    const cancelBtn   = document.getElementById('create-post-cancel-btn');
    const charCounter = document.getElementById('create-post-char-counter');
    const imageBtn    = document.getElementById('create-post-image-btn');
    const imageInput  = document.getElementById('create-post-image-input');
    const tagsInput   = document.getElementById('create-post-tags-input');
    const tagsHidden  = document.getElementById('create-post-tags-hidden');
    const tagsContainer = document.getElementById('create-post-tags-container');

    let tags = [];

    function openBox() {
        collapsed.style.display = 'none';
        expanded.style.display = 'block';
        textarea.focus();
    }

    function closeBox() {
        expanded.style.display = 'none';
        collapsed.style.display = 'flex';
        textarea.value = '';
        tags = [];
        renderTags();
        tagsInput.value = '';
        tagsHidden.value = '';
        updateCharCounter();
        submitBtn.disabled = true;
    }

    prompt.addEventListener('click', openBox);
    cancelBtn.addEventListener('click', closeBox);

    // Close when clicking outside
    document.addEventListener('mousedown', (e) => {
        if (!box.contains(e.target)) closeBox();
    });

    // Char counter + submit state
    function updateCharCounter() {
        const len = textarea.value.length;
        charCounter.textContent = `${len.toLocaleString()} / 30,000`;
        submitBtn.disabled = textarea.value.trim().length < 10;
    }

    textarea.addEventListener('input', updateCharCounter);

    // Auto-grow textarea
    textarea.addEventListener('input', () => {
        textarea.style.height = 'auto';
        textarea.style.height = textarea.scrollHeight + 'px';
    });

    // Code block insertion
    document.addEventListener('click', (e) => {
        if (!e.target.closest('[data-action="create-post-code"]')) return;
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        const selected = text.substring(start, end);
        const block = `\n\`\`\`language\n${selected || 'your code here'}\n\`\`\`\n`;
        textarea.value = text.substring(0, start) + block + text.substring(end);
        textarea.setSelectionRange(start + 4, start + 12);
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    });

    // Image picker
    imageBtn.addEventListener('click', () => imageInput.click());

    imageInput.addEventListener('change', () => {
        const file = imageInput.files[0];
        if (!file) return;
        const placeholder = `\n![${file.name}](image-url)\n`;
        const start = textarea.selectionStart;
        const text = textarea.value;
        textarea.value = text.substring(0, start) + placeholder + text.substring(start);
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
        imageInput.value = ''; // reset so same file can be picked again
    });

    // ── Tag pill system (exact same as create.js) ─────────────────────────
    function addTag(tag) {
        if (tags.length >= 5) return;
        const normalized = tag.toLowerCase().replace(/[^a-z0-9-]/g, '');
        if (!normalized || tags.includes(normalized)) return;
        tags.push(normalized);
        renderTags();
        updateTagsHidden();
    }

    function removeTag(tag) {
        tags = tags.filter(t => t !== tag);
        renderTags();
        updateTagsHidden();
    }

    function renderTags() {
        tagsContainer.innerHTML = tags.map(tag => `
            <div class="search-tag-pill">
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58s1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41s-.23-1.06-.59-1.42M5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7"></path>
                </svg>
                ${tag}
                <button type="button" class="remove-tag" data-tag="${tag}">×</button>
            </div>
        `).join('');
        tagsContainer.querySelectorAll('.remove-tag').forEach(btn => {
            btn.addEventListener('click', () => removeTag(btn.dataset.tag));
        });
    }

    function updateTagsHidden() {
        tagsHidden.value = tags.join(' ');
    }

    tagsInput.addEventListener('keydown', (e) => {
        const value = tagsInput.value.trim();
        if (e.key === ' ' || e.key === 'Enter') {
            e.preventDefault();
            if (value) addTag(value);
            tagsInput.value = '';
        }
        if (e.key === 'Backspace' && !tagsInput.value && tags.length > 0) {
            removeTag(tags[tags.length - 1]);
        }
    });

    // ── Submit ────────────────────────────────────────────────────────────
    submitBtn.addEventListener('click', async () => {
        const content = textarea.value.trim();
        if (!content || content.length < 10) return;

        const csrf = document.querySelector('meta[name="csrf-token"]')?.content || '';
        if (!csrf) return;

        submitBtn.disabled = true;
        submitBtn.textContent = 'Posting…';

        const formData = new URLSearchParams();
        formData.append('content', content);
        formData.append('tags', tagsHidden.value);
        formData.append('csrf_token', csrf);

        try {
            const res = await fetch('/posts/create', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                    'Accept': 'text/html',
                },
                body: formData.toString(),
            });

            if (!res.ok) {
                const err = await res.text();
                alert(err || 'Failed to post.');
                submitBtn.disabled = false;
                submitBtn.textContent = 'Post';
                return;
            }

            const html = await res.text();
            const feedItems = document.getElementById('feed-items');
            if (feedItems) feedItems.insertAdjacentHTML('afterbegin', html);

            closeBox();
        } catch (e) {
            console.error('Post error:', e);
            submitBtn.disabled = false;
            submitBtn.textContent = 'Post';
        }
    });
})();

document.addEventListener('click', (e) => {
    const btn = e.target.closest('.copy-email-btn');
    if (!btn) return;
    e.stopPropagation();
    const email = btn.dataset.email;
    if (!email) return;
    navigator.clipboard.writeText(email).then(() => {
        const original = btn.innerHTML;
        btn.textContent = 'Email copied!';
        setTimeout(() => { btn.innerHTML = original; }, 2000);
    });
});