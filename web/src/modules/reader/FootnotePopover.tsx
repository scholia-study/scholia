import { Popover } from "@mui/material";
import parse from "html-react-parser";
import type { FootnoteResponse, FootnoteSentenceResponse } from "../../api/model";
import { footnoteSentenceMatchesKey } from "./BlockRenderer";

interface FootnotePopoverProps {
    footnote: FootnoteResponse;
    anchorEl: HTMLElement | null;
    open: boolean;
    onClose: () => void;
    selectedFootnoteSentenceId: string | undefined;
    onSelectFootnoteSentence: (sentence: FootnoteSentenceResponse, shiftKey: boolean) => void;
    showOriginal?: boolean;
}

export function FootnotePopover({
    footnote,
    anchorEl,
    open,
    onClose,
    selectedFootnoteSentenceId,
    onSelectFootnoteSentence,
    showOriginal,
}: FootnotePopoverProps) {
    return (
        <Popover
            open={open}
            anchorEl={anchorEl}
            onClose={onClose}
            anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
            transformOrigin={{ vertical: "top", horizontal: "center" }}
            slotProps={{
                paper: {
                    sx: { maxWidth: 480, boxShadow: 3 },
                },
            }}
        >
            <div className="px-3 py-2 border-b border-stone-200">
                <span className="text-xs font-medium text-stone-500">
                    Footnote {footnote.number}
                </span>
            </div>
            <div className="px-3 py-2 max-h-[40vh] overflow-y-auto leading-relaxed text-sm text-stone-700">
                {footnote.sentences.map((sentence) => {
                    const isSelected = footnoteSentenceMatchesKey(sentence, selectedFootnoteSentenceId);
                    const html =
                        showOriginal && sentence.original_html
                            ? sentence.original_html
                            : sentence.html;
                    return (
                        <span key={sentence.id}>
                            <span
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onSelectFootnoteSentence(sentence, e.shiftKey);
                                }}
                                className={`cursor-pointer transition-colors rounded-sm ${
                                    isSelected
                                        ? "bg-amber-200"
                                        : "hover:bg-stone-200"
                                }`}
                            >
                                {parse(html)}
                            </span>{" "}
                        </span>
                    );
                })}
            </div>
        </Popover>
    );
}
