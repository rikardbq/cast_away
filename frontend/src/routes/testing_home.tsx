import { View } from "../components/container/view";
import marioPoster from "../assets/mario-movie-poster.jpg";
import type { GamepadUtils } from "../hooks/useGamepad";
import { useCallback, useMemo, useRef, useState } from "react";
import { useRateLimit } from "../hooks/useRateLimit";
import "../app.css";
import { Link } from "react-router";

const arr = new Array(10).fill(0);
console.log(arr.length);

type Props = {
    gamepadUtils: GamepadUtils;
};

export default ({
    gamepadUtils: {
        gamepads,
        isButtonPressed,
        stick: { deadzone, moveX },
    },
}: Props) => {
    const [gamepad1, ..._] = useMemo(() => gamepads, [gamepads]);
    const limitRate = useRateLimit();
    const [focused, setFocused] = useState(arr.map((_, i) => i === 0));
    const itemRef = useRef(null);
    const nextFocusRef = useRef(0);
    const navx = useCallback(
        () =>
            setFocused(() => {
                const elementRef = itemRef.current! as HTMLElement;
                elementRef.scrollIntoView({
                    behavior: "smooth",
                });
                
                return focused.map((_, i) => i === nextFocusRef.current);
            }),
        [focused],
    );

    if (gamepad1) {
        // if (focused.indexOf(true) !== 0) {
        //     onButtonPressed("XBOX.DPAD_LEFT", () => {
        //         const nextFocus = focused.indexOf(true) - 1;
        //         setFocused(() => {
        //             const elementRef = itemRef.current! as HTMLElement;
        //             elementRef.scrollIntoView({
        //                 behavior: "smooth",
        //             });
        //             return focused.map((_, i) => i === nextFocus);
        //         });
        //     });
        // }

        // if (focused.indexOf(true) !== arr.length - 1) {
        //     const nextFocus = focused.indexOf(true) + 1;
        //     onButtonPressed("XBOX.DPAD_RIGHT", () => {
        //         setFocused(() => {
        //             const elementRef = itemRef.current! as HTMLElement;
        //             elementRef.scrollIntoView({
        //                 behavior: "smooth",
        //             });
        //             return focused.map((_, i) => i === nextFocus);
        //         });
        //     });
        // }

        if (
            isButtonPressed(gamepad1, "XBOX.DPAD_LEFT") ||
            moveX(gamepad1, "LEFT_STICK") < 0 - deadzone
        ) {
            if (focused.indexOf(true) !== 0) {
                nextFocusRef.current = focused.indexOf(true) - 1;
                limitRate(navx, 100);
            }
        } else if (
            isButtonPressed(gamepad1, "XBOX.DPAD_RIGHT") ||
            moveX(gamepad1, "LEFT_STICK") > 0 + deadzone
        ) {
            if (focused.indexOf(true) !== arr.length - 1) {
                nextFocusRef.current = focused.indexOf(true) + 1;
                limitRate(navx, 100);
            }
        }
    }

    return (
        <View>
            <Link to="/">Home</Link>
            <div className="tesst flex flex-row items-end h-lvh gap-2">
                {arr.map((_x, i) => (
                    <div
                        ref={nextFocusRef.current === i ? itemRef : null}
                        key={i}
                        id={`${i}`}
                        className={`transition-all duration-150 ease-in-out min-w-40 min-h-40 max-w-40 max-h-40 overflow-clip border-2 ${focused[i] ? "scale-125 border-primary rounded-xl shadow-[0px_0px_20px_5px_rgba(0,0,0,0.25)] shadow-primary" : "shadow-md rounded-lg border-transparent"}`}
                    >
                        <img className="" src={marioPoster} />
                    </div>
                ))}
            </div>
        </View>
    );
};
