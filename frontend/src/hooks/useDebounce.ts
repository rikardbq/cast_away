import { useRef, useCallback, useEffect } from "react";

export const useDebounce = () => {
    const timer = useRef<number | undefined>(undefined);
    const debouncer = useCallback((cb: Function, time: number = 250) => {
        clearTimeout(timer.current);
        timer.current = setTimeout(() => {
            cb();
        }, time);
    }, []);

    useEffect(() => {
        return () => {
            clearTimeout(timer.current);
        };
    }, []);

    return debouncer;
};
