// qmpo popup script

document.addEventListener('DOMContentLoaded', async () => {
  const enabledCheckbox = document.getElementById('enabled');
  const showIndicatorCheckbox = document.getElementById('showIndicator');
  const statusDiv = document.getElementById('status');
  const openOptionsLink = document.getElementById('openOptions');

  // Load settings
  const settings = await chrome.storage.sync.get({
    enabled: true,
    showIndicator: true
  });

  enabledCheckbox.checked = settings.enabled;
  showIndicatorCheckbox.checked = settings.showIndicator;

  updateStatus(settings.enabled);

  // Save settings on change
  enabledCheckbox.addEventListener('change', async () => {
    const enabled = enabledCheckbox.checked;
    await chrome.storage.sync.set({ enabled });
    updateStatus(enabled);
  });

  showIndicatorCheckbox.addEventListener('change', async () => {
    await chrome.storage.sync.set({ showIndicator: showIndicatorCheckbox.checked });
  });

  // Open options page
  openOptionsLink.addEventListener('click', (e) => {
    e.preventDefault();
    chrome.runtime.openOptionsPage();
  });

  function updateStatus(enabled) {
    if (enabled) {
      statusDiv.className = 'status active';
      statusDiv.textContent = 'file:// links will open in file manager';
    } else {
      statusDiv.className = 'status inactive';
      statusDiv.textContent = 'Conversion disabled';
    }
  }
});
