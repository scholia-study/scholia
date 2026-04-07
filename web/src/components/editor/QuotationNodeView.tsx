import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import { IconButton } from "@mui/material";
import { useNodeViewContext } from "@prosemirror-adapter/react";
import { QuotationCard } from "../QuotationCard";

export function QuotationNodeView() {
    const { node, selected, view, getPos } = useNodeViewContext();
    const { book, node: nodeSlug, start, end, kind, mode, layout } = node.attrs;

    const handleDelete = () => {
        const pos = getPos();
        if (pos == null) return;
        const tr = view.state.tr.delete(pos, pos + node.nodeSize);
        view.dispatch(tr);
        view.focus();
    };

    return (
        <div
            className={`group relative my-2 ${selected ? "ring-2 ring-blue-300 rounded" : ""}`}
            contentEditable={false}
        >
            <QuotationCard
                book={book}
                node={nodeSlug}
                start={start}
                end={end ?? undefined}
                kind={kind}
                mode={mode}
                layout={layout}
            />
            <div className="absolute top-2 right-10 opacity-0 group-hover:opacity-100 transition-opacity">
                <IconButton
                    size="small"
                    onClick={handleDelete}
                    title="Remove quotation"
                    sx={{ backgroundColor: "rgba(255,255,255,0.8)" }}
                >
                    <DeleteOutlined fontSize="small" />
                </IconButton>
            </div>
        </div>
    );
}
