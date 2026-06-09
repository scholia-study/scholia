import "driver.js/dist/driver.css";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { driver } from "driver.js";
import { useCallback } from "react";
import toast from "react-hot-toast";
import { getGetTocQueryOptions } from "#/api/toc/toc";
import {
    buildTourSteps,
    firstContentNodeSlug,
    TOUR_BOOK_SLUG,
    TOUR_SEEN_KEY,
} from "./tourConfig";
import { waitForElement } from "./waitForElement";

const OVERLAY_COLOR = "#1c1917"; // stone-900

/** Drives a guided tour of the reader, pinned to the KJV Bible. The tour steers
 *  the reader purely through the router (URL search state) and anchors popovers
 *  to DOM data-attributes, so it never reaches into reader internals. */
export function useReaderTour() {
    const navigate = useNavigate();
    const queryClient = useQueryClient();

    const startReaderTour = useCallback(async () => {
        // Resolve the demo book's first content node; bail gracefully if the
        // KJV Bible isn't ingested rather than navigating to a broken page.
        let nodeSlug: string | undefined;
        try {
            const toc = await queryClient.fetchQuery(
                getGetTocQueryOptions(TOUR_BOOK_SLUG),
            );
            nodeSlug = firstContentNodeSlug(toc.data);
        } catch {
            nodeSlug = undefined;
        }
        if (!nodeSlug) {
            toast.error("The guided tour isn't available right now.");
            return;
        }

        const steps = buildTourSteps();
        const params = { bookSlug: TOUR_BOOK_SLUG, nodeSlug };

        // Apply a step's reader state, then wait for its anchor to render.
        const prepareStep = async (index: number, replace = true) => {
            const step = steps[index];
            await navigate({
                to: "/books/$bookSlug/$nodeSlug",
                params,
                search: step.search,
                replace,
            });
            if (step.element) {
                await waitForElement(step.element).catch(() => {});
            }
        };

        const driverObj = driver({
            showProgress: true,
            smoothScroll: true,
            overlayColor: OVERLAY_COLOR,
            stagePadding: 6,
            nextBtnText: "Next",
            prevBtnText: "Back",
            doneBtnText: "Done",
            steps: steps.map((s) => ({
                element: s.element,
                popover: s.popover,
            })),
            onNextClick: async (_el, _step, { state }) => {
                const next = (state.activeIndex ?? 0) + 1;
                if (next >= steps.length) {
                    driverObj.destroy();
                    return;
                }
                await prepareStep(next);
                driverObj.moveNext();
            },
            onPrevClick: async (_el, _step, { state }) => {
                const prev = (state.activeIndex ?? 0) - 1;
                if (prev < 0) return;
                await prepareStep(prev);
                driverObj.movePrevious();
            },
        });

        // First hop pushes a history entry (so Back leaves the tour cleanly);
        // subsequent steps replace it.
        await prepareStep(0, false);
        driverObj.drive(0);
    }, [navigate, queryClient]);

    /** Show the one-time welcome prompt on first visit. Marks "seen" the moment
     *  it's shown — accepted or dismissed, it never auto-nags again. */
    const maybeWelcome = useCallback(() => {
        if (typeof window === "undefined") return;
        if (window.localStorage.getItem(TOUR_SEEN_KEY)) return;
        window.localStorage.setItem(TOUR_SEEN_KEY, "1");

        const welcome = driver({
            overlayColor: OVERLAY_COLOR,
            steps: [
                {
                    popover: {
                        title: "Welcome to Scholia",
                        description:
                            "Want a quick tour of how to read, select, and annotate a text?",
                        showButtons: ["next", "close"],
                        nextBtnText: "Show me around",
                    },
                },
            ],
            onNextClick: () => {
                welcome.destroy();
                void startReaderTour();
            },
        });
        welcome.drive();
    }, [startReaderTour]);

    return { startReaderTour, maybeWelcome };
}
