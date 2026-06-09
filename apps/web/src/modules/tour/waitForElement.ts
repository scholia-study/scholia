/** Resolve once a matching element is in the DOM, scrolling it into view.
 *  Rejects after `timeoutMs`. Bridges the gap between a router navigation and
 *  React committing the resulting DOM, so a tour step can anchor reliably to a
 *  freshly-rendered element (a selected sentence, the resources panel, …).
 *
 *  Note: the reader is NOT windowed — once content is rendered it stays in the
 *  DOM, so this only waits out the initial navigate→render, never a re-mount. */
export function waitForElement(
    selector: string,
    timeoutMs = 4000,
): Promise<HTMLElement> {
    return new Promise((resolve, reject) => {
        const found = () => document.querySelector<HTMLElement>(selector);

        const settle = (el: HTMLElement) => {
            el.scrollIntoView({ block: "center", behavior: "smooth" });
            resolve(el);
        };

        const immediate = found();
        if (immediate) {
            settle(immediate);
            return;
        }

        const start = performance.now();
        const tick = () => {
            const el = found();
            if (el) {
                settle(el);
                return;
            }
            if (performance.now() - start > timeoutMs) {
                reject(
                    new Error(
                        `waitForElement: "${selector}" not found within ${timeoutMs}ms`,
                    ),
                );
                return;
            }
            requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
    });
}
