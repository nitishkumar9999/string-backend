// ============================================================================
// comments.js — reuses same patterns as create.js
// ============================================================================

// ── Shared helpers (same as create.js) ─────────────────────────────────────

(function() {



    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function debounce(fn, delay) {
        let timer;
        return function (...args) {
            clearTimeout(timer);
            timer = setTimeout(() => fn.apply(this, args), delay);
        };
    }

    function buildCommentMarkdownRenderer() {
        const renderer = new marked.Renderer();
        renderer.code = function (code, language) {
            const lang = (language && language.trim() && !language.includes('\n'))
                ? language.trim().toLowerCase() : 'text';
            const lines = code.split('\n');
            if (lines[lines.length - 1] === '') lines.pop();
            const numbered = lines.map((line, i) =>
                `<span class="line"><span class="line-number">${i + 1}</span>${escapeHtml(line)}</span>`
            ).join('\n');
            return `<div class="code-block-wrapper" data-language="${lang}">
                <div class="code-block-header">
                    <span class="code-language">${lang}</span>
                    <button class="copy-button" data-action="copy-code">Copy</button>
                </div>
                <pre class="code-block"><code>${numbered}</code></pre>
            </div>`;
        };
        return renderer;
    }

    marked.setOptions({
        renderer: buildCommentMarkdownRenderer(),
        breaks: true,
        gfm: true,
    });

    // ── Get textarea for a given form ───────────────────────────────────────────

    function getCommentTextarea(formId) {
        return document.querySelector(`.comment-textarea[data-form-id="${formId}"]`);
    }

    // ── Insert markdown (same as create.js but targets comment textarea) ────────

    function insertCommentMarkdown(textarea, before, after) {
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        const selected = text.substring(start, end);
        textarea.value = text.substring(0, start) + before + selected + after + text.substring(end);
        const newPos = start + before.length + selected.length;
        textarea.setSelectionRange(newPos, newPos);
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    }

    function insertCommentLink(textarea) {
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        const selected = text.substring(start, end) || 'link text';
        const insertion = `[${selected}](url)`;
        textarea.value = text.substring(0, start) + insertion + text.substring(end);
        const urlStart = start + selected.length + 3;
        textarea.setSelectionRange(urlStart, urlStart + 3);
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    }

    // ── Preview (same logic as create.js) ──────────────────────────────────────

    function updateCommentPreview(formId) {
        const textarea = getCommentTextarea(formId);
        const contentEl = document.getElementById(`preview-content-${formId}`);
        if (!textarea || !contentEl) return;
        const text = textarea.value.trim();
        if (!text) {
            contentEl.innerHTML = '<p style="color:#64748b;font-style:italic">Start typing to see preview...</p>';
            return;
        }

        // Build renderer fresh each time — same as create.js
        const renderer = new marked.Renderer();
        renderer.code = function (code, language) {
            const lang = (language && language.trim() && !language.includes('\n'))
                ? language.trim().toLowerCase() : 'text';
            const lines = code.split('\n');
            if (lines[lines.length - 1] === '') lines.pop();
            const numbered = lines.map((line, i) =>
                `<span class="line"><span class="line-number">${i + 1}</span>${escapeHtml(line)}</span>`
            ).join('\n');
            return `<div class="code-block-wrapper" data-language="${lang}">
                <div class="code-block-header">
                    <span class="code-language">${lang}</span>
                    <button class="copy-button" data-action="copy-code">Copy</button>
                </div>
                <pre class="code-block"><code>${numbered}</code></pre>
            </div>`;
        };
        marked.setOptions({ renderer, breaks: true, gfm: true });

        try {
            contentEl.innerHTML = marked.parse(text);
        } catch (e) {
            contentEl.innerHTML = '<p style="color:#dc2626">Error rendering preview.</p>';
        }
    }

    const debouncedPreviewUpdate = debounce(updateCommentPreview, 400);

    // ── Copy code ───────────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('[data-action="copy-code"]');
        if (!btn) return;
        e.stopPropagation();
        const wrapper = btn.closest('.code-block-wrapper');
        const code = wrapper?.querySelector('code');
        if (!code) return;
        const lines = Array.from(code.querySelectorAll('.line'));
        const text = lines.map(line => {
            const num = line.querySelector('.line-number');
            return line.textContent.substring(num ? num.textContent.length : 0);
        }).join('\n');
        navigator.clipboard.writeText(text).then(() => {
            btn.textContent = 'Copied!';
            btn.style.background = '#059669';
            setTimeout(() => { btn.textContent = 'Copy'; btn.style.background = ''; }, 2000);
        });
    });

    // ── Toolbar: show on focus ──────────────────────────────────────────────────

    document.addEventListener('focusin', (e) => {
        if (!e.target.classList.contains('comment-textarea')) return;
        const formId = e.target.dataset.formId;
        console.log('Form ID:', formId);  // DEBUG: Check what this prints
        console.log('Toolbar element:', document.getElementById(`toolbar-${formId}`));  // DEBUG
        if (!formId) return;
        const toolbar = document.getElementById(`toolbar-${formId}`);
        if (toolbar) toolbar.style.display = 'flex';
        e.target.style.minHeight = '80px';
    });

    // ── Toolbar: hide on outside click (only if empty) ─────────────────────────

    document.addEventListener('click', (e) => {
        if (e.target.closest('.comment-input-container')) return;
        document.querySelectorAll('.comment-toolbar').forEach(toolbar => {
            const formId = toolbar.id.replace('toolbar-', '');
            const textarea = getCommentTextarea(formId);
            if (textarea && !textarea.value.trim()) {
                toolbar.style.display = 'none';
                textarea.style.minHeight = '';
                textarea.style.height = '';
            }
        });
    });

    // ── Input: counter + submit enable + auto-grow + live preview ───────────────

    document.addEventListener('input', (e) => {
        if (!e.target.classList.contains('comment-textarea')) return;
        const textarea = e.target;
        const formId = textarea.dataset.formId;

        // Auto-grow
        textarea.style.height = 'auto';
        textarea.style.height = Math.min(textarea.scrollHeight, 300) + 'px';

        const len = textarea.value.length;
        const max = parseInt(textarea.maxLength);

        // Char counter
        const counter = document.querySelector(`.comment-char-counter[data-form-id="${formId}"]`);
        if (counter) {
            counter.textContent = `${len} / ${max}`;
            counter.classList.toggle('over-limit', len > max);
        }

        // Submit button
        const submitBtn = document.querySelector(`.comment-submit-btn[data-form-id="${formId}"]`);
        if (submitBtn) submitBtn.disabled = len < 1 || len > max;

        // Live preview if open
        const previewPanel = document.getElementById(`preview-${formId}`);
        if (previewPanel && previewPanel.style.display !== 'none') {
            debouncedPreviewUpdate(formId);
        }
    });

    // ── Markdown toolbar buttons ────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.comment-md-btn');
        if (!btn) return;
        e.preventDefault();
        const formId = btn.dataset.form;
        const textarea = getCommentTextarea(formId);
        if (!textarea) return;
        switch (btn.dataset.action) {
            case 'bold':        insertCommentMarkdown(textarea, '**', '**'); break;
            case 'italic':      insertCommentMarkdown(textarea, '*', '*'); break;
            case 'inline-code': insertCommentMarkdown(textarea, '`', '`'); break;
            case 'quote':       insertCommentMarkdown(textarea, '> ', ''); break;
            case 'bullet-list': insertCommentMarkdown(textarea, '- ', ''); break;
            case 'link':        insertCommentLink(textarea); break;
        }
    });

    // ── Preview toggle ──────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.comment-preview-btn');
        if (!btn) return;
        const formId = btn.dataset.formId;
        const panel = document.getElementById(`preview-${formId}`);
        if (!panel) return;
        const isVisible = panel.style.display !== 'none';
        panel.style.display = isVisible ? 'none' : 'block';
        btn.textContent = isVisible ? 'Preview' : 'Hide Preview';
        btn.classList.toggle('active', !isVisible);
        if (!isVisible) updateCommentPreview(formId);
    });

    // ── Reply toggle ────────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.reply-btn');
        if (!btn) return;
        const commentId = btn.dataset.commentId;
        const wrapper = document.getElementById(`reply-input-${commentId}`);
        if (!wrapper) return;
        document.querySelectorAll('.reply-input-wrapper').forEach(w => {
            if (w !== wrapper) w.style.display = 'none';
        });
        const isVisible = wrapper.style.display !== 'none';
        wrapper.style.display = isVisible ? 'none' : 'block';
        if (!isVisible) {
            const ta = wrapper.querySelector('.comment-textarea');
            if (ta) ta.focus();
        }
    });

    // ── Cancel reply ────────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.comment-cancel-btn');
        if (!btn) return;
        const commentId = btn.dataset.commentId;
        const wrapper = document.getElementById(`reply-input-${commentId}`);
        if (wrapper) wrapper.style.display = 'none';
    });


// Manual toggle for subsequent clicks
// ── Show/hide replies ───────────────────────────────────────────────────────
document.addEventListener('click', async (e) => {
    const btn = e.target.closest('.show-replies-btn');
    if (!btn) return;

    e.stopPropagation();

    const commentId = btn.dataset.commentId;
    let container = document.getElementById(`replies-${commentId}`);
    if (!container) return;

    // If already loaded, just toggle visibility
    if (btn.dataset.loaded === 'true') {
        const isVisible = container.style.display !== 'none';
        if (isVisible) {
            container.style.display = 'none';
            btn.classList.remove('expanded');
            const count = btn.dataset.replyCount;
            const span = btn.querySelector('span');
            if (span) span.textContent = `${count} repl${count == 1 ? 'y' : 'ies'}`;
        } else {
            container.style.display = 'block';
            btn.classList.add('expanded');
            const span = btn.querySelector('span');
            if (span) span.textContent = 'Hide replies';
        }
        return;
    }

    // First click — fetch replies
    const span = btn.querySelector('span');
    if (span) span.textContent = 'Loading...';
    btn.disabled = true;

    try {
        const res = await fetch(`/comments/${commentId}/replies`);
        if (!res.ok) throw new Error('Failed to load replies');
        const html = await res.text();
        container.innerHTML = html;
        container.style.display = 'block';
        btn.dataset.loaded = 'true';
        btn.classList.add('expanded');
        if (span) span.textContent = 'Hide replies';
        
        // Re-initialize HTMX on new content if needed
        if (typeof htmx !== 'undefined') {
            htmx.process(container);
        }
    } catch (err) {
        console.error('Failed to load replies:', err);
        if (span) {
            const count = btn.dataset.replyCount;
            span.textContent = `${count} repl${count == 1 ? 'y' : 'ies'}`;
        }
    } finally {
        btn.disabled = false;
    }
});

// Add to comments.js
document.addEventListener('click', async (e) => {
    const btn = e.target.closest('.btn-load-more, .btn-load-more-replies');
    if (!btn) return;
    e.stopPropagation();

    const isReply = btn.classList.contains('btn-load-more-replies');
    const url = btn.dataset.url;
    if (!url) return;

    const wrapper = btn.closest(isReply ? '.load-more-comments' : '.load-more-comments');
    btn.disabled = true;
    btn.textContent = 'Loading...';

    try {
        const res = await fetch(url);
        if (!res.ok) throw new Error('Failed');
        const html = await res.text();

        wrapper?.remove();

        if (isReply) {
            const commentId = btn.dataset.commentId;
            const container = document.getElementById(`replies-${commentId}`);
            if (container) container.insertAdjacentHTML('beforeend', html);
        } else {
            const list = document.getElementById('comment-list');
            if (list) list.insertAdjacentHTML('beforeend', html);
        }

    } catch (err) {
        console.error('Load more failed:', err);
        btn.disabled = false;
        btn.textContent = isReply ? 'Load more replies' : 'Load more comments';
    }
});
// ── Load more replies ───────────────────────────────────────────────────────

document.addEventListener('click', async (e) => {
    const btn = e.target.closest('.btn-load-more');
    if (!btn) return;
    e.stopPropagation();

    const commentId = btn.dataset.commentId;
    const url = btn.dataset.url;
    if (!url) return;

    const wrapper = btn.closest('.load-more-comments');
    btn.disabled = true;
    btn.textContent = 'Loading...';

    try {
        const res = await fetch(url);
        if (!res.ok) throw new Error('Failed');
        const html = await res.text();

        // Remove load more button
        wrapper?.remove();

        // Append new replies to the container
        const container = document.getElementById(`replies-${commentId}`);
        if (container) container.insertAdjacentHTML('beforeend', html);

    } catch (err) {
        console.error('Load more replies failed:', err);
        btn.disabled = false;
        btn.textContent = 'Load more replies';
    }
});

    // ── Submit comment ──────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.comment-submit-btn');
        if (!btn || btn.disabled) return;

        const formId = btn.dataset.formId;
        const textarea = getCommentTextarea(formId);
        if (!textarea) return;

        const content = textarea.value.trim();
        if (!content) return;

        const parentType = btn.dataset.parentType;
        const parentId = parseInt(btn.dataset.parentId);
        const parentCommentId = parseInt(btn.dataset.parentCommentId) || null;
        const depth = parseInt(btn.dataset.depth);
        const csrf = btn.dataset.csrf;
        

        const body = { content };
        if (parentCommentId) body.parent_comment_id = parentCommentId;
        if (parentType === 'post') body.post_id = parentId;
        else if (parentType === 'question') body.question_id = parentId;
        else if (parentType === 'answer' ) body.answer_id = parentId;


        btn.disabled = true;
        btn.textContent = 'Posting...';

        fetch('/comments/create', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': csrf,
            },
            body: JSON.stringify(body),
        })
        .then(res => {
            if (!res.ok) {
                // Map status codes to user-friendly messages
                const messages = {
                    400: 'Invalid comment. Please check your input.',
                    401: 'You need to be logged in to comment.',
                    403: 'You are not allowed to do that.',
                    409: 'You already posted this comment recently.',
                    429: 'You are posting too fast. Please wait a moment.',
                };
                const msg = messages[res.status] || 'Something went wrong. Please try again.';
                return Promise.reject(new Error(msg));
            }
            return res.text();
        })
        .then(html => {
            if (parentCommentId) {
                const container = document.getElementById(`replies-${parentCommentId}`);

                if (!container) {
                    const parentComment = document.getElementById(`comment-${parentCommentId}`);
                    if (parentComment) {
                        container = document.createElement('div');
                        container.className = 'replies-container';
                        container.id = `replies-${parentCommentId}`;
                        parentComment.appendChild(container);
                    }
                }

                if (container) {
                    container.style.display = 'block';
                    container.insertAdjacentHTML('beforeend', html);
                    container.lastElementChild?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                }
                const wrapper = document.getElementById(`reply-input-${parentCommentId}`);
                if (wrapper) wrapper.style.display = 'none';

                const replyBtn = document.querySelector(`.show-replies-btn[data-comment-id="${parentCommentId}"]`);
                if (replyBtn) {
                    const newCount = (parseInt(replyBtn.dataset.replyCount) || 0) + 1;
                    replyBtn.dataset.replyCount = newCount;
                    replyBtn.dataset.loaded = 'true';
                    replyBtn.classList.add('expanded');
                    const span = replyBtn.querySelector('span');
                    if (span) span.textContent = 'Hide replies';
                }
            } else {
                const list = document.getElementById('comment-list');
                if (!list) {
                    console.error('comment-list not found');
                    return;
                }

                const noComments = list.querySelector('.no-comments');
                if (noComments) noComments.remove();

                list.insertAdjacentHTML('afterbegin', html);
                
                // Scroll new comment into view
                list.firstElementChild?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });

                // Update count
                const countEl = document.querySelector('.comments-count');
                if (countEl) countEl.textContent = parseInt(countEl.textContent || '0') + 1;
            }

            // Clear textarea
            textarea.value = '';
            textarea.style.height = '';
            textarea.style.minHeight = '';
            textarea.dispatchEvent(new Event('input'));

            // Hide toolbar for reply forms
            if (depth !== 0) {
                const toolbar = document.getElementById(`toolbar-${formId}`);
                if (toolbar) toolbar.style.display = 'none';
            }

            // Hide preview
            const preview = document.getElementById(`preview-${formId}`);
            if (preview) preview.style.display = 'none';
        })

        .catch(err => {
            console.error('Comment error:', err);
            showCommentError(formId, err?.message || 'Failed to post. Please try again.');
        })
        .finally(() => {
            btn.disabled = false;
            btn.textContent = parentCommentId ? 'Reply' : 'Comment';
        });
    });

    // ── Helpful toggle ──────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.helpful-btn');
        if (!btn) return;
        const commentId = btn.dataset.commentId;
        const csrf = btn.dataset.csrf;
        const isActive = btn.classList.contains('active');
        const countEl = btn.querySelector('.helpful-count');

        const url = isActive 
        ? `/comments/${commentId}/unhelpful`  // DELETE route
        : `/comments/${commentId}/helpful`;   // POST route

        fetch(url, {
            method: isActive ? 'DELETE' : 'POST',
            headers: { 'X-CSRF-Token': csrf },
        })
        .then(res => {
            if (!res.ok) throw new Error('Failed');
            btn.classList.toggle('active', !isActive);
            if (countEl) {
                const n = parseInt(countEl.textContent) || 0;
                countEl.textContent = isActive ? n - 1 : n + 1;
            }
        })
        .catch(err => console.error('Helpful error:', err));
    });

    // ── Error display ───────────────────────────────────────────────────────────

    function showCommentError(formId, message) {
        const container = document.getElementById(`input-container-${formId}`);
        if (!container) return;
        let err = container.querySelector('.comment-error');
        if (!err) {
            err = document.createElement('p');
            err.className = 'comment-error';
            container.appendChild(err);
        }
        err.textContent = message;
        setTimeout(() => err.remove(), 4000);
    }

}) ();