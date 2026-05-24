import type { PropsWithChildren } from "react"

export const View = (props: PropsWithChildren) => {

    return (
        <div className="flex items-center">
            {props.children}
        </div>
    )
}