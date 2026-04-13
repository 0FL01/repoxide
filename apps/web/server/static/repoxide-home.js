(function () {
  const MAX_UPLOAD_BYTES = 50 * 1024 * 1024;
  const CHUNK_THRESHOLD_BYTES = 2 * 1024 * 1024;
  const CHUNK_SIZE_BYTES = 1024 * 1024;
  const THEME_STORAGE_KEY = 'repoxide-theme-preference';
  const THEME_MEDIA_QUERY = '(prefers-color-scheme: dark)';

  document.addEventListener('DOMContentLoaded', () => {
    initTheme();

    const form = document.querySelector('[data-home-form]');
    if (!(form instanceof HTMLFormElement)) {
      return;
    }

    const responseRoot = form.querySelector('[data-response-root]');
    const sourceKindInput = form.querySelector('[data-source-kind-input]');
    const formatInput = form.querySelector('[data-format-input]');
    const urlInput = form.querySelector('[data-url-input]');
    const urlWarning = form.querySelector('[data-url-warning]');
    const packButton = form.querySelector('[data-pack-button]');
    const packButtonLabel = form.querySelector('[data-pack-button-label]');
    const resetTooltip = form.querySelector('[data-reset-tooltip]');
    const loadingTemplate = document.getElementById('home-loading-template');

    if (!(responseRoot instanceof HTMLElement) || !(sourceKindInput instanceof HTMLInputElement) || !(formatInput instanceof HTMLInputElement) || !(packButton instanceof HTMLButtonElement)) {
      return;
    }

    const state = {
      mode: sourceKindInput.value || 'url',
      loading: false,
      controller: null,
      zipFile: null,
      zipUploadId: null,
      folderFiles: [],
      folderLabel: '',
      zipError: '',
      folderError: '',
    };

    const modePanels = new Map(Array.from(form.querySelectorAll('[data-mode-panel]')).map((panel) => [panel.getAttribute('data-mode-panel'), panel]));
    const modeTabs = Array.from(form.querySelectorAll('[data-mode-tab]'));
    const formatButtons = Array.from(form.querySelectorAll('[data-format-button]'));
    const zipZone = form.querySelector('[data-upload-zone="zip"]');
    const folderZone = form.querySelector('[data-upload-zone="folder"]');
    const zipInput = form.querySelector('[data-upload-input="zip"]');
    const folderInput = form.querySelector('[data-upload-input="folder"]');
    const includeInput = form.querySelector('[data-include-input]');
    const ignoreInput = form.querySelector('[data-ignore-input]');

    const defaultState = {
      mode: 'url',
      format: 'xml',
      includePatterns: '',
      ignorePatterns: '',
      fileSummary: true,
      directoryStructure: true,
      showLineNumbers: false,
      outputParsable: false,
      compress: false,
      removeComments: false,
      removeEmptyLines: false,
    };

    bindModeTabs();
    bindFormatButtons();
    bindUploadZone(zipZone, zipInput, 'zip');
    bindUploadZone(folderZone, folderInput, 'folder');
    bindClearButtons();
    bindDelegatedInteractions();
    bindFieldStateUpdates();
    bindFormSubmit();

    setMode(state.mode);
    updateFormatButtons();
    updateUrlValidation();
    updatePackButtonState();
    updateResetVisibility();
    refreshFileSelectionState(responseRoot);

    function bindModeTabs() {
      for (const tab of modeTabs) {
        tab.addEventListener('click', () => {
          const nextMode = tab.getAttribute('data-mode-tab');
          if (!nextMode || state.loading) {
            return;
          }
          setMode(nextMode);
          updatePackButtonState();
          updateResetVisibility();
        });
      }
    }

    function bindFormatButtons() {
      for (const button of formatButtons) {
        button.addEventListener('click', () => {
          if (state.loading) {
            return;
          }
          const nextFormat = button.getAttribute('data-format-button');
          if (!nextFormat) {
            return;
          }
          formatInput.value = nextFormat;
          updateFormatButtons();
          updateResetVisibility();
        });
      }
    }

    function bindUploadZone(zone, input, kind) {
      if (!(zone instanceof HTMLElement) || !(input instanceof HTMLInputElement)) {
        return;
      }

      zone.addEventListener('click', (event) => {
        if (event.target instanceof HTMLElement && event.target.closest('[data-clear-selection]')) {
          return;
        }
        input.click();
      });

      zone.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          input.click();
        }
      });

      zone.addEventListener('dragover', (event) => {
        event.preventDefault();
        zone.classList.add('is-drag-active');
      });

      zone.addEventListener('dragleave', () => {
        zone.classList.remove('is-drag-active');
      });

      zone.addEventListener('drop', async (event) => {
        event.preventDefault();
        zone.classList.remove('is-drag-active');

        if (kind === 'zip') {
          const file = event.dataTransfer?.files?.[0] ?? null;
          applyZipSelection(file);
        } else {
          const files = await collectDroppedFolderFiles(event);
          applyFolderSelection(files);
        }

        updatePackButtonState();
        updateResetVisibility();
      });

      input.addEventListener('change', async () => {
        if (kind === 'zip') {
          applyZipSelection(input.files?.[0] ?? null);
        } else {
          applyFolderSelection(normalizeFolderInputFiles(input.files));
        }

        updatePackButtonState();
        updateResetVisibility();
      });
    }

    function bindClearButtons() {
      form.addEventListener('click', (event) => {
        const button = event.target instanceof HTMLElement ? event.target.closest('[data-clear-selection]') : null;
        if (!(button instanceof HTMLElement)) {
          return;
        }

        event.preventDefault();
        const kind = button.getAttribute('data-clear-selection');
        if (kind === 'zip') {
          clearZipSelection();
        }
        if (kind === 'folder') {
          clearFolderSelection();
        }

        updatePackButtonState();
        updateResetVisibility();
      });
    }

    function bindDelegatedInteractions() {
      form.addEventListener('click', async (event) => {
        const target = event.target instanceof HTMLElement ? event.target : null;
        if (!target) {
          return;
        }

        const resetButton = target.closest('[data-reset-button]');
        if (resetButton instanceof HTMLElement) {
          event.preventDefault();
          resetForm();
          return;
        }

        const copyButton = target.closest('[data-copy-output]');
        if (copyButton instanceof HTMLButtonElement) {
          event.preventDefault();
          await copyResult(copyButton);
          return;
        }

        const downloadButton = target.closest('[data-download-output]');
        if (downloadButton instanceof HTMLButtonElement) {
          event.preventDefault();
          downloadResult(downloadButton);
          return;
        }

        const resultTab = target.closest('[data-result-tab]');
        if (resultTab instanceof HTMLButtonElement) {
          event.preventDefault();
          switchResultTab(resultTab.getAttribute('data-result-tab') || 'result');
          return;
        }

        const selectAllButton = target.closest('[data-select-all]');
        if (selectAllButton instanceof HTMLButtonElement) {
          event.preventDefault();
          setAllFileSelection(true);
          return;
        }

        const deselectAllButton = target.closest('[data-deselect-all]');
        if (deselectAllButton instanceof HTMLButtonElement) {
          event.preventDefault();
          setAllFileSelection(false);
          return;
        }

        const repackButton = target.closest('[data-repack-selected]');
        if (repackButton instanceof HTMLButtonElement) {
          event.preventDefault();
          await submitForm({
            includePatternsOverride: getSelectedFiles().join(','),
            ignorePatternsOverride: '',
          });
          return;
        }
      });

      form.addEventListener('change', (event) => {
        const target = event.target;
        if (target instanceof HTMLInputElement && target.hasAttribute('data-file-checkbox')) {
          syncRowSelectionState(target);
          refreshFileSelectionState(responseRoot);
          return;
        }

        if (target instanceof HTMLInputElement && target.hasAttribute('data-file-master-toggle')) {
          setAllFileSelection(target.checked);
        }
      });
    }

    function bindFieldStateUpdates() {
      if (urlInput instanceof HTMLInputElement) {
        urlInput.addEventListener('input', () => {
          updateUrlValidation();
          updatePackButtonState();
          updateResetVisibility();
        });
      }

      for (const element of form.querySelectorAll('input[name="includePatterns"], input[name="ignorePatterns"], input[data-option-checkbox]')) {
        element.addEventListener('input', updateResetVisibility);
        element.addEventListener('change', () => {
          updatePackButtonState();
          updateResetVisibility();
        });
      }

      if (includeInput instanceof HTMLInputElement) {
        includeInput.addEventListener('input', updateResetVisibility);
      }

      if (ignoreInput instanceof HTMLInputElement) {
        ignoreInput.addEventListener('input', updateResetVisibility);
      }
    }

    function bindFormSubmit() {
      form.addEventListener('submit', async (event) => {
        event.preventDefault();
        await submitForm();
      });

      packButton.addEventListener('click', (event) => {
        if (!state.loading || event.detail === 0) {
          return;
        }
        event.preventDefault();
        cancelRequest();
      });
    }

    function setMode(mode) {
      state.mode = mode;
      sourceKindInput.value = mode;

      for (const tab of modeTabs) {
        const isActive = tab.getAttribute('data-mode-tab') === mode;
        tab.classList.toggle('is-active', isActive);
        tab.setAttribute('aria-selected', isActive ? 'true' : 'false');
      }

      for (const [panelMode, panel] of modePanels.entries()) {
        const isActive = panelMode === mode;
        panel.classList.toggle('is-active', isActive);
        panel.setAttribute('aria-hidden', isActive ? 'false' : 'true');
      }
    }

    function updateFormatButtons() {
      for (const button of formatButtons) {
        const isActive = button.getAttribute('data-format-button') === formatInput.value;
        button.classList.toggle('is-active', isActive);
        button.setAttribute('aria-pressed', isActive ? 'true' : 'false');
      }
    }

    function updateUrlValidation() {
      if (!(urlInput instanceof HTMLInputElement) || !(urlWarning instanceof HTMLElement)) {
        return;
      }

      const value = urlInput.value.trim();
      const valid = value === '' || isValidRemoteValue(value);
      urlInput.classList.toggle('is-invalid', value !== '' && !valid);
      urlWarning.classList.toggle('is-hidden', value === '' || valid);
    }

    function applyZipSelection(file) {
      state.zipFile = null;
      state.zipUploadId = null;
      state.zipError = '';

      if (!file) {
        renderUploadState('zip');
        return;
      }

      if (!file.name.toLowerCase().endsWith('.zip')) {
        state.zipError = 'Please choose a ZIP file.';
      } else if (file.size > MAX_UPLOAD_BYTES) {
        state.zipError = 'ZIP upload must be 50MB or smaller.';
      } else {
        state.zipFile = file;
      }

      renderUploadState('zip');
    }

    function applyFolderSelection(files) {
      state.folderFiles = [];
      state.folderLabel = '';
      state.folderError = '';

      if (!files || files.length === 0) {
        renderUploadState('folder');
        return;
      }

      const totalBytes = files.reduce((sum, item) => sum + item.file.size, 0);
      if (totalBytes > MAX_UPLOAD_BYTES) {
        state.folderError = 'Folder upload must be 50MB or smaller.';
      } else {
        state.folderFiles = files;
        state.folderLabel = deriveFolderLabel(files);
      }

      renderUploadState('folder');
    }

    function clearZipSelection() {
      state.zipFile = null;
      state.zipUploadId = null;
      state.zipError = '';
      const input = form.querySelector('[data-upload-input="zip"]');
      if (input instanceof HTMLInputElement) {
        input.value = '';
      }
      renderUploadState('zip');
    }

    function clearFolderSelection() {
      state.folderFiles = [];
      state.folderLabel = '';
      state.folderError = '';
      const input = form.querySelector('[data-upload-input="folder"]');
      if (input instanceof HTMLInputElement) {
        input.value = '';
      }
      renderUploadState('folder');
    }

    function renderUploadState(kind) {
      const zone = form.querySelector(`[data-upload-zone="${kind}"]`);
      if (!(zone instanceof HTMLElement)) {
        return;
      }

      const placeholder = zone.querySelector('[data-upload-placeholder]');
      const selection = zone.querySelector('[data-upload-selection]');
      const selectionName = zone.querySelector('[data-upload-selection-name]');
      const error = zone.querySelector('[data-upload-error]');

      const selectedLabel = kind === 'zip' ? state.zipFile?.name ?? '' : state.folderLabel;
      const errorMessage = kind === 'zip' ? state.zipError : state.folderError;

      zone.classList.toggle('has-error', errorMessage !== '');

      if (placeholder instanceof HTMLElement) {
        placeholder.classList.toggle('is-hidden', selectedLabel !== '' || errorMessage !== '');
      }
      if (selection instanceof HTMLElement) {
        selection.classList.toggle('is-hidden', selectedLabel === '' || errorMessage !== '');
      }
      if (selectionName instanceof HTMLElement) {
        selectionName.textContent = selectedLabel;
      }
      if (error instanceof HTMLElement) {
        error.textContent = errorMessage;
        error.classList.toggle('is-hidden', errorMessage === '');
      }
    }

    function updatePackButtonState() {
      const valid = isCurrentModeValid();
      packButton.disabled = !state.loading && !valid;
    }

    function updateResetVisibility() {
      if (!(resetTooltip instanceof HTMLElement)) {
        return;
      }

      const show = hasNonDefaultState();
      resetTooltip.classList.toggle('is-hidden', !show);
    }

    function hasNonDefaultState() {
      return state.mode !== defaultState.mode
        || (urlInput instanceof HTMLInputElement && urlInput.value.trim() !== '')
        || formatInput.value !== defaultState.format
        || (includeInput instanceof HTMLInputElement && includeInput.value.trim() !== defaultState.includePatterns)
        || (ignoreInput instanceof HTMLInputElement && ignoreInput.value.trim() !== defaultState.ignorePatterns)
        || checkboxChecked('fileSummary') !== defaultState.fileSummary
        || checkboxChecked('directoryStructure') !== defaultState.directoryStructure
        || checkboxChecked('showLineNumbers') !== defaultState.showLineNumbers
        || checkboxChecked('outputParsable') !== defaultState.outputParsable
        || checkboxChecked('compress') !== defaultState.compress
        || checkboxChecked('removeComments') !== defaultState.removeComments
        || checkboxChecked('removeEmptyLines') !== defaultState.removeEmptyLines
        || !!state.zipFile
        || state.folderFiles.length > 0;
    }

    function checkboxChecked(name) {
      const input = form.querySelector(`input[name="${name}"]`);
      return input instanceof HTMLInputElement ? input.checked : false;
    }

    function isCurrentModeValid() {
      if (state.mode === 'url') {
        const value = urlInput instanceof HTMLInputElement ? urlInput.value.trim() : '';
        return value !== '' && isValidRemoteValue(value);
      }

      if (state.mode === 'zip') {
        return !!state.zipFile && state.zipError === '';
      }

      if (state.mode === 'folder') {
        return state.folderFiles.length > 0 && state.folderError === '';
      }

      return false;
    }

    async function submitForm(overrides = {}) {
      if (state.loading) {
        return;
      }

      if (!isCurrentModeValid()) {
        updatePackButtonState();
        return;
      }

      setLoading(true);
      responseRoot.innerHTML = loadingTemplate instanceof HTMLTemplateElement ? loadingTemplate.innerHTML : '';
      responseRoot.scrollIntoView({ behavior: 'smooth', block: 'start' });

      const controller = new AbortController();
      state.controller = controller;

      try {
        const formData = new FormData(form);
        formData.set('sourceKind', state.mode);

        if (overrides.includePatternsOverride !== undefined) {
          formData.set('includePatterns', overrides.includePatternsOverride);
        }
        if (overrides.ignorePatternsOverride !== undefined) {
          formData.set('ignorePatterns', overrides.ignorePatternsOverride);
        }

        formData.delete('uploadId');
        formData.delete('file');
        formData.delete('folderManifest');
        formData.delete('folderFiles');

        if (state.mode === 'zip') {
          await appendZipPayload(formData, controller.signal);
        }

        if (state.mode === 'folder') {
          appendFolderPayload(formData);
        }

        const response = await fetch('/pack', {
          method: 'POST',
          body: formData,
          headers: {
            'x-repoxide-response-fragment': '1',
          },
          signal: controller.signal,
        });

        const html = await response.text();
        responseRoot.innerHTML = html;
        refreshFileSelectionState(responseRoot);
        responseRoot.scrollIntoView({ behavior: 'smooth', block: 'start' });
      } catch (error) {
        if (error instanceof DOMException && error.name === 'AbortError') {
          responseRoot.innerHTML = '';
        } else {
          responseRoot.innerHTML = `<div class="result-viewer"><div class="result-error"><div class="result-error__icon">!</div><h2 class="result-error__title">Request failed</h2><p class="result-error__message">${escapeHtml(String(error))}</p></div></div>`;
        }
      } finally {
        state.controller = null;
        setLoading(false);
        updatePackButtonState();
        updateResetVisibility();
      }
    }

    function setLoading(loading) {
      state.loading = loading;
      packButton.classList.toggle('is-loading', loading);
      if (packButtonLabel instanceof HTMLElement) {
        packButtonLabel.textContent = loading ? 'Processing...' : 'Pack';
      }

      const repackLabel = responseRoot.querySelector('[data-repack-label]');
      if (repackLabel instanceof HTMLElement) {
        repackLabel.textContent = loading
          ? repackLabel.getAttribute('data-loading') || 'Re-packing...'
          : repackLabel.getAttribute('data-default') || 'Re-pack Selected';
      }
    }

    function cancelRequest() {
      state.controller?.abort();
    }

    function resetForm() {
      cancelRequest();
      form.reset();
      state.zipUploadId = null;
      setMode(defaultState.mode);
      formatInput.value = defaultState.format;
      updateFormatButtons();
      clearZipSelection();
      clearFolderSelection();
      if (urlInput instanceof HTMLInputElement) {
        urlInput.value = '';
      }
      if (includeInput instanceof HTMLInputElement) {
        includeInput.value = defaultState.includePatterns;
      }
      if (ignoreInput instanceof HTMLInputElement) {
        ignoreInput.value = defaultState.ignorePatterns;
      }
      setCheckbox('fileSummary', defaultState.fileSummary);
      setCheckbox('directoryStructure', defaultState.directoryStructure);
      setCheckbox('showLineNumbers', defaultState.showLineNumbers);
      setCheckbox('outputParsable', defaultState.outputParsable);
      setCheckbox('compress', defaultState.compress);
      setCheckbox('removeComments', defaultState.removeComments);
      setCheckbox('removeEmptyLines', defaultState.removeEmptyLines);
      responseRoot.innerHTML = '';
      updateUrlValidation();
      updatePackButtonState();
      updateResetVisibility();
    }

    function setCheckbox(name, checked) {
      const input = form.querySelector(`input[name="${name}"]`);
      if (input instanceof HTMLInputElement) {
        input.checked = checked;
      }
    }

    async function appendZipPayload(formData, signal) {
      if (!state.zipFile) {
        return;
      }

      if (state.zipFile.size <= CHUNK_THRESHOLD_BYTES) {
        formData.append('file', state.zipFile, state.zipFile.name);
        return;
      }

      if (!state.zipUploadId) {
        state.zipUploadId = await uploadZipInChunks(state.zipFile, signal);
      }

      formData.append('uploadId', state.zipUploadId);
    }

    function appendFolderPayload(formData) {
      const paths = state.folderFiles.map((item) => item.relativePath);
      formData.append('folderManifest', JSON.stringify({ paths }));
      for (const item of state.folderFiles) {
        formData.append('folderFiles', item.file, item.relativePath);
      }
    }

    async function uploadZipInChunks(file, signal) {
      const totalChunks = Math.ceil(file.size / CHUNK_SIZE_BYTES);
      const initResponse = await fetch('/api/upload/init', {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          fileName: file.name,
          fileSize: file.size,
          totalChunks,
        }),
        signal,
      });

      if (!initResponse.ok) {
        throw new Error(await readResponseError(initResponse, 'Failed to initialize ZIP upload.'));
      }

      const initPayload = await initResponse.json();
      const uploadId = initPayload.uploadId;

      if (!uploadId) {
        throw new Error('Upload initialization did not return an uploadId.');
      }

      for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex += 1) {
        const start = chunkIndex * CHUNK_SIZE_BYTES;
        const end = Math.min(file.size, start + CHUNK_SIZE_BYTES);
        const chunk = file.slice(start, end);

        const chunkResponse = await fetch(`/api/upload/chunk?uploadId=${encodeURIComponent(uploadId)}&chunkIndex=${chunkIndex}`, {
          method: 'POST',
          headers: {
            'content-type': 'application/octet-stream',
          },
          body: chunk,
          signal,
        });

        if (!chunkResponse.ok) {
          throw new Error(await readResponseError(chunkResponse, 'Failed to upload ZIP chunk.'));
        }
      }

      return uploadId;
    }

    async function readResponseError(response, fallbackMessage) {
      const contentType = response.headers.get('content-type') || '';

      if (contentType.includes('application/json')) {
        try {
          const payload = await response.json();
          if (payload && typeof payload.error === 'string' && payload.error.trim() !== '') {
            return payload.error;
          }
        } catch {
          return fallbackMessage;
        }
      }

      const text = await response.text();
      return text.trim() || fallbackMessage;
    }

    async function copyResult(button) {
      const output = responseRoot.querySelector('[data-result-output]');
      if (!(output instanceof HTMLElement) || !navigator.clipboard) {
        return;
      }

      await navigator.clipboard.writeText(output.textContent || '');
      button.classList.add('is-success');
      const label = button.querySelector('[data-copy-label]');
      if (label instanceof HTMLElement) {
        label.textContent = label.getAttribute('data-success') || 'Copied!';
      }
      window.setTimeout(() => {
        button.classList.remove('is-success');
        if (label instanceof HTMLElement) {
          label.textContent = label.getAttribute('data-default') || 'Copy';
        }
      }, 2000);
    }

    function downloadResult(button) {
      const output = responseRoot.querySelector('[data-result-output]');
      if (!(output instanceof HTMLElement)) {
        return;
      }

      const content = output.textContent || '';
      const blob = new Blob([content], { type: button.getAttribute('data-download-type') || 'text/plain;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = button.getAttribute('data-download-name') || 'repoxide-output.txt';
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
    }

    function switchResultTab(tabName) {
      for (const button of responseRoot.querySelectorAll('[data-result-tab]')) {
        const active = button.getAttribute('data-result-tab') === tabName;
        button.classList.toggle('is-active', active);
        button.setAttribute('aria-selected', active ? 'true' : 'false');
      }

      for (const panel of responseRoot.querySelectorAll('[data-result-panel]')) {
        panel.classList.toggle('is-active', panel.getAttribute('data-result-panel') === tabName);
      }
    }

    function refreshFileSelectionState(root) {
      const selectionRoot = root.querySelector('[data-file-selection]');
      if (!(selectionRoot instanceof HTMLElement)) {
        return;
      }

      const checkboxes = Array.from(selectionRoot.querySelectorAll('[data-file-checkbox]')).filter((input) => input instanceof HTMLInputElement);
      const selectedCheckboxes = checkboxes.filter((input) => input.checked);
      const totalTokens = checkboxes.reduce((sum, input) => sum + Number(input.getAttribute('data-token-count') || 0), 0);
      const selectedTokens = selectedCheckboxes.reduce((sum, input) => sum + Number(input.getAttribute('data-token-count') || 0), 0);
      const selectedCount = selectedCheckboxes.length;
      const totalCount = checkboxes.length;

      const selectedCountNode = selectionRoot.querySelector('[data-selected-count]');
      const totalCountNode = selectionRoot.querySelector('[data-total-count-label]');
      const selectedTokensNode = selectionRoot.querySelector('[data-selected-tokens]');
      const selectedPercentNode = selectionRoot.querySelector('[data-selected-percent]');
      const masterToggle = selectionRoot.querySelector('[data-file-master-toggle]');
      const repackButton = selectionRoot.querySelector('[data-repack-selected]');

      if (selectedCountNode instanceof HTMLElement) {
        selectedCountNode.textContent = String(selectedCount);
      }
      if (totalCountNode instanceof HTMLElement) {
        totalCountNode.textContent = String(totalCount);
      }
      if (selectedTokensNode instanceof HTMLElement) {
        selectedTokensNode.textContent = selectedTokens.toLocaleString();
      }
      if (selectedPercentNode instanceof HTMLElement) {
        selectedPercentNode.textContent = `${totalTokens === 0 ? '0.0' : ((selectedTokens / totalTokens) * 100).toFixed(1)}%`;
      }
      if (masterToggle instanceof HTMLInputElement) {
        masterToggle.checked = totalCount > 0 && selectedCount === totalCount;
        masterToggle.indeterminate = selectedCount > 0 && selectedCount < totalCount;
      }
      if (repackButton instanceof HTMLButtonElement) {
        repackButton.disabled = selectedCount === 0 || state.loading;
      }

      for (const checkbox of checkboxes) {
        syncRowSelectionState(checkbox);
      }
    }

    function setAllFileSelection(checked) {
      for (const input of responseRoot.querySelectorAll('[data-file-checkbox]')) {
        if (input instanceof HTMLInputElement) {
          input.checked = checked;
          syncRowSelectionState(input);
        }
      }
      refreshFileSelectionState(responseRoot);
    }

    function getSelectedFiles() {
      return Array.from(responseRoot.querySelectorAll('[data-file-checkbox]'))
        .filter((input) => input instanceof HTMLInputElement && input.checked)
        .map((input) => input.getAttribute('data-file-path') || '')
        .filter(Boolean);
    }

    function syncRowSelectionState(input) {
      const row = input.closest('[data-file-row]');
      if (row instanceof HTMLElement) {
        row.classList.toggle('file-row-selected', input.checked);
      }
    }

    function normalizeFolderInputFiles(fileList) {
      if (!fileList) {
        return [];
      }

      return Array.from(fileList).map((file) => ({
        file,
        relativePath: file.webkitRelativePath || file.name,
      }));
    }

    async function collectDroppedFolderFiles(event) {
      const items = Array.from(event.dataTransfer?.items || []);
      const entries = items
        .map((item) => (typeof item.webkitGetAsEntry === 'function' ? item.webkitGetAsEntry() : null))
        .filter(Boolean);

      if (entries.length === 0) {
        return normalizeFolderInputFiles(event.dataTransfer?.files || null);
      }

      const files = [];
      for (const entry of entries) {
        await walkFileTree(entry, '', files);
      }
      return files;
    }

    async function walkFileTree(entry, parentPath, files) {
      if (!entry) {
        return;
      }

      if (entry.isFile) {
        const file = await new Promise((resolve, reject) => entry.file(resolve, reject));
        const relativePath = parentPath ? `${parentPath}/${file.name}` : file.name;
        files.push({ file, relativePath });
        return;
      }

      if (!entry.isDirectory) {
        return;
      }

      const directoryPath = parentPath ? `${parentPath}/${entry.name}` : entry.name;
      const reader = entry.createReader();
      const children = await readAllEntries(reader);
      for (const child of children) {
        await walkFileTree(child, directoryPath, files);
      }
    }

    async function readAllEntries(reader) {
      const entries = [];
      while (true) {
        const batch = await new Promise((resolve, reject) => reader.readEntries(resolve, reject));
        if (!batch.length) {
          break;
        }
        entries.push(...batch);
      }
      return entries;
    }

    function deriveFolderLabel(files) {
      const firstPath = files[0]?.relativePath || files[0]?.file?.name || '';
      const firstSegment = firstPath.split('/')[0];
      return firstSegment || firstPath;
    }

    function isValidRemoteValue(value) {
      const trimmed = value.trim();
      if (!trimmed) {
        return false;
      }

      if (/^[^/\s]+\/[^/\s]+$/.test(trimmed)) {
        return true;
      }

      try {
        new URL(trimmed);
        return true;
      } catch {
        return false;
      }
    }

    function escapeHtml(value) {
      return value
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
    }
  });

  function initTheme() {
    const root = document.documentElement;
    const themeButtons = Array.from(document.querySelectorAll('[data-theme-option]')).filter((button) => button instanceof HTMLButtonElement);
    let preference = normalizeThemePreference(root.dataset.themePreference || readStoredThemePreference());

    applyThemePreference(root, preference);
    syncThemeButtons(themeButtons, preference);

    for (const button of themeButtons) {
      button.addEventListener('click', () => {
        const nextPreference = normalizeThemePreference(button.getAttribute('data-theme-option'));
        if (nextPreference === preference) {
          return;
        }

        preference = nextPreference;
        writeStoredThemePreference(preference);
        applyThemePreference(root, preference);
        syncThemeButtons(themeButtons, preference);
      });
    }

    const mediaQuery = typeof window.matchMedia === 'function' ? window.matchMedia(THEME_MEDIA_QUERY) : null;
    const refreshSystemTheme = () => {
      if (preference !== 'system') {
        return;
      }

      applyThemePreference(root, preference);
      syncThemeButtons(themeButtons, preference);
    };

    if (mediaQuery) {
      if (typeof mediaQuery.addEventListener === 'function') {
        mediaQuery.addEventListener('change', refreshSystemTheme);
      } else if (typeof mediaQuery.addListener === 'function') {
        mediaQuery.addListener(refreshSystemTheme);
      }
    }

    window.addEventListener('storage', (event) => {
      if (event.key !== THEME_STORAGE_KEY) {
        return;
      }

      preference = normalizeThemePreference(event.newValue);
      applyThemePreference(root, preference);
      syncThemeButtons(themeButtons, preference);
    });
  }

  function syncThemeButtons(buttons, activePreference) {
    for (const button of buttons) {
      const isActive = button.getAttribute('data-theme-option') === activePreference;
      button.classList.toggle('is-active', isActive);
      button.setAttribute('aria-pressed', isActive ? 'true' : 'false');
    }
  }

  function applyThemePreference(root, preference) {
    const normalizedPreference = normalizeThemePreference(preference);
    const effectiveTheme = resolveThemePreference(normalizedPreference);

    root.dataset.themePreference = normalizedPreference;
    root.dataset.theme = effectiveTheme;
    root.style.colorScheme = effectiveTheme;
  }

  function resolveThemePreference(preference) {
    if (preference === 'light' || preference === 'dark') {
      return preference;
    }

    return prefersDarkMode() ? 'dark' : 'light';
  }

  function prefersDarkMode() {
    return typeof window.matchMedia === 'function' && window.matchMedia(THEME_MEDIA_QUERY).matches;
  }

  function readStoredThemePreference() {
    try {
      return window.localStorage.getItem(THEME_STORAGE_KEY);
    } catch {
      return 'system';
    }
  }

  function writeStoredThemePreference(preference) {
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, preference);
    } catch {}
  }

  function normalizeThemePreference(value) {
    return value === 'light' || value === 'dark' || value === 'system' ? value : 'system';
  }
})();
