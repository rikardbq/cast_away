pub mod application;

pub const HOST: &str = "127.0.0.1";
pub const PORT: &str = "8000";
pub const ROOT_DIR: &str = "assets";
// pub const WEBVIEW_LOADED_SCRIPT: &str = r#"
//     window.ipc.postMessage("Webview loaded!");
// "#;
pub const IPC_HANDLER_INIT_SCRIPT: &str = r#"
if (!window.ipc_handler) {
    window.ipc_handler = {
        currRequestId: 0,
        pendingRequests: new Map(),
        responseHandler: function (response) {
            const { id, result } = response;
            if (window.ipc_handler.pendingRequests.has(id)) {
                window.ipc_handler.pendingRequests.get(id)(result);
                window.ipc_handler.pendingRequests.delete(id);
            }
        }
    };
}
"#;
pub const KIOSK_SCRIPT: &str = r#"
window.addEventListener("selectstart", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

window.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

["dragstart", "drop", "dragover"].forEach((eventName) => {
    window.addEventListener(eventName, (e) => {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }, true);
});

// Disable middle-click / aux click (open new tab behavior in some engines)
window.addEventListener("auxclick", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

// Disable mouse down on middle click
window.addEventListener("mousedown", (e) => {
    if (e.button === 1) {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }
}, true);

window.addEventListener("keydown", (e) => {
    const key = e.key.toLowerCase();

    // Disable refresh (F5 / Ctrl+R)
    if (key === "f5" || (e.ctrlKey && key === "r")) {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+Shift+I (DevTools)
    if (e.ctrlKey && e.shiftKey && key === "i") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable F12 (DevTools)
    if (key === "f12") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+Shift+J (DevTools console in Chromium)
    if (e.ctrlKey && e.shiftKey && key === "j") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+Shift+C (Inspect element)
    if (e.ctrlKey && e.shiftKey && key === "c") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+U (view source)
    if (e.ctrlKey && key === "u") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+S (save page)
    if (e.ctrlKey && key === "s") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+P (print)
    if (e.ctrlKey && key === "p") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+O (open file)
    if (e.ctrlKey && key === "o") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+N (new window)
    if (e.ctrlKey && key === "n") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Ctrl+W (close tab/window)
    if (e.ctrlKey && key === "w") {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Alt+Left/Right / Backspace navigation
    if ((e.altKey && (key === "arrowleft" || key === "left")) || e.altKey && (key === "arrowright" || key === "right")) {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    if (key === "backspace") {
        // prevent browser navigating back when focus isn't in an input
        const el = document.activeElement;
        const isInput = el && (
            el.tagName === "INPUT" ||
            el.tagName === "TEXTAREA" ||
            el.isContentEditable
        );

        if (!isInput) {
            e.preventDefault();
            e.stopPropagation();
            return false;
        }
    }

    // Disable zoom (Ctrl + +/-/0)
    if (e.ctrlKey && ["+", "-", "=", "0"].includes(key)) {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }

    // Disable Escape (optional, can break UI dialogs)
    // if (key === "escape") {
    //   e.preventDefault();
    //   e.stopPropagation();
    //   return false;
    // }

}, true);

// Disable wheel zoom (Ctrl+Scroll)
window.addEventListener("wheel", (e) => {
    if (e.ctrlKey) {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }
}, { passive: false, capture: true });

// Disable pinch zoom gesture
window.addEventListener("gesturestart", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

window.addEventListener("gesturechange", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

window.addEventListener("gestureend", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

// Optional: disable double click zoom (some webviews)
window.addEventListener("dblclick", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
}, true);

// Optional: disable touch callout on iOS-ish environments
const style = document.createElement("style");
style.innerHTML = `
    * {
        -webkit-touch-callout: none !important;
        -webkit-user-select: none !important;
        user-select: none !important;
    }
    html, body {
        overscroll-behavior: none !important;
    }
`;
document.head.appendChild(style);
"#;
