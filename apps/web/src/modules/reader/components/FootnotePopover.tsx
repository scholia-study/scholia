import CloseOutlined from "@mui/icons-material/CloseOutlined";
import { IconButton, Paper, Popper } from "@mui/material";
import parse from "html-react-parser";
import { useEffect } from "react";
import type { FootnoteResponse } from "../../../api/model";
import { useFootnoteActions, useFootnoteSelection } from "../context/selection";

interface FootnotePopoverProps {
    footnote: FootnoteResponse;
    anchorEl: HTMLElement | null;
    open: boolean;
    onClose: () => void;
    showOriginal?: boolean;
}

export function FootnotePopover({
    footnote,
    anchorEl,
    open,
    onClose,
    showOriginal,
}: FootnotePopoverProps) {
    const { select } = useFootnoteActions();

    useEffect(() => {
        if (!open) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key === "Escape") onClose();
        };
        window.addEventListener("keydown", onKey);
        return () => window.removeEventListener("keydown", onKey);
    }, [open, onClose]);

    return (
        <Popper
            open={open}
            anchorEl={anchorEl}
            placement="bottom"
            sx={{ zIndex: 1300 }}
            modifiers={[{ name: "offset", options: { offset: [0, 4] } }]}
        >
            <Paper sx={{ maxWidth: 480, boxShadow: 3 }}>
                <div className="flex items-center justify-between px-3 py-2 border-b border-stone-200">
                    <span className="text-xs font-medium text-stone-500">
                        Footnote {footnote.number}
                    </span>
                    <IconButton
                        size="small"
                        onClick={onClose}
                        title="Close (Esc)"
                    >
                        <CloseOutlined fontSize="small" />
                    </IconButton>
                </div>
                <div className="px-3 py-2 max-h-[40vh] overflow-y-auto leading-relaxed text-sm text-stone-700">
                    {footnote.sentences.map((sentence) => (
                        <FootnoteSentenceSpan
                            key={sentence.id}
                            sentence={sentence}
                            showOriginal={showOriginal}
                            onSelect={select}
                        />
                    ))}
                </div>
            </Paper>
        </Popper>
    );
}

function FootnoteSentenceSpan({
    sentence,
    showOriginal,
    onSelect,
}: {
    sentence: FootnoteResponse["sentences"][number];
    showOriginal: boolean | undefined;
    onSelect: ReturnType<typeof useFootnoteActions>["select"];
}) {
    const { isSelected } = useFootnoteSelection(sentence);
    const html =
        showOriginal && sentence.original_html
            ? sentence.original_html
            : sentence.html;
    return (
        <span>
            <span
                onClick={(e) => {
                    e.stopPropagation();
                    onSelect(sentence, e.shiftKey);
                }}
                className={`cursor-pointer transition-colors rounded-sm ${
                    isSelected ? "bg-amber-200" : "hover:bg-stone-200"
                }`}
            >
                {parse(html)}
            </span>{" "}
        </span>
    );
}
