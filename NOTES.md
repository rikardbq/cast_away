### Notes to self (RUST)

- frontend is a react + vite combo 
- http-server based on https://github.com/diegorodrigo90/rust-http-server
- use (rust wry + winit) to connect a webview to the served content from http-server
    - build out the http-server to better handle multiple types of requests
    - separate static files serve and regular requests
- UX will probably be some tailwind'
- Applications can come in the form of on-device apps and websites
    - Shortcuts may come in various forms
        - Actual windows shortcuts
        - Some homebrewed format to support website/local app linkage
            - Need to add some way to open dialog box to manually find apps on the system so that the homebrewed format could work seamlessly and without hand written links
- If I feel like this even becomes useful I will add some simple installer to streamline the installation and usage

### Ideas for the future

- specify component tree to render from to better and more explicitly set navigable items to make gamepad work smoother


### IPC CALL STUFF
```rust

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct IpcRequest {
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize)]
struct IpcResponse {
    id: u64,
    result: serde_json::Value,
}

#[derive(Debug)]
enum UserEvents {
    ExecEval(String),
}

let event_loop: EventLoop<UserEvents> = EventLoop::with_user_event();
let proxy = event_loop.create_proxy();

let webview = WebViewBuilder::new(window)?
    .with_ipc_handler(move |_window, req| {
        let msg = req.body().to_string();

        // Send message into winit event loop
        proxy
            .send_event(UserEvents::ExecEval(msg))
            .expect("Failed to send event");
    })
    .build()?;

event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::UserEvent(UserEvents::ExecEval(msg)) => {
          let req: IpcRequest = serde_json::from_str(&msg).unwrap();

          let result = match req.method.as_str() {
              "list_files" => {
                  let files: Vec<String> = std::fs::read_dir(".")
                      .unwrap()
                      .filter_map(|e| e.ok())
                      .map(|e| e.file_name().to_string_lossy().to_string())
                      .collect();

                  serde_json::to_value(files).unwrap()
              }
              _ => serde_json::json!({"error": "unknown method"})
          };

          let response = IpcResponse {
              id: req.id,
              result,
          };

          let json = serde_json::to_string(&response).unwrap();

          webview.evaluate_script(&format!(
              "window.ipcResponseHandler({});",
              json
          )).unwrap();
      },
      _ => {}
  }
});
```

```javascript
let requestId = 0;
const pending = new Map();

window.ipcResponseHandler = function (response) {
  const { id, result } = response;

  if (pending.has(id)) {
    pending.get(id)(result);
    pending.delete(id);
  }
};

export const ipc = {
  call(method, params = {}) {
    return new Promise((resolve) => {
      const id = requestId++;

      pending.set(id, resolve);

      window.ipc.postMessage(JSON.stringify({
        id,
        method,
        params
      }));
    });
  }
};

const files = await ipc.call("list_files");
console.log(files);
```


### Kiosk mode stuff

```javascript

(() => {
  // Disable right-click context menu
  window.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }, true);

  // Disable text selection
  document.addEventListener("selectstart", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }, true);

  // Disable drag & drop (prevents dragging images/links out, etc)
  ["dragstart", "drop", "dragover"].forEach((eventName) => {
    document.addEventListener(eventName, (e) => {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }, true);
  });

  // Disable middle-click / aux click (open new tab behavior in some engines)
  document.addEventListener("auxclick", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }, true);

  // Disable mouse down on middle click
  document.addEventListener("mousedown", (e) => {
    if (e.button === 1) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
  }, true);

  // Disable keyboard shortcuts
  document.addEventListener("keydown", (e) => {
    const key = e.key.toLowerCase();

    // Disable refresh (F5 / Ctrl+R)
    if (e.key === "F5" || (e.ctrlKey && key === "r")) {
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

    // Disable Alt+Left / Backspace navigation
    if (e.altKey && (key === "arrowleft" || key === "left")) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
    if (e.key === "Backspace") {
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
    if (e.ctrlKey && ["+", "-", "=", "0"].includes(e.key)) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }

    // Disable Escape (optional, can break UI dialogs)
    // if (e.key === "Escape") {
    //   e.preventDefault();
    //   e.stopPropagation();
    //   return false;
    // }

  }, true);

  // Disable wheel zoom (Ctrl+Scroll)
  document.addEventListener("wheel", (e) => {
    if (e.ctrlKey) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
  }, { passive: false, capture: true });

  // Disable pinch zoom gesture
  document.addEventListener("gesturestart", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }, true);

  document.addEventListener("gesturechange", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }, true);

  document.addEventListener("gestureend", (e) => {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }, true);

  // Optional: disable double click zoom (some webviews)
  document.addEventListener("dblclick", (e) => {
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

  console.log("[kiosk] hardening enabled");
})();

```