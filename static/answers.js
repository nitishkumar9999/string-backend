// ============================================================================
// answers.js — answer input, markdown toolbar, preview, submit, load more,
//              echo, delete, copy link
// Mirrors the patterns from comment.js and create.js exactly.
// ============================================================================

(function () {
    'use strict';

    // ── DOM refs ─────────────────────────────────────────────────────────────

    const promptWrapper   = document.getElementById('answer-input-prompt');
    const promptBtn       = document.getElementById('answer-prompt-btn');
    const editorWrapper   = document.getElementById('answer-editor');
    const textarea        = document.getElementById('answer-textarea');
    const charCounter     = document.getElementById('answer-char-counter');
    const submitBtn       = document.getElementById('answer-submit-btn');
    const cancelBtn       = document.getElementById('answer-cancel-btn');
    const previewBtn      = document.getElementById('answer-preview-btn');
    const previewPanel    = document.getElementById('answer-preview-panel');
    const previewOverlay  = document.getElementById('answer-preview-overlay');
    const previewContent  = document.getElementById('answer-preview-content');
    const headingDropdown = document.getElementById('answer-heading-dropdown');
    const headingMenu     = document.getElementById('answer-heading-menu');
    const answerList      = document.getElementById('answer-list');
    const previewBtnSplit  = document.getElementById('answer-preview-btn-split');
    const charCounterSplit = document.getElementById('answer-char-counter-split')

    // Split-view elements (panel on the right, editor clone on the left)
    const editorSplitPanel = document.getElementById('answer-editor-split-panel');
    const textareaSplit    = document.getElementById('answer-textarea-split');

    const MAX_CHARS = 30000;

    // ── Expand / collapse editor ─────────────────────────────────────────────

    function openEditor() {
        if (!promptWrapper || !editorWrapper) return;
        promptWrapper.style.display = 'none';
        editorWrapper.style.display = 'block';
        textarea.focus();
    }

    function closeEditor() {
        if (!promptWrapper || !editorWrapper) return;
        editorWrapper.style.display = 'none';
        promptWrapper.style.display = 'flex';
        textarea.value = '';
        updateCharCounter();
        if (submitBtn) submitBtn.disabled = true;
        closePreview();
    }

    if (promptBtn) promptBtn.addEventListener('click', openEditor);
    if (promptWrapper) promptWrapper.addEventListener('click', openEditor);
    document.addEventListener('click', (e) => {
        if (!editorWrapper || editorWrapper.style.display === 'none') return;
        const container = document.getElementById('answer-input-container');
        if (container && !container.contains(e.target)) {
            closeEditor();
        }
    });

    if (previewBtnSplit) {
        previewBtnSplit.addEventListener('click', closePreview);
    }
    // ── Auto-grow textarea ───────────────────────────────────────────────────

    function autoGrow(el) {
        el.style.height = 'auto';
        el.style.height = el.scrollHeight + 'px';
    }

    if (textarea) {
        textarea.addEventListener('input', () => {
            autoGrow(textarea);
            updateCharCounter();
            updateSubmitState();
        });
    }

    // Split textarea input → sync back to original + live preview
    if (textareaSplit) {
        textareaSplit.addEventListener('input', () => {
            textarea.value = textareaSplit.value;
            updateCharCounter();
            updateSubmitState();
            autoGrow(textareaSplit);
            schedulePreview();
        });
    }

    // ── Char counter ─────────────────────────────────────────────────────────

    function updateCharCounter() {
        if (!charCounter || !textarea) return;
        const len = textarea.value.length;
        const display = `${len.toLocaleString()} / 30,000`;
        charCounter.textContent = display;
        if (charCounterSplit) charCounterSplit.textContent = display;
        charCounter.classList.toggle('over-limit', len > MAX_CHARS);
    }

    function updateSubmitState() {
        if (!submitBtn || !textarea) return;
        const len = textarea.value.trim().length;
        submitBtn.disabled = len < 10 || len > MAX_CHARS;
    }

    // ── Markdown toolbar ─────────────────────────────────────────────────────

    if (headingDropdown) {
        const headingBtn = document.getElementById('answer-heading-btn');
        headingBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            headingMenu.classList.toggle('active');
        });

        headingMenu.addEventListener('click', (e) => {
            const opt = e.target.closest('[data-answer-heading]');
            if (!opt) return;
            insertLinePrefix(opt.dataset.answerHeading);
            headingMenu.classList.remove('active');
        });

        document.addEventListener('click', () => {
            headingMenu.classList.remove('active');
        });

        headingDropdown.addEventListener('click', (e) => e.stopPropagation());
    }

    const toolbar = editorWrapper
        ? editorWrapper.querySelector('.answer-editor-toolbar')
        : null;

    if (toolbar) {
        toolbar.addEventListener('click', (e) => {
            const btn = e.target.closest('[data-answer-action]');
            if (!btn) return;
            e.preventDefault();
            handleToolbarAction(btn.dataset.answerAction);
        });
    }

    // Prevent toolbar buttons from stealing focus from textarea
    document.querySelectorAll('.answer-md-btn, .answer-heading-option').forEach(btn => {
        btn.addEventListener('mousedown', (e) => e.preventDefault());
    });

    const splitToolbar = editorSplitPanel
    ? editorSplitPanel.querySelector('.answer-editor-toolbar')
    : null;

    if (splitToolbar) {
        splitToolbar.addEventListener('click', (e) => {
            const btn = e.target.closest('[data-answer-action]');
            if (!btn) return;
            e.preventDefault();
            handleToolbarAction(btn.dataset.answerAction);
        });

        const splitHeadingBtn  = document.getElementById('answer-heading-btn-split');
        const splitHeadingMenu = document.getElementById('answer-heading-menu-split');

        if (splitHeadingBtn && splitHeadingMenu) {
            splitHeadingBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                splitHeadingMenu.classList.toggle('active');
            });

            splitHeadingMenu.addEventListener('click', (e) => {
                const opt = e.target.closest('[data-answer-heading]');
                if (!opt) return;
                insertLinePrefix(opt.dataset.answerHeading);
                splitHeadingMenu.classList.remove('active');
            });

            document.addEventListener('click', () => {
                splitHeadingMenu.classList.remove('active');
            });

            splitToolbar.addEventListener('click', (e) => e.stopPropagation());
        }
    }

    function handleToolbarAction(action) {
        if (!textarea) return;
        switch (action) {
            case 'bold':           wrapSelection('**', '**');           break;
            case 'italic':         wrapSelection('*', '*');             break;
            case 'strikethrough':  wrapSelection('~~', '~~');           break;
            case 'inline-code':    wrapSelection('`', '`');             break;
            case 'code-block':     insertCodeBlock();                   break;
            case 'link':           insertLink();                        break;
            case 'image':          insertImage();                       break;
            case 'bullet-list':    insertLinePrefix('- ');              break;
            case 'numbered-list':  insertLinePrefix('1. ');             break;
            case 'quote':          insertLinePrefix('> ');              break;
            case 'table':          insertTable();                       break;
            case 'hr':             insertHr();                          break;
            case 'footnote':       insertFootnote();                    break;
        }
        getActiveTextarea().focus();
        getActiveTextarea().dispatchEvent(new Event('input'));
    }

    // Keyboard shortcuts
    if (textarea) {
        textarea.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                if (e.key === 'b') { e.preventDefault(); wrapSelection('**', '**'); getActiveTextarea().dispatchEvent(new Event('input')); }
                if (e.key === 'i') { e.preventDefault(); wrapSelection('*', '*');   getActiveTextarea().dispatchEvent(new Event('input')); }
                if (e.key === 'k') { e.preventDefault(); insertLink();              getActiveTextarea().dispatchEvent(new Event('input')); }
            }
        });
    }

    // ── Always target the active textarea ────────────────────────────────────

    function getActiveTextarea() {
        return (previewBtn && previewBtn.classList.contains('active') && textareaSplit)
            ? textareaSplit
            : textarea;
    }

    // ── Insertion helpers ────────────────────────────────────────────────────

    function wrapSelection(before, after) {
        const ta     = getActiveTextarea();
        const start  = ta.selectionStart;
        const end    = ta.selectionEnd;
        const text   = ta.value;
        const sel    = text.substring(start, end);
        ta.value = text.substring(0, start) + before + sel + after + text.substring(end);
        const cur = start + before.length + sel.length;
        ta.setSelectionRange(cur, cur);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    function insertLinePrefix(prefix) {
        const ta        = getActiveTextarea();
        const start     = ta.selectionStart;
        const text      = ta.value;
        const lineStart = text.lastIndexOf('\n', start - 1) + 1;
        ta.value        = text.substring(0, lineStart) + prefix + text.substring(lineStart);
        const cur       = start + prefix.length;
        ta.setSelectionRange(cur, cur);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    function insertCodeBlock() {
        const ta    = getActiveTextarea();
        const start = ta.selectionStart;
        const end   = ta.selectionEnd;
        const text  = ta.value;
        const sel   = text.substring(start, end);
        const block = `\n\`\`\`language\n${sel || 'your code here'}\n\`\`\`\n`;
        ta.value = text.substring(0, start) + block + text.substring(end);
        ta.setSelectionRange(start + 4, start + 12);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    function insertLink() {
        const ta    = getActiveTextarea();
        const start = ta.selectionStart;
        const end   = ta.selectionEnd;
        const text  = ta.value;
        const sel   = text.substring(start, end) || 'link text';
        const ins   = `[${sel}](url)`;
        ta.value = text.substring(0, start) + ins + text.substring(end);
        const urlStart = start + sel.length + 3;
        ta.setSelectionRange(urlStart, urlStart + 3);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    function insertImage() {
        const ta    = getActiveTextarea();
        const start = ta.selectionStart;
        const text  = ta.value;
        const ins   = '\n![alt text](image-url)\n';
        ta.value = text.substring(0, start) + ins + text.substring(start);
        const urlStart = start + '\n![alt text]('.length;
        ta.setSelectionRange(urlStart, urlStart + 9);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    function insertTable() {
        const ta    = getActiveTextarea();
        const start = ta.selectionStart;
        const text  = ta.value;
        const tbl   = '\n| Header | Header | Header |\n| --- | --- | --- |\n| Cell | Cell | Cell |\n| Cell | Cell | Cell |\n\n';
        ta.value = text.substring(0, start) + tbl + text.substring(start);
        ta.setSelectionRange(start + tbl.length, start + tbl.length);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    function insertHr() {
        const ta    = getActiveTextarea();
        const start = ta.selectionStart;
        const text  = ta.value;
        const hr    = '\n---\n';
        ta.value = text.substring(0, start) + hr + text.substring(start);
        ta.setSelectionRange(start + hr.length, start + hr.length);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    let footnoteCounter = 1;
    function insertFootnote() {
        const ta    = getActiveTextarea();
        const start = ta.selectionStart;
        const text  = ta.value;

        const footnoteRef = `[^${footnoteCounter}]`;
        const footnoteDef = `\n\n[^${footnoteCounter}]: Your footnote text here`;

        ta.value = text.substring(0, start) + footnoteRef + text.substring(start) + footnoteDef;
        footnoteCounter++;
        ta.setSelectionRange(start + footnoteRef.length, start + footnoteRef.length);
        if (ta === textareaSplit) { textarea.value = textareaSplit.value; schedulePreview(); }
        updateCharCounter();
        updateSubmitState();
    }

    // ── Live preview ─────────────────────────────────────────────────────────

    let previewTimer = null;

    function schedulePreview() {
        clearTimeout(previewTimer);
        previewTimer = setTimeout(() => {
            if (previewContent) renderMarkdownPreview(textarea.value, previewContent);
        }, 300);
    }

    function openPreview() {
        if (!previewPanel || !previewBtn) return;

        // Sync split textarea FROM original before showing
        if (textareaSplit) {
            textareaSplit.value = textarea.value;
            textareaSplit.style.height = 'auto';
            textareaSplit.style.height = Math.max(textareaSplit.scrollHeight, 300) + 'px';
        }

        previewPanel.classList.add('active');
        previewBtn.classList.add('active');
        previewBtn.textContent = 'Cancel Preview';
        if (previewOverlay) previewOverlay.classList.add('active');
        if (editorWrapper)  editorWrapper.classList.add('split-view');
        if (editorSplitPanel) {
            editorSplitPanel.style.display = 'flex';
            editorSplitPanel.classList.add('active');
        }

        renderMarkdownPreview(textarea.value, previewContent);

        // Focus split textarea so user types there
        if (textareaSplit) textareaSplit.focus();
    }

    function closePreview() {
        if (!previewPanel || !previewBtn) return;

        // Sync original FROM split — capture anything typed in split
        if (textareaSplit) textarea.value = textareaSplit.value;

        previewPanel.classList.remove('active');
        previewBtn.classList.remove('active');
        previewBtn.textContent = 'Preview';
        if (previewOverlay) previewOverlay.classList.remove('active');
        if (editorWrapper)  editorWrapper.classList.remove('split-view');
        if (editorSplitPanel) {
            editorSplitPanel.style.display = 'none';
            editorSplitPanel.classList.remove('active');

        }

        updateCharCounter();
        updateSubmitState();
        if (textarea) textarea.focus();
    }

    if (previewBtn) {
        previewBtn.addEventListener('click', () => {
            previewBtn.classList.contains('active') ? closePreview() : openPreview();
        });
    }

    if (previewOverlay) {
        previewOverlay.addEventListener('click', closePreview);
    }

    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && previewBtn && previewBtn.classList.contains('active')) {
            closePreview();
        }
    });

    // ── Markdown renderer (matches create.js exactly) ─────────────────────────

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function renderMarkdownPreview(text, targetEl) {
        if (!targetEl) return;
        if (!text || !text.trim()) {
            targetEl.innerHTML = '<p style="color:var(--text-secondary);font-style:italic">Start typing to see a preview...</p>';
            return;
        }

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
        } catch (e) {
            targetEl.innerHTML = '<p style="color:#dc2626">Error rendering preview.</p>';
        }
    }

    // Copy code button inside preview
    // Replace the existing copy handler in answers.js:
document.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-action="copy-code"]');
    if (!btn) return;
    e.stopPropagation();
    const wrapper = btn.closest('.code-block-wrapper');
    const pre = wrapper?.querySelector('pre.code-block');
    if (!pre) return;

    const clone = pre.cloneNode(true);
    clone.querySelectorAll('.line-number').forEach(el => el.remove());
    const text = clone.textContent;

    navigator.clipboard.writeText(text).then(() => {
        btn.textContent = 'Copied!';
        btn.style.background = '#059669';
        setTimeout(() => { btn.textContent = 'Copy'; btn.style.background = ''; }, 2000);
    });
});


    // ── Submit answer ─────────────────────────────────────────────────────────

    if (submitBtn) {
        submitBtn.addEventListener('click', async () => {
            // Ensure original textarea has latest value if user typed in split
            if (previewBtn && previewBtn.classList.contains('active') && textareaSplit) {
                textarea.value = textareaSplit.value;
            }

            const content      = textarea.value.trim();
            const questionId   = submitBtn.dataset.questionId;
            const questionSlug = submitBtn.dataset.questionSlug;
            const csrf         = submitBtn.dataset.csrf;

            if (!content || content.length < 10 || content.length > MAX_CHARS) return;

            submitBtn.disabled = true;
            submitBtn.textContent = 'Posting…';

            try {
                const res = await fetch(`/questions/${questionId}/answer`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-CSRF-Token': csrf,
                    },
                    body: JSON.stringify({ content }),
                });

                if (!res.ok) {
                    const err = await res.text();
                    showAnswerError(err || 'Failed to post answer.');
                    submitBtn.disabled = false;
                    submitBtn.textContent = 'Post Answer';
                    return;
                }

                const html = await res.text();

                const noAnswers = answerList.querySelector('.no-answers');
                if (noAnswers) noAnswers.remove();

                const existingLoadMore = answerList.querySelector('#load-more-answers');
                if (existingLoadMore) existingLoadMore.remove();

                answerList.insertAdjacentHTML('afterbegin', html);

                if (existingLoadMore) answerList.appendChild(existingLoadMore);

                updateAnswerCount(1);
                closeEditor();
                closePreview();

                const firstCard = answerList.querySelector('.answer-card');
                if (firstCard) firstCard.scrollIntoView({ behavior: 'smooth', block: 'nearest' });

            } catch (e) {
                showAnswerError('Network error. Please try again.');
                submitBtn.disabled = false;
                submitBtn.textContent = 'Post Answer';
            }
        });
    }

    function showAnswerError(msg) {
        let errEl = document.querySelector('.answer-error');
        if (!errEl) {
            errEl = document.createElement('div');
            errEl.className = 'answer-error';
            if (editorWrapper) editorWrapper.appendChild(errEl);
        }
        errEl.textContent = msg;
        setTimeout(() => errEl.remove(), 5000);
    }

    function updateAnswerCount(delta) {
        const countEl = document.querySelector('.answers-count');
        if (!countEl) return;
        const next = (parseInt(countEl.textContent, 10) || 0) + delta;
        countEl.textContent = next;
        const title = countEl.closest('.answers-title');
        if (title) {
            title.childNodes.forEach(node => {
                if (node.nodeType === Node.TEXT_NODE) {
                    node.textContent = next === 1 ? ' Answer' : ' Answers';
                }
            });
        }
    }

    // ── Load more answers ─────────────────────────────────────────────────────

    document.addEventListener('click', async (e) => {
        const btn = e.target.closest('.btn-load-more-answers');
        if (!btn) return;

        const questionId = btn.dataset.questionId;
        const questionSlug = btn.dataset.questionSlug;
        const cursor     = btn.dataset.cursor;
        if (!questionId || !cursor) return;

        btn.disabled = true;
        btn.textContent = 'Loading…';

        try {
            const res = await fetch(
                `/questions/${questionSlug}/answers?cursor=${cursor}&limit=3`,
                { headers: { 'Accept': 'text/html' } }
            );

            if (!res.ok) {
                btn.disabled = false;
                btn.innerHTML = `Load more answers`;
                return;
            }

            const html = await res.text();
            const container = btn.closest('#load-more-answers');
            if (container) {
                container.insertAdjacentHTML('beforebegin', html);
                container.remove();
            } else {
                answerList.insertAdjacentHTML('beforeend', html);
            }

        } catch {
            btn.disabled = false;
            btn.innerHTML = `${ICON_CHEVRON_DOWN_STR} Load more answers`;
        }
    });

    // ── Echo ─────────────────────────────────────────────────────────────────

    document.addEventListener('click', async (e) => {
        const btn = e.target.closest('.answer-echo-btn');
        if (!btn) return;

        const answerId = btn.dataset.answerId;
        const csrf     = btn.dataset.csrf;
        if (!answerId) return;

        const countEl = btn.querySelector('.answer-echo-count');
        const cur     = parseInt(countEl.textContent, 10) || 0;
        const echoed  = btn.classList.contains('echoed');

        // Optimistic update
        btn.classList.toggle('echoed', !echoed);
        countEl.textContent = echoed ? Math.max(0, cur - 1) : cur + 1;

        try {
            const res = await fetch('/echo', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': csrf,
                },
                body: JSON.stringify({ answer_id: parseInt(answerId, 10) }),
            });

            if (!res.ok) {
                btn.classList.toggle('echoed', echoed);
                countEl.textContent = cur;
            }
        } catch {
            btn.classList.toggle('echoed', echoed);
            countEl.textContent = cur;
        }
    });

    // ── Delete ────────────────────────────────────────────────────────────────

    document.addEventListener('click', async (e) => {
        const btn = e.target.closest('.answer-delete-btn');
        if (!btn) return;

        const answerId = btn.dataset.answerId;
        const csrf     = btn.dataset.csrf;
        if (!answerId) return;

        if (!confirm('Delete this answer? This cannot be undone.')) return;

        btn.disabled = true;

        try {
            const res = await fetch(`/api/answers/${answerId}`, {
                method: 'DELETE',
                headers: { 'X-CSRF-Token': csrf },
            });

            if (res.ok) {
                const card = btn.closest('.answer-card');
                if (card) {
                    card.style.transition = 'opacity 0.25s, transform 0.25s';
                    card.style.opacity    = '0';
                    card.style.transform  = 'translateY(-4px)';
                    setTimeout(() => { card.remove(); updateAnswerCount(-1); }, 250);
                }
            } else {
                btn.disabled = false;
                showAnswerError('Could not delete answer. Please try again.');
            }
        } catch {
            btn.disabled = false;
            showAnswerError('Network error. Please try again.');
        }
    });

    // ── Copy link ─────────────────────────────────────────────────────────────

    document.addEventListener('click', (e) => {
        const btn = e.target.closest('.answer-copy-link-btn');
        if (!btn) return;
        e.stopPropagation();
        const link = btn.dataset.copyLink;
        if (!link) return;
        navigator.clipboard.writeText(window.location.origin + link).then(() => {
            const orig = btn.title;
            btn.title  = 'Copied!';
            btn.style.color = 'var(--accent)';
            setTimeout(() => { btn.title = orig; btn.style.color = ''; }, 2000);
        });
    });

    // ── Scroll to anchor on page load ─────────────────────────────────────────

    if (window.location.hash) {
        const target = document.querySelector(window.location.hash);
        if (target && target.classList.contains('answer-card')) {
            setTimeout(() => target.scrollIntoView({ behavior: 'smooth', block: 'start' }), 300);
        }
    }

    // ── Tabs (answers / comments) ─────────────────────────────────────────────

    // Tab switching
    const answersTab      = document.getElementById('answers-tab');
    const commentsTab     = document.getElementById('comments-tab');
    const answersSection  = document.getElementById('answers-section');
    const commentsSection = document.getElementById('comments-section');

    if (answersTab && commentsTab) {
        answersTab.addEventListener('click', () => {
            answersTab.classList.add('active');
            commentsTab.classList.remove('active');
            answersSection.style.display = 'block';
            commentsSection.style.display = 'none';
        });

        commentsTab.addEventListener('click', () => {
            commentsTab.classList.add('active');
            answersTab.classList.remove('active');
            commentsSection.style.display = 'block';
            answersSection.style.display = 'none';
        });

        // Handle hash navigation from feed links
        setTimeout(() => {
            const hash = window.location.hash;
            if (hash === '#comments') {
                commentsTab.click();
            } else if (hash === '#answers') {
                answersTab.click();
            }
        }, 50);
        
    }

})();

