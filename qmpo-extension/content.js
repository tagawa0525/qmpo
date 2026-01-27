// qmpo - Convert file:// links to directory:// scheme

(function() {
  'use strict';

  // Default settings
  const DEFAULT_SETTINGS = {
    enabled: true,
    showIndicator: true,
    allowedDomains: [], // Empty = all domains allowed
    blockedDomains: []
  };

  let settings = DEFAULT_SETTINGS;

  // Load settings from storage
  function loadSettings() {
    return new Promise((resolve) => {
      if (chrome.storage && chrome.storage.sync) {
        chrome.storage.sync.get(DEFAULT_SETTINGS, (result) => {
          settings = { ...DEFAULT_SETTINGS, ...result };
          resolve(settings);
        });
      } else {
        resolve(settings);
      }
    });
  }

  // Check if current domain is allowed
  function isDomainAllowed() {
    const hostname = window.location.hostname;

    // Check blocked domains first
    if (settings.blockedDomains.length > 0) {
      for (const domain of settings.blockedDomains) {
        if (hostname === domain || hostname.endsWith('.' + domain)) {
          return false;
        }
      }
    }

    // If allowedDomains is empty, allow all (except blocked)
    if (settings.allowedDomains.length === 0) {
      return true;
    }

    // Check allowed domains
    for (const domain of settings.allowedDomains) {
      if (hostname === domain || hostname.endsWith('.' + domain)) {
        return true;
      }
    }

    return false;
  }

  // Convert file:// URL to directory:// URL
  function convertFileToDirectory(url) {
    if (!url || !url.startsWith('file://')) {
      return null;
    }
    return url.replace(/^file:\/\//, 'directory://');
  }

  // Check if element is a file:// link
  function isFileLink(element) {
    if (element.tagName !== 'A') return false;
    const href = element.getAttribute('href');
    return href && href.startsWith('file://');
  }

  // Add visual indicator to converted links
  function addIndicator(link) {
    if (!settings.showIndicator) return;
    if (link.dataset.qmpoConverted) return;

    link.dataset.qmpoConverted = 'true';
    link.style.position = 'relative';

    // Add a small folder icon indicator
    const indicator = document.createElement('span');
    indicator.className = 'qmpo-indicator';
    indicator.textContent = ' \uD83D\uDCC2';
    indicator.title = 'Opens in file manager (qmpo)';
    indicator.style.cssText = 'font-size: 0.8em; opacity: 0.7;';
    link.appendChild(indicator);
  }

  // Process a single link
  function processLink(link) {
    if (!isFileLink(link)) return;
    addIndicator(link);
  }

  // Process all file:// links on the page
  function processAllLinks() {
    const links = document.querySelectorAll('a[href^="file://"]');
    links.forEach(processLink);
  }

  // Open directory:// URL via background script
  function openDirectoryUrl(url) {
    chrome.runtime.sendMessage({ action: 'openDirectory', url: url }, (response) => {
      if (chrome.runtime.lastError) {
        console.error('qmpo: Failed to open directory:', chrome.runtime.lastError);
      }
    });
  }

  // Handle click events on file:// links
  function handleClick(event) {
    if (!settings.enabled) return;
    if (!isDomainAllowed()) return;

    const link = event.target.closest('a[href^="file://"]');
    if (!link) return;

    const href = link.getAttribute('href');
    const directoryUrl = convertFileToDirectory(href);

    if (directoryUrl) {
      event.preventDefault();
      event.stopPropagation();

      // Open directory:// URL via hidden iframe
      openDirectoryUrl(directoryUrl);
    }
  }

  // Observe DOM changes for dynamically added links
  function observeDOM() {
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (node.nodeType !== Node.ELEMENT_NODE) continue;

          // Check if the added node is a file:// link
          if (node.tagName === 'A' && isFileLink(node)) {
            processLink(node);
          }

          // Check descendants
          const links = node.querySelectorAll?.('a[href^="file://"]');
          if (links) {
            links.forEach(processLink);
          }
        }
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }

  // Initialize
  async function init() {
    await loadSettings();

    if (!settings.enabled) return;
    if (!isDomainAllowed()) return;

    // Process existing links
    processAllLinks();

    // Listen for clicks
    document.addEventListener('click', handleClick, true);

    // Observe for new links
    observeDOM();

    // Listen for settings changes
    if (chrome.storage && chrome.storage.onChanged) {
      chrome.storage.onChanged.addListener((changes, namespace) => {
        if (namespace === 'sync') {
          for (const key in changes) {
            settings[key] = changes[key].newValue;
          }
        }
      });
    }
  }

  // Start when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
