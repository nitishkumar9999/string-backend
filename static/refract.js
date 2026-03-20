// ============================================================================
// REFRACT MODAL — add this block to create.js
// ============================================================================

(function () {
    const overlay = document.getElementById('refract-modal-overlay');
    if (!overlay) return; // only runs on post pages

    const textarea = document.getElementById('refract-textarea');
    const submitBtn = document.getElementById('refract-submit-btn');
    const charCounter = document.getElementById('refract-char-counter');
    const previewBtn = document.getElementById('refract-preview-btn');
    const previewPanel = document.getElementById('refract-preview-panel');
    const previewContent = document.getElementById('refract-preview-content');

    // ── Open / Close ─────────────────────────────────────────────────────────

    // Open when clicking any .refract-btn on the page

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.refract-btn');
        if (!btn) return;
        e.stopPropagation();

        // Read post ID from the button's data attribute and SET it on the hidden input
        const postId = btn.dataset.postId;
        
        
        const hiddenInput = document.getElementById('refract-post-id');
        if (hiddenInput) hiddenInput.value = postId;

        overlay.classList.add('active');
        document.body.style.overflow = 'hidden';
        setTimeout(() => textarea.focus(), 300);
    });

    function closeModal() {
        overlay.classList.remove('active');
        document.body.style.overflow = '';
        setTimeout(() => {
            textarea.value = '';
            updateCounter();
            previewPanel.style.display = 'none';
            previewBtn.textContent = 'Preview';
            previewBtn.classList.remove('active');
        }, 300);
    }

    // Close on overlay click
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) closeModal();
    });

    // Close button
    document.getElementById('refract-modal-close')
        ?.addEventListener('click', closeModal);
    document.getElementById('refract-cancel-btn')
        ?.addEventListener('click', closeModal);

    // Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && overlay.classList.contains('active')) closeModal();
    });

    // ── Char counter + submit enable ──────────────────────────────────────────

    function updateCounter() {
        const len = textarea.value.length;
        const max = 1500;
        charCounter.textContent = `${len} / ${max}`;
        charCounter.classList.toggle('over-limit', len > max);
        submitBtn.disabled = len < 1 || len > max;

        if (previewBtn.classList.contains('active')) {
            debouncedRefractPreview();
        }
    }


    textarea.addEventListener('input', updateCounter);

    // ── Markdown toolbar ──────────────────────────────────────────────────────

    document.querySelector('.refract-md-buttons')
        ?.addEventListener('click', (e) => {
            const btn = e.target.closest('[data-refract-action]');
            if (!btn) return;
            e.preventDefault();
            switch (btn.dataset.refractAction) {
                case 'bold':        insertRefractMarkdown('**', '**'); break;
                case 'italic':      insertRefractMarkdown('*', '*'); break;
                case 'strikethrough': insertRefractMarkdown('~~', '~~'); break;
                case 'inline-code': insertRefractMarkdown('`', '`'); break;
                case 'quote':       insertRefractMarkdown('> ', ''); break;
                case 'bullet-list': insertRefractMarkdown('- ', ''); break;
                case 'link':        insertRefractLink(); break;
            }
        });

    function insertRefractMarkdown(before, after) {
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        const selected = text.substring(start, end);
        textarea.value = text.substring(0, start) + before + selected + after + text.substring(end);
        const newPos = start + before.length + selected.length;
        textarea.setSelectionRange(newPos, newPos);
        textarea.focus();
        updateCounter();
    }

    function insertRefractLink() {
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        const selected = text.substring(start, end) || 'link text';
        const insertion = `[${selected}](url)`;
        textarea.value = text.substring(0, start) + insertion + text.substring(end);
        const urlStart = start + selected.length + 3;
        textarea.setSelectionRange(urlStart, urlStart + 3);
        textarea.focus();
        updateCounter();
    }

    // ── Preview ───────────────────────────────────────────────────────────────

    previewBtn?.addEventListener('click', () => {
        const isActive = previewBtn.classList.contains('active');
        if (isActive) {
            previewPanel.style.display = 'none';
            previewBtn.classList.remove('active');
            previewBtn.textContent = 'Preview';
        } else {
            previewPanel.style.display = 'block';
            previewBtn.classList.add('active');
            previewBtn.textContent = 'Hide Preview';
            updateRefractPreview();
        }
    });

    function updateRefractPreview() {
        const text = textarea.value.trim();
        if (!text) {
            previewContent.innerHTML =
                '<p style="color:var(--text-secondary);font-style:italic">Start typing to see preview...</p>';
            return;
        }
        // reuse renderMarkdownPreview from script.js (already loaded on page)
        if (typeof renderMarkdownPreview === 'function') {
            renderMarkdownPreview(text, previewContent);
        } else {
            previewContent.innerHTML = '<p>Preview unavailable</p>';
        }
    }

    function debounce(fn, delay) {
        let t;
        return (...args) => { clearTimeout(t); t = setTimeout(() => fn(...args), delay); };
    }
    const debouncedRefractPreview = debounce(updateRefractPreview, 400);

    // ── Submit ────────────────────────────────────────────────────────────────

    submitBtn?.addEventListener('click', async () => {
        const content = textarea.value.trim();
        const postId = parseInt(document.getElementById('refract-post-id')?.value);
        const csrf = document.getElementById('refract-csrf')?.value;

        if (!content || !postId || !csrf) return;

        submitBtn.disabled = true;
        submitBtn.textContent = 'Refracting...';

        try {
            const res = await fetch('/refracts/create', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': csrf,
                },
                body: JSON.stringify({
                    original_post_id: postId,
                    content,
                }),
            });

            if (!res.ok) {
                const errorText = await res.text(); // read actual error response
                console.error('Server error:', res.status, errorText);
                const messages = {
                    400: 'Invalid content. Please check your input.',
                    401: 'You need to be logged in.',
                    403: 'You cannot refract your own post.',
                    409: 'You already refracted this content recently.',
                    429: 'Too many refracts. Please wait a moment.',
                };
                throw new Error(messages[res.status] || 'Something went wrong.');
            }

            // Success — close modal and go to feed
            closeModal();
            window.location.href = '/feed';

        } catch (err) {
            showRefractError(err.message);
            submitBtn.disabled = false;
            submitBtn.textContent = 'Refract';
        }
    });

    function showRefractError(message) {
        let err = overlay.querySelector('.refract-error');
        if (!err) {
            err = document.createElement('p');
            err.className = 'refract-error';
            err.style.cssText =
                'color:#dc2626;font-size:0.875rem;font-family:var(--font-ui);padding:0 var(--space-lg) var(--space-sm);';
            document.querySelector('.refract-modal-footer')?.before(err);
        }
        err.textContent = message;
        setTimeout(() => err.remove(), 4000);
    }
})();