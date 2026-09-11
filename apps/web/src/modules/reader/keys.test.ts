import { describe, expect, it } from "vitest";
import { deepLinkScrollKey, parseRangeKey } from "./keys";

describe("deepLinkScrollKey", () => {
    it("lands a range on its first sentence", () => {
        // The range key itself is never a `data-sentence-key`, so querying it
        // verbatim matched nothing and the reader stayed at the node top.
        expect(deepLinkScrollKey("345-348")).toBe("345");
    });

    it("passes a single sentence, figure and uuid key through", () => {
        expect(deepLinkScrollKey("348")).toBe("348");
        expect(deepLinkScrollKey("fig3")).toBe("fig3");
        const uuid = "5075a889-bb70-487e-b3ff-2eeed2da3de4";
        expect(deepLinkScrollKey(uuid)).toBe(uuid);
    });

    it("does not read a uuid as a range", () => {
        expect(
            parseRangeKey("5075a889-bb70-487e-b3ff-2eeed2da3de4"),
        ).toBeNull();
    });
});
