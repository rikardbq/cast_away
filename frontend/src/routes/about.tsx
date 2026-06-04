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

export default ({
    gamepadUtils: {
        gamepads,
        isButtonPressed,
        stick: { moveX, deadzone },
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

    if (gamepad) {
        if (
            isButtonPressed(gamepad, "XBOX.DPAD_LEFT") ||
            moveX(gamepad, "LEFT_STICK") < 0 - deadzone
        ) {
            const nFocus = currentFocus - 1;
            limitRate(
                () => setFocused(nFocus < 0 ? items.length - 1 : nFocus),
                100,
            );
            // if (currentFocus !== 0) {
            // }
        } else if (
            isButtonPressed(gamepad, "XBOX.DPAD_RIGHT") ||
            moveX(gamepad, "LEFT_STICK") > 0 + deadzone
        ) {
            const nFocus = currentFocus + 1;
            limitRate(
                () => setFocused(nFocus > items.length - 1 ? 0 : nFocus),
                100,
            );
            // if (currentFocus !== items.length - 1) {
            // }
        }
    }

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
            <ul
                style={{
                    position: "absolute",
                }}
            >
                {items.map((x, idx) => {
                    const getKeyFrameAnim = () => {
                        if (currentFocus > previousFocus) {
                            switch (idx) {
                                case currentFocus - 1:
                                    return "down_prev_1";
                                case currentFocus - 2:
                                    return "down_prev_2";
                                case currentFocus - 3:
                                    return "down_prev_3";
                                case currentFocus + 1:
                                    return "down_next_1";
                                case currentFocus + 2:
                                    return "down_next_2";
                                case currentFocus + 3:
                                    return "down_next_3";
                                default:
                                    return "";
                            }
                        } else if (currentFocus < previousFocus) {
                            switch (idx) {
                                case currentFocus - 1:
                                    return "up_prev_1";
                                case currentFocus - 2:
                                    return "up_prev_2";
                                case currentFocus - 3:
                                    return "up_prev_3";
                                case currentFocus + 1:
                                    return "up_next_1";
                                case currentFocus + 2:
                                    return "up_next_2";
                                case currentFocus + 3:
                                    return "up_next_3";
                                default:
                                    return "";
                            }
                        }
                    };
                    return (
                        <li
                            key={idx}
                            className={getKeyFrameAnim()}
                            style={{
                                width: "max-content",
                                position: "absolute",
                                backgroundColor:
                                    idx === currentFocus
                                        ? "coral"
                                        : "lightblue",
                                transition:
                                    idx === currentFocus
                                        ? "opacity 0.250s linear, transform 0.250s cubic-bezier(.14,.91,.41,1.32)"
                                        : "",
                                opacity:
                                    idx === currentFocus
                                        ? 1
                                        : idx === currentFocus + 1 ||
                                            idx === currentFocus - 1
                                          ? 0.5
                                          : idx === currentFocus + 2 ||
                                              idx === currentFocus - 2
                                            ? 0.15
                                            : 0,
                                transformOrigin: "left",
                                transform: (() => {
                                    if (idx === currentFocus) {
                                        return "scale(1.5) translate3d(0px, 0px, 0px)";
                                    } else {
                                        if (
                                            idx === currentFocus - 1 ||
                                            (currentFocus === 3 &&
                                                idx === currentFocus + (testItems.length - 1))
                                        ) {
                                            return "translate3d(0px, -100px, 0px)";
                                        }
                                        if (idx === currentFocus - 2) {
                                            return "translate3d(0px, -200px, 0px)";
                                        }
                                        if (idx === currentFocus - 3) {
                                            return "translate3d(0px, -300px, 0px)";
                                        }
                                        if (idx === currentFocus + 1 ||
                                            (currentFocus === items.length - 4 &&
                                                idx === currentFocus - (testItems.length - 1))) {
                                            return "translate3d(0px, 100px, 0px)";
                                        }
                                        if (idx === currentFocus + 2) {
                                            return "translate3d(0px, 200px, 0px)";
                                        }
                                        if (idx === currentFocus + 3) {
                                            return "translate3d(0px, 300px, 0px)";
                                        }
                                        return "";
                                    }
                                })(),
                            }}
                        >
                            {x.name}
                        </li>
                    );
                })}
            </ul>
        </>
    );
};
