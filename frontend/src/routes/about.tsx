import { Link } from "react-router";

import "../app.css";
import { useEffect, useMemo, useRef, useState } from "react";
import { useRateLimit } from "../hooks/useRateLimit";
import type { GamepadUtils } from "../hooks/useGamepad";

const ListItem = ({ id, name, description: _, focused, ...rest }: any) => {
    return (
        <li id={id} {...rest}>
            <div
                className={`transition-all duration-150 ease-in-out min-w-40 min-h-40 ${focused ? "scale-125 border-primary rounded-xl shadow-[0px_0px_20px_5px_rgba(0,0,0,0.25)] shadow-primary" : "shadow-md rounded-lg border-transparent"}`}
                // style={{
                //     width: "500px",
                //     height: "500px",
                //     border: focused ? "2px solid green" : "",
                // }}
            >
                {name}
            </div>
        </li>
    );
};

const testItems = [
    {
        name: "Paramount+",
        desc: "Halo",
        focused: false,
    },
    {
        name: "Netflix",
        desc: "description",
        focused: false,
    },
    {
        name: "N chill",
        desc: "description 2 chill 2",
        focused: false,
    },
    {
        name: "HBO",
        desc: "description 3",
        focused: false,
    },
    {
        name: "PRIME",
        desc: "description 7",
        focused: false,
    },
    {
        name: "Apple TV",
        desc: "description apple",
        focused: false,
    },
    {
        name: "Viaplay",
        desc: "aaaaaaaaaa",
        focused: false,
    },
];

const keyDownHandler =
    (currFocus: number, setFocused: Function, items: any[]) => (ev: any) => {
        ev.preventDefault();
        if (ev.code === "ArrowLeft" || ev.code === "ArrowUp") {
            const nextFocus = currFocus - 1;
            const willoop = nextFocus < 3;
            setFocused(willoop ? items.length - 5 : nextFocus);
        } else if (ev.code === "ArrowRight" || ev.code === "ArrowDown") {
            const nextFocus = currFocus + 1;
            const willoop = nextFocus > items.length - 4;
            setFocused(willoop ? 4 : nextFocus);
        }
        console.log(ev.code);
    };

type Props = {
    gamepadUtils: GamepadUtils;
};

const getKeyFrameAnim = (
    idx: number,
    current_focus: number,
    previous_focus: number,
) => {
    for (let i = 1; i <= 3; i++) {
        let pn = undefined;
        if (idx === current_focus - i) pn = `left-${i}`;
        if (idx === current_focus + i) pn = `right-${i}`;
        if (pn) {
            if (current_focus > previous_focus) return `${pn} r`;
            if (current_focus < previous_focus) return `${pn} l`;
            return pn;
        }
    }

    return "";
};

const willoopStyles = (
    idx: number,
    current_focus: number,
    items_splice: any[],
    items: any[],
) => {
    if (current_focus === 3 && idx === current_focus + (items.length - 1)) {
        return "loop l";
    }
    if (
        current_focus === items_splice.length - 4 &&
        idx === current_focus - (items.length - 1)
    ) {
        return "loop r";
    }

    return "";
};

export default ({
    gamepadUtils: {
        gamepads,
        isButtonPressed,
        stick: { moveX: _moveX, moveY, deadzone },
    },
}: Props) => {
    const limitRate = useRateLimit();
    const gamepad = useMemo(() => gamepads[0], [gamepads]);
    const [items, _setItems] = useState([...testItems, ...testItems]);
    const [previousFocus, setPreviousFocus] = useState(testItems.length);
    const [currentFocus, setCurrentFocus] = useState(testItems.length);
    const setFocused = (next_focus: number) => {
        setPreviousFocus(
            next_focus - currentFocus > 1
                ? next_focus + 1
                : next_focus - currentFocus < -1
                  ? next_focus - 1
                  : currentFocus,
        );
        console.log("looped left ", next_focus - currentFocus > 1);

        setCurrentFocus(next_focus);
        document.getElementById(`${next_focus}`)?.scrollIntoView({
            behavior: "smooth",
        });
    };
    // const setFocused = (idx: number) => {
    //     setPreviousFocus(currentFocus);
    //     setCurrentFocus(idx);
    //     // setItems(
    //     //     items.map((y, i) => ({
    //     //         ...y,
    //     //         focused: idx === i,
    //     //     })),
    //     // );
    //     document.getElementById(`${idx}`)?.scrollIntoView({
    //         behavior: "smooth",
    //     });
    // };
    const navHandler = useRef(keyDownHandler(currentFocus, setFocused, items));

    useEffect(() => {
        return () => {
            window.removeEventListener("keydown", navHandler.current);
        };
    }, []);

    useEffect(() => {
        window.removeEventListener("keydown", navHandler.current);
        navHandler.current = keyDownHandler(currentFocus, setFocused, items);
        window.addEventListener("keydown", navHandler.current);
    }, [currentFocus]);

    useEffect(() => {
        if (gamepad) {
            if (
                isButtonPressed(gamepad, "XBOX.DPAD_UP") ||
                moveY(gamepad, "LEFT_STICK") < 0 - deadzone
            ) {
                const nextFocus = currentFocus - 1;
                const willoop = nextFocus < 3;
                limitRate(
                    () => setFocused(willoop ? items.length - 5 : nextFocus),
                    250,
                );
            } else if (
                isButtonPressed(gamepad, "XBOX.DPAD_DOWN") ||
                moveY(gamepad, "LEFT_STICK") > 0 + deadzone
            ) {
                const nextFocus = currentFocus + 1;
                const willoop = nextFocus > items.length - 4;
                limitRate(() => setFocused(willoop ? 4 : nextFocus), 250);
            }
        }
    });

    return (
        <>
            <div>
                <ul
                    style={{
                        display: "flex",
                        flexDirection: gamepad?.buttons[0]?.pressed
                            ? "column"
                            : "row",
                        gap: "5px",
                    }}
                >
                    {items.map((x, idx) => (
                        <ListItem
                            onClick={() => {
                                setFocused(idx);
                            }}
                            id={idx}
                            key={`${x.name}:${idx}`}
                            name={x.name}
                            focused={idx === currentFocus}
                        />
                    ))}
                </ul>
            </div>
            <h1>TEST</h1>
            <Link to="/">Home</Link>
            <div>
                <ul
                    className="x-items"
                    style={{
                        position: "absolute",
                    }}
                >
                    {items.map((x, idx) => {
                        // ${getItemStyles(idx, currentFocus, items, testItems)}
                        return (
                            <li
                                key={idx}
                                className={`${getKeyFrameAnim(idx, currentFocus, previousFocus)}
                                 ${idx === currentFocus ? "selected" : ""}
                                 ${willoopStyles(idx, currentFocus, items, testItems)}
                                 `}
                                style={{
                                    width: "max-content",
                                    position: "absolute",
                                    backgroundColor:
                                        idx === currentFocus
                                            ? "coral"
                                            : "lightblue",
                                }}
                            >
                                {x.name}
                            </li>
                        );
                    })}
                </ul>
            </div>
        </>
    );
};
