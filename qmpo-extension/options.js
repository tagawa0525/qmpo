// qmpo options page script

const DEFAULT_SETTINGS = {
  enabled: true,
  showIndicator: true,
  allowedDomains: [],
  blockedDomains: []
};

document.addEventListener('DOMContentLoaded', async () => {
  const enabledCheckbox = document.getElementById('enabled');
  const showIndicatorCheckbox = document.getElementById('showIndicator');
  const allowedDomainsTextarea = document.getElementById('allowedDomains');
  const blockedDomainsTextarea = document.getElementById('blockedDomains');
  const saveBtn = document.getElementById('save');
  const resetBtn = document.getElementById('reset');
  const toast = document.getElementById('toast');

  // Load settings
  const settings = await chrome.storage.sync.get(DEFAULT_SETTINGS);

  enabledCheckbox.checked = settings.enabled;
  showIndicatorCheckbox.checked = settings.showIndicator;
  allowedDomainsTextarea.value = settings.allowedDomains.join('\n');
  blockedDomainsTextarea.value = settings.blockedDomains.join('\n');

  // Parse domain list from textarea
  function parseDomains(text) {
    return text
      .split('\n')
      .map(line => line.trim().toLowerCase())
      .filter(line => line.length > 0 && !line.startsWith('#'));
  }

  // Show toast notification
  function showToast(message) {
    toast.textContent = message;
    toast.classList.add('show');
    setTimeout(() => {
      toast.classList.remove('show');
    }, 2000);
  }

  // Save settings
  saveBtn.addEventListener('click', async () => {
    const newSettings = {
      enabled: enabledCheckbox.checked,
      showIndicator: showIndicatorCheckbox.checked,
      allowedDomains: parseDomains(allowedDomainsTextarea.value),
      blockedDomains: parseDomains(blockedDomainsTextarea.value)
    };

    await chrome.storage.sync.set(newSettings);
    showToast('Settings saved');
  });

  // Reset to defaults
  resetBtn.addEventListener('click', async () => {
    if (confirm('Reset all settings to default?')) {
      await chrome.storage.sync.set(DEFAULT_SETTINGS);

      enabledCheckbox.checked = DEFAULT_SETTINGS.enabled;
      showIndicatorCheckbox.checked = DEFAULT_SETTINGS.showIndicator;
      allowedDomainsTextarea.value = '';
      blockedDomainsTextarea.value = '';

      showToast('Settings reset');
    }
  });
});
