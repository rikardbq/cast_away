import { useRef, useCallback } from "react";

export const useRateLimit = () => {
    const oldNow = useRef<number>(Date.now());
    const rateLimiter = useCallback((cb: Function, time: number = 250) => {
        const now = Date.now();
        if (now > oldNow.current + time) {
            oldNow.current = now;
            cb();
        }
    }, []);

    return rateLimiter;
};
