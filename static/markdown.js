const bodyTextarea = document.getElementById('body-textarea');
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