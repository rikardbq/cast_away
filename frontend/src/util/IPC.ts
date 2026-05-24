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
    call(method: string, params = {}) {
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
};
