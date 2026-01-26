// qmpo background service worker

// Listen for messages from content script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'openDirectory') {
    const url = message.url;

    // Create a new tab with the directory:// URL
    chrome.tabs.create({ url: url, active: false }, (tab) => {
      // Close the tab immediately after it triggers the protocol handler
      setTimeout(() => {
        if (tab && tab.id) {
          chrome.tabs.remove(tab.id).catch(() => {});
        }
      }, 500);
    });

    sendResponse({ success: true });
  }
  return true;
});
