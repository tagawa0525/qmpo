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

  // Toast notification for errors
  function showToast(message, isError = false) {
    // Remove existing toast
    const existing = document.getElementById('qmpo-toast');
    if (existing) existing.remove();

    const toast = document.createElement('div');
    toast.id = 'qmpo-toast';
    toast.textContent = message;
    toast.style.cssText = `
      position: fixed;
      bottom: 20px;
      right: 20px;
      padding: 12px 20px;
      background: ${isError ? '#dc3545' : '#28a745'};
      color: white;
      border-radius: 6px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      font-size: 14px;
      z-index: 999999;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
      opacity: 0;
      transition: opacity 0.3s ease;
    `;
    document.body.appendChild(toast);

    // Fade in
    requestAnimationFrame(() => {
      toast.style.opacity = '1';
    });

    // Auto remove after 4 seconds
    setTimeout(() => {
      toast.style.opacity = '0';
      setTimeout(() => toast.remove(), 300);
    }, 4000);
  }

  // Load settings from storage
  function loadSettings() {
    return new Promise((resolve) => {
      if (!chrome.storage || !chrome.storage.sync) {
        console.warn('qmpo: chrome.storage.sync not available, using defaults');
        resolve(settings);
        return;
      }

      try {
        chrome.storage.sync.get(DEFAULT_SETTINGS, (result) => {
          if (chrome.runtime.lastError) {
            console.warn('qmpo: Failed to load settings:', chrome.runtime.lastError.message);
            resolve(settings);
            return;
          }
          settings = { ...DEFAULT_SETTINGS, ...result };
          resolve(settings);
        });
      } catch (e) {
        console.warn('qmpo: Error loading settings:', e.message);
        resolve(settings);
      }
    });
  }

  // Check if hostname matches a domain (exact match or subdomain)
  function matchesDomain(hostname, domain) {
    return hostname === domain || hostname.endsWith('.' + domain);
  }

  // Check if current domain is allowed
  function isDomainAllowed() {
    const hostname = window.location.hostname;

    // Check blocked domains first
    if (settings.blockedDomains.some(domain => matchesDomain(hostname, domain))) {
      return false;
    }

    // If allowedDomains is empty, allow all (except blocked)
    if (settings.allowedDomains.length === 0) {
      return true;
    }

    // Check allowed domains
    return settings.allowedDomains.some(domain => matchesDomain(hostname, domain));
  }

  // Convert file:// URL to directory:// URL
  function convertFileToDirectory(url) {
    if (!url || !url.startsWith('file://')) {
      return null;
    }

    let directoryUrl = url.replace(/^file:\/\//, 'directory://');

    // Fix Windows drive letter without colon
    // Some browsers convert "C:/" to "C/" (removing the colon)
    // Pattern: directory://X/ where X is a single letter -> directory:///X:/
    // Also handles: directory:///X/ -> directory:///X:/
    directoryUrl = directoryUrl.replace(
      /^directory:\/\/\/?([A-Za-z])\//,
      'directory:///$1:/'
    );

    return directoryUrl;
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
    chrome.runtime.sendMessage({ action: 'openDirectory', url }, () => {
      if (chrome.runtime.lastError) {
        console.error('qmpo: Failed to open directory:', chrome.runtime.lastError);
        showToast('Failed to open directory. Is qmpo installed?', true);
      } else if (response && response.error) {
        showToast(response.error, true);
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
          if (isFileLink(node)) {
            processLink(node);
          }

          // Check descendants
          node.querySelectorAll?.('a[href^="file://"]')?.forEach(processLink);
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
            if (Object.prototype.hasOwnProperty.call(DEFAULT_SETTINGS, key)) {
              settings[key] = changes[key].newValue;
            }
          }
        }
      });
    }
  }

  // Wrap init to catch unexpected errors
  async function safeInit() {
    try {
      await init();
    } catch (e) {
      console.error('qmpo: Initialization failed:', e.message);
    }
  }

  // Start when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', safeInit);
  } else {
    safeInit();
  }
})();
