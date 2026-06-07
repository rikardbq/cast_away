import { Link } from "react-router";

import "../app.css";
import { useEffect, useMemo, useRef, useState } from "react";
import { useRateLimit } from "../hooks/useRateLimit";
import type { GamepadUtils } from "../hooks/useGamepad";

// const ListItem = ({ id, name, description: _, focused, ...rest }: any) => {
//     return (
//         <li id={id} {...rest}>
//             <div
//                 className={`transition-all duration-150 ease-in-out min-w-40 min-h-40 ${focused ? "scale-125 border-primary rounded-xl shadow-[0px_0px_20px_5px_rgba(0,0,0,0.25)] shadow-primary" : "shadow-md rounded-lg border-transparent"}`}
//                 // style={{
//                 //     width: "500px",
//                 //     height: "500px",
//                 //     border: focused ? "2px solid green" : "",
//                 // }}
//             >
//                 {name}
//             </div>
//         </li>
//     );
// };

const testItems = [
    {
        name: "Paramount+",
        desc: "Halo",
        focused: false,
        vendor_image: "",
    },
    {
        name: "Netflix",
        desc: "description",
        focused: false,
        vendor_image: "",
    },
    {
        name: "N chill",
        desc: "description 2 chill 2",
        focused: false,
        vendor_image: "",
    },
    {
        name: "HBO",
        desc: "description 3",
        focused: false,
        vendor_image: "",
    },
    {
        name: "PRIME",
        desc: "description 7",
        focused: false,
        vendor_image: "",
    },
    {
        name: "Apple TV",
        desc: "description apple",
        focused: false,
        vendor_image: "",
    },
    {
        name: "Viaplay",
        desc: "aaaaaaaaaa",
        focused: false,
        vendor_image: "",
    },
    {
        name: "chromecast",
        desc: "aaaaaaaaaa",
        focused: false,
        vendor_image: "../vendor/chromecast_select.webp",
    },
];

const keyDownHandler =
    (
        currFocus: number,
        setFocused: Function,
        list: any[],
        padded_list: any[],
    ) =>
    (ev: any) => {
        ev.preventDefault();
        const list_threshold = list.length > 3 ? 3 : list.length;
        if (ev.code === "ArrowLeft" || ev.code === "ArrowUp") {
            const nextFocus = currFocus - 1;
            const willoop = nextFocus < list_threshold;
            setFocused(
                willoop ? padded_list.length - (list_threshold + 1) : nextFocus,
            );
        } else if (ev.code === "ArrowRight" || ev.code === "ArrowDown") {
            const nextFocus = currFocus + 1;
            const willoop =
                nextFocus > padded_list.length - (list_threshold + 1);
            setFocused(willoop ? list_threshold : nextFocus);
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
    list: any[],
    padded_list: any[],
) => {
    const list_threshold = list.length > 3 ? 3 : list.length;
    let classes = "";
    if (
        current_focus === list_threshold &&
        idx === padded_list.length - (list_threshold + 1)
    ) {
        classes = "loop l";
    }
    if (
        current_focus === padded_list.length - (list_threshold + 1) &&
        idx === 3
    ) {
        classes = "loop r";
    }

    return classes;
};

export default ({
    gamepadUtils: {
        gamepads,
        isButtonPressed,
        stick: { moveX: _moveX, moveY, deadzone },
    },
}: Props) => {
    const limitRate = useRateLimit();
    const vendorSelect = useRef("");
    const [focusedElem, _setFocusedElem] = useState("back_btn");
    const gamepad = useMemo(() => gamepads[0], [gamepads]);
    const [items, _setItems] = useState([
        ...testItems.slice(-3),
        ...testItems,
        ...testItems.slice(0, 3),
    ]);
    const [previousFocus, setPreviousFocus] = useState(
        Math.floor(items.length / 2),
    );
    const [currentFocus, setCurrentFocus] = useState(previousFocus);
    const setFocused = (next_focus: number) => {
        setPreviousFocus(
            next_focus - currentFocus > 1
                ? next_focus + 1
                : next_focus - currentFocus < -1
                  ? next_focus - 1
                  : currentFocus,
        );

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
    const navHandler = useRef(
        keyDownHandler(currentFocus, setFocused, testItems, items),
    );

    useEffect(() => {
        return () => {
            window.removeEventListener("keydown", navHandler.current);
        };
    }, []);

    useEffect(() => {
        window.removeEventListener("keydown", navHandler.current);
        navHandler.current = keyDownHandler(
            currentFocus,
            setFocused,
            testItems,
            items,
        );
        window.addEventListener("keydown", navHandler.current);
    }, [currentFocus]);

    useEffect(() => {
        if (gamepad) {
            if (
                isButtonPressed(gamepad, "XBOX.DPAD_UP") ||
                moveY(gamepad, "LEFT_STICK") < 0 - deadzone
            ) {
                const nextFocus = currentFocus - 1;
                const willoop = nextFocus < 0;
                limitRate(
                    () => setFocused(willoop ? items.length - 1 : nextFocus),
                    250,
                );
            } else if (
                isButtonPressed(gamepad, "XBOX.DPAD_DOWN") ||
                moveY(gamepad, "LEFT_STICK") > 0 + deadzone
            ) {
                const nextFocus = currentFocus + 1;
                const willoop = nextFocus > items.length - 1;
                limitRate(() => setFocused(willoop ? 0 : nextFocus), 250);
            }
        }
    });

    // prevent list from getting bounds issues
    // split list
    // floor value
    // slice from index floor val
    // take original list and slice on index 0 up to its split floor val
    // combine with first list slice
    /*
    > let list = [1,2,3,4,5];
    > let list_s1 = list.slice(Math.floor(list.length / 2));
    > list_s1
    [ 3, 4, 5 ]
    > let list_s2 = list.slice(0, Math.floor(list.length / 2))
    > list_s2
    [ 1, 2 ]
    > [...list_s1, ...list_s2]
    [ 3, 4, 5, 1, 2 ]
    >
    */

    return (
        <div
            style={{
                height: "100vh",
                width: "100vw",
                background: "url(../wp.jpg)",
                display: "flex",
                flexDirection: "row",
            }}
        >
            <div
                style={{
                    height: "100vh",
                    width: "100vw",
                    backgroundImage:
                        "linear-gradient(to right, black 25%, transparent 100%)",
                    position: "absolute",
                    left: 0,
                    top: 0,
                }}
            />
            <div
                style={{
                    placeContent: "center",
                    placeItems: "center",
                    height: "100vh",
                    width: "40vw",
                }}
            >
                <ul
                    className="x-items"
                    style={{
                        lineHeight: "normal",
                    }}
                >
                    {items.map((x, idx) => {
                        // ${getItemStyles(idx, currentFocus, items, testItems)}
                        let fontFamily = "FetteUnzFraktur";
                        let textColor = "#FF4444";
                        if (x.name === "chromecast") {
                            fontFamily = "Cyberpunk";
                            textColor = "#ffff44";
                            // change this later, possible this location causes the jank
                            if (idx === currentFocus) {
                                vendorSelect.current = "chromecast_select.webp";
                            } else {
                                vendorSelect.current = "";
                            }
                        }
                        return (
                            <li
                                key={idx}
                                className={`${getKeyFrameAnim(idx, currentFocus, previousFocus)}
                            ${idx === currentFocus ? `selected${currentFocus > previousFocus ? " r" : currentFocus < previousFocus ? " l" : ""}` : ""}
                            ${willoopStyles(idx, currentFocus, testItems, items)}
                                 `}
                                style={{
                                    width: "max-content",
                                    position: "absolute",
                                    fontFamily,
                                    fontSize: "42px",
                                    color: textColor,
                                }}
                            >
                                {x.name.toLowerCase()}
                            </li>
                        );
                    })}
                </ul>
            </div>
            {vendorSelect.current.length > 0 ? (
                <div
                    style={{
                        placeContent: "center",
                        zIndex: 1,
                        overflow: "hidden",
                        width: "60vw",
                    }}
                >
                    <img
                        src={`../vendor/${vendorSelect.current}`}
                        className="vendor-select-slide-in"
                    />
                </div>
            ) : null}
            <div>
                <Link
                    style={{
                        fontFamily: "FetteUnzFraktur",
                        color: "#FF4444",
                        fontSize: "42px",
                        lineHeight: "normal",
                        borderRadius: "10px",
                        padding: "4px 16px",
                        border:
                            focusedElem === "back_btn"
                                ? "2px solid #dddddd"
                                : "none",
                    }}
                    className="absolute bottom-5 left-8 hover-3d"
                    to="/"
                >
                    back
                </Link>
                {/* <div className="btn btn-error">
                </div> */}
            </div>
        </div>
    );
};
