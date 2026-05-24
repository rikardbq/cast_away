import { useCallback, useEffect, useState } from "react";

// UTILITY
const GAMEPAD_BUTTONS = {
    XBOX: {
        A: 0,
        B: 1,
        X: 2,
        Y: 3,
        LB: 4,
        RB: 5,
        LT: 6,
        RT: 7,
        SELECT: 8,
        START: 9,
        THUMB_LEFT: 10,
        THUMB_RIGHT: 11,
        DPAD_UP: 12,
        DPAD_DOWN: 13,
        DPAD_LEFT: 14,
        DPAD_RIGHT: 15,
    },
};

const GAMEPAD_AXES = {
    LEFT_STICK: {
        X_AXIS: 0,
        Y_AXIS: 1,
    },
    RIGHT_STICK: {
        X_AXIS: 2,
        Y_AXIS: 3,
    },
};

type GamepadButtons = {
    [K in keyof typeof GAMEPAD_BUTTONS]: `${K & string}.${keyof (typeof GAMEPAD_BUTTONS)[K] & string}`;
}[keyof typeof GAMEPAD_BUTTONS];
type GamepadAxes = typeof GAMEPAD_AXES;

const isButtonPressed = (gamepad: Gamepad, button: GamepadButtons) => {
    const [controller, key] = button.split(".") as [
        keyof typeof GAMEPAD_BUTTONS,
        string,
    ];
    const val = (GAMEPAD_BUTTONS[controller] as any)[key];
    return gamepad.buttons[val].pressed;
};

const moveX = (gamepad: Gamepad, stick: keyof GamepadAxes) => {
    return gamepad.axes[(GAMEPAD_AXES as GamepadAxes)[stick].X_AXIS];
};

const moveY = (gamepad: Gamepad, stick: keyof GamepadAxes) => {
    return gamepad.axes[(GAMEPAD_AXES as GamepadAxes)[stick].Y_AXIS];
};
// UTILITY END

type Gamepads = (Gamepad | null)[];
type GamepadUtilsOptions = {
    pollRate?: number;
    stickDeadzone?: number;
};

export type GamepadUtils = {
    gamepads: Gamepads;
    isButtonPressed: (gamepad: Gamepad, button: GamepadButtons) => boolean;
    stick: {
        deadzone: number;
        moveX: (gamepad: Gamepad, stick: keyof GamepadAxes) => number;
        moveY: (gamepad: Gamepad, stick: keyof GamepadAxes) => number;
    };
};

export const useGamepad = ({
    pollRate = 100,
    stickDeadzone = 0.1,
}: GamepadUtilsOptions): GamepadUtils => {
    const [gamepads, setGamepads] = useState<Gamepads>([]);

    const connectedHandler = useCallback((ev: GamepadEvent) => {
        const evGamepad = ev.gamepad;
        const navigatorGamepads = navigator.getGamepads();

        if (navigatorGamepads[evGamepad.index]) {
            console.debug(`GAMEPAD ${evGamepad.index} CONNECTED`);
            setGamepads(navigatorGamepads);
        }
    }, []);

    const disconnectedHandler = useCallback((ev: GamepadEvent) => {
        const evGamepad = ev.gamepad;
        console.debug(`GAMEPAD ${evGamepad.index} DISCONNECTED`);
        setGamepads(gamepads.filter((_x, idx) => idx === evGamepad.index));
    }, []);

    useEffect(() => {
        const navigatorGamepads = navigator.getGamepads();
        if (navigatorGamepads.length === 0) {
            return;
        }

        setTimeout(() => {
            setGamepads(navigatorGamepads);
        }, pollRate);
    });

    useEffect(() => {
        window.addEventListener("gamepadconnected", connectedHandler);
        window.addEventListener("gamepaddisconnected", disconnectedHandler);

        return () => {
            window.removeEventListener("gamepadconnected", connectedHandler);
            window.removeEventListener(
                "gamepaddisconnected",
                disconnectedHandler,
            );
        };
    }, []);

    return {
        gamepads,
        isButtonPressed,
        stick: {
            deadzone: stickDeadzone,
            moveX,
            moveY,
        },
    };
};
