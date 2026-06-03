type IpcPostMessageKind = "DeviceDiscovered";
type IpcPostMessage = {
    kind: IpcPostMessageKind;
    data: Record<string, any>;
};
type IpcMethod =
    | "ListFiles"
    | "DiscoverDevices"
    | "ConnectToDevice"
    | "DisconnectFromDevice"
    | "RequestCastLocal"

declare global {
    interface Window {
        ipc: {
            postMessage: (message: string) => void;
        };
        ipc_handler: {
            responseHandler: (response: any) => void;
            currRequestId: number;
            pendingRequests: Map<number, (value: unknown) => void>;
        };
    }
}

export const ipc = {
    call(method: IpcMethod, params = {}) {
        return new Promise((resolve) => {
            const id = window.ipc_handler.currRequestId++;
            window.ipc_handler.pendingRequests.set(id, resolve);
            window.ipc.postMessage(
                JSON.stringify({
                    id,
                    method,
                    params,
                }),
            );
        });
    },
    listen(cb: (ipcMessage: IpcPostMessage) => void) {
        window.addEventListener("message", ({ data }) => {
            cb(data);
        });
    },
};
