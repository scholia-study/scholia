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
    og?: string;
    og2?: string;
    og3?: string;
    og4?: string;
    rv?: string;
    rv2?: string;
    rv3?: string;
    rv4?: string;
    vm?: string;
    vm2?: string;
    vm3?: string;
    vm4?: string;
    vl?: string;
    vl2?: string;
    vl3?: string;
    vl4?: string;
    vt?: string;
    vt2?: string;
    vt3?: string;
    vt4?: string;
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
        og: search.og as string | undefined,
        og2: search.og2 as string | undefined,
        og3: search.og3 as string | undefined,
        og4: search.og4 as string | undefined,
        rv: search.rv as string | undefined,
        rv2: search.rv2 as string | undefined,
        rv3: search.rv3 as string | undefined,
        rv4: search.rv4 as string | undefined,
        vm: search.vm as string | undefined,
        vm2: search.vm2 as string | undefined,
        vm3: search.vm3 as string | undefined,
        vm4: search.vm4 as string | undefined,
        vl: search.vl as string | undefined,
        vl2: search.vl2 as string | undefined,
        vl3: search.vl3 as string | undefined,
        vl4: search.vl4 as string | undefined,
        vt: search.vt as string | undefined,
        vt2: search.vt2 as string | undefined,
        vt3: search.vt3 as string | undefined,
        vt4: search.vt4 as string | undefined,
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

    const showOriginal = new Set<number>();
    if (search.og) showOriginal.add(0);
    if (search.og2) showOriginal.add(1);
    if (search.og3) showOriginal.add(2);
    if (search.og4) showOriginal.add(3);

    const resourceViews = new Map<number, string>();
    if (search.rv) resourceViews.set(0, search.rv);
    if (search.rv2) resourceViews.set(1, search.rv2);
    if (search.rv3) resourceViews.set(2, search.rv3);
    if (search.rv4) resourceViews.set(3, search.rv4);

    const viewModes = new Map<number, string>();
    if (search.vm) viewModes.set(0, search.vm);
    if (search.vm2) viewModes.set(1, search.vm2);
    if (search.vm3) viewModes.set(2, search.vm3);
    if (search.vm4) viewModes.set(3, search.vm4);

    const viewLayouts = new Map<number, string>();
    if (search.vl) viewLayouts.set(0, search.vl);
    if (search.vl2) viewLayouts.set(1, search.vl2);
    if (search.vl3) viewLayouts.set(2, search.vl3);
    if (search.vl4) viewLayouts.set(3, search.vl4);

    const companionSlugs = new Map<number, string>();
    if (search.vt) companionSlugs.set(0, search.vt);
    if (search.vt2) companionSlugs.set(1, search.vt2);
    if (search.vt3) companionSlugs.set(2, search.vt3);
    if (search.vt4) companionSlugs.set(3, search.vt4);

    return (
        <ReaderLayout
            panels={panels}
            selections={selections}
            resourcesOpen={resourcesOpen}
            showOriginal={showOriginal}
            resourceViews={resourceViews}
            viewModes={viewModes}
            viewLayouts={viewLayouts}
            companionSlugs={companionSlugs}
        />
    );
}
