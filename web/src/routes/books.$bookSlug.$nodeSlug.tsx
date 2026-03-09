import { createFileRoute } from "@tanstack/react-router";
import { getGetTocQueryOptions } from "../api/toc/toc";
import type { PanelState } from "../components/ReaderLayout";
import { ReaderLayout } from "../components/ReaderLayout";

export type ReaderSearch = {
    p2?: string;
    p3?: string;
    p4?: string;
    s?: string;
    s2?: string;
    s3?: string;
    s4?: string;
    r?: string;
    r2?: string;
    r3?: string;
    r4?: string;
};

function parsePanel(param: string): PanelState {
    const slashIdx = param.indexOf("/");
    if (slashIdx === -1) return { bookSlug: param, nodeSlug: undefined };
    return {
        bookSlug: param.slice(0, slashIdx),
        nodeSlug: param.slice(slashIdx + 1) || undefined,
    };
}

export const Route = createFileRoute("/books/$bookSlug/$nodeSlug")({
    validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
        p2: search.p2 as string | undefined,
        p3: search.p3 as string | undefined,
        p4: search.p4 as string | undefined,
        s: search.s as string | undefined,
        s2: search.s2 as string | undefined,
        s3: search.s3 as string | undefined,
        s4: search.s4 as string | undefined,
        r: search.r as string | undefined,
        r2: search.r2 as string | undefined,
        r3: search.r3 as string | undefined,
        r4: search.r4 as string | undefined,
    }),
    loader: async ({ context, params }) => {
        await context.queryClient.ensureQueryData(
            getGetTocQueryOptions(params.bookSlug),
        );
    },
    component: ReaderPage,
});

function ReaderPage() {
    const { bookSlug, nodeSlug } = Route.useParams();
    const search = Route.useSearch();

    const panels: PanelState[] = [
        { bookSlug, nodeSlug },
        ...(search.p2 ? [parsePanel(search.p2)] : []),
        ...(search.p3 ? [parsePanel(search.p3)] : []),
        ...(search.p4 ? [parsePanel(search.p4)] : []),
    ];

    const selections = new Map<number, string>();
    if (search.s) selections.set(0, search.s);
    if (search.s2) selections.set(1, search.s2);
    if (search.s3) selections.set(2, search.s3);
    if (search.s4) selections.set(3, search.s4);

    const resourcesOpen = new Set<number>();
    if (search.r) resourcesOpen.add(0);
    if (search.r2) resourcesOpen.add(1);
    if (search.r3) resourcesOpen.add(2);
    if (search.r4) resourcesOpen.add(3);

    return (
        <ReaderLayout
            panels={panels}
            selections={selections}
            resourcesOpen={resourcesOpen}
        />
    );
}
