const bodyTextarea = document.getElementById('body-textarea');
const bodyEditor = document.getElementById('body-editor');
const previewPanel = document.getElementById('preview-panel');
const previewToggle = document.getElementById('preview-toggle');
const previewContent = document.getElementById('preview-content');
const bodyCharCounter = document.getElementById('body-char-counter');
const previewOverlay = document.getElementById('preview-overlay');

// ============================================
// CHARACTER COUNTER
// ============================================
if (bodyTextarea) {
    bodyTextarea.addEventListener('input', () => {
        const charCount = bodyTextarea.value.length;
        bodyCharCounter.textContent = `${charCount.toLocaleString()} / ${bodyTextarea.maxLength.toLocaleString()}`;

        if (previewToggle.classList.contains('active')) {
            debouncedPreview();
        }
    });
}


// ============================================
// FOCUS STATE
// ============================================
if (bodyTextarea) {
    bodyTextarea.addEventListener('focus', () => {
        bodyEditor.classList.add('focused');
    });

    bodyTextarea.addEventListener('blur', () => {
        if (!previewToggle.classList.contains('active')) {
            bodyEditor.classList.remove('focused');
        }
    });

    // ============================================
    // PREVIEW
    // ============================================

    previewToggle.addEventListener('click', togglePreview);

    previewOverlay.addEventListener('click', () => {
        if (previewToggle.classList.contains('active')) {
            togglePreview();
        }
    });
}

function togglePreview() {
    const isActive = previewToggle.classList.contains('active');

    previewToggle.classList.toggle('active');
    previewPanel.classList.toggle('active');
    bodyEditor.classList.toggle('split-view');
    previewOverlay.classList.toggle('active');

    if (!isActive) {
        previewToggle.textContent = 'Cancel Live Preview';
        bodyEditor.classList.add('focused');
        updatePreview();
    } else {
        previewToggle.textContent = 'Live Preview';
        bodyEditor.classList.remove('focused');
    }
}

function debounce(fn, delay) {
    let timer;
    return function (...args) {
        clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, args), delay);
    };
}

const debouncedPreview = debounce(updatePreview, 400);

function updatePreview() {
    const text = bodyTextarea.value;
    if (!text.trim()) {
        previewContent.innerHTML = '<p>Start typing to see a preview of your post...</p>';
        return;
    }

    const renderer = new marked.Renderer();

    renderer.code = function (code, language, isEscaped) {
        const lang = (language && language.trim() && !language.includes('\n'))
            ? language.trim().toLowerCase()
            : 'text';

        const lines = code.split('\n');
        if (lines[lines.length - 1] === '') lines.pop();

        const numberedLines = lines.map((line, i) => {
            const escaped = escapeHtml(line);
            return `<span class="line"><span class="line-number">${i + 1}</span>${escaped}</span>`;
        }).join('\n');

        return `<div class="code-block-wrapper" data-language="${lang}">
            <div class="code-block-header">
                <span class="code-language">${lang}</span>
                <button class="copy-button" onclick="copyCode(this)">Copy</button>
            </div>
            <pre class="code-block"><code>${numberedLines}</code></pre>
        </div>`;
    };

    marked.setOptions({
        renderer: renderer,
        breaks: true,
        gfm: true,
        mangle: false,
        sanitize: false,
        smartLists: true,
        smartypants: false,
        xhtml: false,
    });

    try {
        const html = marked.parse(text);
        previewContent.innerHTML = html;
    } catch (error) {
        console.error('Markdown parsing error:', error);
        previewContent.innerHTML = '<p style="color: #DC2626;">Error rendering preview. Please check your markdown syntax.</p>';
    }
}

// ============================================
// COPY CODE
// ============================================

function copyCode(button) {
    const wrapper = button.closest('.code-block-wrapper');
    const pre = wrapper?.querySelector('pre.code-block');
    if (!pre) return;

    const clone = pre.cloneNode(true);
    clone.querySelectorAll('.line-number').forEach(el => el.remove());
    const text = clone.textContent;

    navigator.clipboard.writeText(text).then(() => {
        button.textContent = 'Copied!';
        button.style.background = '#059669';
        setTimeout(() => { button.textContent = 'Copy'; button.style.background = ''; }, 2000);
    }).catch(() => {
        button.textContent = 'Failed';
        setTimeout(() => { button.textContent = 'Copy'; }, 2000);
    });
}

// ============================================
// HELPER
// ============================================

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ============================================
// TOOLBAR — EVENT DELEGATION
// ============================================

document.querySelector('.markdown-buttons').addEventListener('click', (e) => {
    const btn = e.target.closest('[data-action]');
    if (!btn) return;
    e.preventDefault();

    switch (btn.dataset.action) {
        case 'bold':           insertMarkdown('**', '**'); break;
        case 'italic':         insertMarkdown('*', '*'); break;
        case 'strikethrough':  insertMarkdown('~~', '~~'); break;
        case 'inline-code':    insertMarkdown('`', '`'); break;
        case 'bullet-list':    insertMarkdown('- ', ''); break;
        case 'numbered-list':  insertMarkdown('1. ', ''); break;
        case 'quote':          insertMarkdown('> ', ''); break;
        case 'hr':             insertMarkdown('\n---\n', ''); break;
        case 'code-block':     insertCodeBlock(); break;
        case 'link':           insertLink(); break;
        case 'image':          insertImage(); break;
        case 'table':          insertTable(); break;
        case 'footnote':       insertFootnote(); break;
    }
});

// Heading toggle button
document.querySelector('.heading-dropdown > .markdown-btn').addEventListener('click', (e) => {
    e.stopPropagation();
    toggleHeadingMenu();
});

// Heading menu options
document.getElementById('heading-menu').addEventListener('click', (e) => {
    const btn = e.target.closest('[data-heading]');
    if (btn) insertHeading(btn.dataset.heading);
});

// Close heading menu when clicking outside
document.addEventListener('click', (e) => {
    const headingDropdown = document.querySelector('.heading-dropdown');
    if (headingDropdown && !headingDropdown.contains(e.target)) {
        document.getElementById('heading-menu').classList.remove('active');
    }
});

// ============================================
// INSERT HELPERS
// ============================================

function insertMarkdown(before, after) {
    const start = bodyTextarea.selectionStart;
    const end = bodyTextarea.selectionEnd;
    const text = bodyTextarea.value;
    const selected = text.substring(start, end);

    const newText = text.substring(0, start) + before + selected + after + text.substring(end);
    bodyTextarea.value = newText;

    const newPos = start + before.length + selected.length;
    bodyTextarea.setSelectionRange(newPos, newPos);
    bodyTextarea.focus();
    bodyTextarea.dispatchEvent(new Event('input'));
}

function toggleHeadingMenu() {
    document.getElementById('heading-menu').classList.toggle('active');
}

function insertHeading(prefix) {
    const start = bodyTextarea.selectionStart;
    const text = bodyTextarea.value;
    const lineStart = text.lastIndexOf('\n', start - 1) + 1;

    const newText = text.substring(0, lineStart) + prefix + text.substring(lineStart);
    bodyTextarea.value = newText;

    const newPos = lineStart + prefix.length;
    bodyTextarea.setSelectionRange(newPos, newPos);
    bodyTextarea.focus();
    document.getElementById('heading-menu').classList.remove('active');
    bodyTextarea.dispatchEvent(new Event('input'));
}

function insertCodeBlock() {
    const start = bodyTextarea.selectionStart;
    const end = bodyTextarea.selectionEnd;
    const text = bodyTextarea.value;
    const selected = text.substring(start, end);

    const insertion = `\n\`\`\`language\n${selected || 'your code here'}\n\`\`\`\n`;
    const newText = text.substring(0, start) + insertion + text.substring(end);
    bodyTextarea.value = newText;

    // Select "language" so user can type over it
    bodyTextarea.setSelectionRange(start + 4, start + 12);
    bodyTextarea.focus();
    bodyTextarea.dispatchEvent(new Event('input'));
}

function insertLink() {
    const start = bodyTextarea.selectionStart;
    const end = bodyTextarea.selectionEnd;
    const text = bodyTextarea.value;
    const selected = text.substring(start, end) || 'link text';

    const insertion = `[${selected}](url)`;
    const newText = text.substring(0, start) + insertion + text.substring(end);
    bodyTextarea.value = newText;

    // Select "url" so user can type over it
    const urlStart = start + selected.length + 3;
    bodyTextarea.setSelectionRange(urlStart, urlStart + 3);
    bodyTextarea.focus();
    bodyTextarea.dispatchEvent(new Event('input'));
}

function insertImage() {
    const start = bodyTextarea.selectionStart;
    const text = bodyTextarea.value;

    const insertion = `\n![alt text](image-url)\n`;
    const newText = text.substring(0, start) + insertion + text.substring(start);
    bodyTextarea.value = newText;

    // Select "image-url" so user can type over it
    const urlStart = start + '\n![alt text]('.length;
    bodyTextarea.setSelectionRange(urlStart, urlStart + 9);
    bodyTextarea.focus();
    bodyTextarea.dispatchEvent(new Event('input'));
}

function insertTable() {
    const start = bodyTextarea.selectionStart;
    const text = bodyTextarea.value;

    const table = '\n| Header | Header | Header |\n| --- | --- | --- |\n| Cell | Cell | Cell |\n| Cell | Cell | Cell |\n\n';
    const newText = text.substring(0, start) + table + text.substring(start);
    bodyTextarea.value = newText;

    bodyTextarea.setSelectionRange(start + table.length, start + table.length);
    bodyTextarea.focus();
    bodyTextarea.dispatchEvent(new Event('input'));
}

let footnoteCounter = 1;
function insertFootnote() {
    const start = bodyTextarea.selectionStart;
    const text = bodyTextarea.value;

    const footnoteRef = `[^${footnoteCounter}]`;
    const footnoteDef = `\n\n[^${footnoteCounter}]: Your footnote text here`;

    const newText = text.substring(0, start) + footnoteRef + text.substring(start) + footnoteDef;
    bodyTextarea.value = newText;

    footnoteCounter++;
    bodyTextarea.setSelectionRange(start + footnoteRef.length, start + footnoteRef.length);
    bodyTextarea.focus();
    bodyTextarea.dispatchEvent(new Event('input'));
}

// ============================================
// KEYBOARD SHORTCUTS
// ============================================

bodyTextarea.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        insertMarkdown('**', '**');
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
        e.preventDefault();
        insertMarkdown('*', '*');
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        insertLink();
    }
});

// ============================================
// TAGS INPUT
// ============================================

const tagsInput = document.getElementById('tags-input');
const tagsContainer = document.getElementById('tags-container');
let tags = [];

tagsInput.addEventListener('keydown', (e) => {
    const value = tagsInput.value.trim();

    if ((e.key === ' ' || e.key === 'Enter') && value) {
        e.preventDefault();
        addTag(value);
        tagsInput.value = '';
    }

    if (e.key === 'Backspace' && !tagsInput.value && tags.length > 0) {
        removeTag(tags[tags.length - 1]);
    }
});

function addTag(tag) {
    if (tags.length >= 5) {
        showTagError('Maximum 5 tags allowed');
        return;
    }
    if (tags.includes(tag)) {
        showTagError('Tag already added');
        return;
    }
    if (tag.length < 2 || tag.length > 30) {
        showTagError('Tag must be between 2 and 30 characters');
        return;
    }
    tags.push(tag);
    renderTags();
}

function removeTag(tag) {
    tags = tags.filter(t => t !== tag);
    renderTags();
}

function renderTags() {
    
    tagsContainer.querySelectorAll('.tag-pill').forEach(pill => pill.remove());

    tags.forEach(tag => {
        const pill = document.createElement('div');
        pill.className = 'tag-pill';
        pill.innerHTML = `
            ${escapeHtml(tag)}
            <button type="button" aria-label="Remove tag">×</button>
        `;
        pill.querySelector('button').addEventListener('click', () => removeTag(tag));
        tagsContainer.insertBefore(pill, tagsInput);
    });

    document.getElementById('tags-hidden').value = tags.join(' ')
    const hiddenEl = document.getElementById('tags-hidden');
    
    if (hiddenEl) hiddenEl.value = tags.join(' ');
    
}

function showTagError(msg) {
    // Show inline error instead of alert
    let err = tagsContainer.parentElement.querySelector('.tag-error');
    if (!err) {
        err = document.createElement('p');
        err.className = 'tag-error form-help';
        err.style.color = '#DC2626';
        tagsContainer.parentElement.appendChild(err);
    }
    err.textContent = msg;
    setTimeout(() => { err.textContent = ''; }, 3000);
}

// ============================================
// FORM SUBMISSION WITH ERROR HANDLING
// ============================================

const createForm = document.getElementById('create-form'); // make sure your form has this id
if (createForm) {
    createForm.addEventListener('submit', async (e) => {
        e.preventDefault();

        const submitBtn = createForm.querySelector('[type="submit"]');
        const originalText = submitBtn?.textContent;
        if (submitBtn) {
            submitBtn.disabled = true;
            submitBtn.textContent = 'Publishing...';
        }

        try {
            const formData = new FormData(createForm);
            const res = await fetch(createForm.action, {
                method: createForm.method || 'POST',
                body: new URLSearchParams(formData),
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded',
                    'Accept': 'text/html',
                },
            });

            if (res.redirected) {
                window.location.href = res.url;
                return;
            }

            if (!res.ok) {
                const messages = {
                    400: 'Invalid content. Please check your input.',
                    401: 'You need to be logged in.',
                    403: 'You do not have permission to do that.',
                    409: 'You posted this content recently. Please wait.',
                    429: 'You are posting too fast. Please wait a moment.',
                    500: 'Server error. Please try again later.',
                };
                if (res.status === 400) {
                    try {
                        const data = await res.json();
                        throw new Error(data.message || 'Invalid content. Please check your input.');
                    } catch {
                        throw new Error('Invalid content. Please check your input.');
                    }
                }

                throw new Error(messages[res.status] || 'Something went wrong.');
            }

            // If server returns HTML (redirect target), navigate there
            window.location.href = res.url || '/feed';

        } catch (err) {
            showFormError(err.message);
            if (submitBtn) {
                submitBtn.disabled = false;
                submitBtn.textContent = originalText;
            }
        }
    });
}

function showFormError(message) {
    let err = document.querySelector('.form-submit-error');
    if (!err) {
        err = document.createElement('p');
        err.className = 'form-submit-error';
        err.style.cssText = 'color:#DC2626;font-family:var(--font-ui);font-size:0.875rem;margin-top:0.5rem;';
        const actions = document.querySelector('.form-actions');
        if (actions) actions.prepend(err);
    }
    err.textContent = message;
    setTimeout(() => err.remove(), 5000);
}