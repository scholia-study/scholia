import ArrowBackOutlined from "@mui/icons-material/ArrowBackOutlined";
import CloseOutlined from "@mui/icons-material/CloseOutlined";
import CommitOutlined from "@mui/icons-material/CommitOutlined";
import CompareOutlined from "@mui/icons-material/CompareOutlined";
import EditNoteOutlined from "@mui/icons-material/EditNoteOutlined";
import ExploreOutlined from "@mui/icons-material/ExploreOutlined";
import FavoriteBorderOutlined from "@mui/icons-material/FavoriteBorderOutlined";
import FavoriteOutlined from "@mui/icons-material/FavoriteOutlined";
import FeedbackOutlined from "@mui/icons-material/FeedbackOutlined";
import InfoOutlined from "@mui/icons-material/InfoOutlined";
import ListOutlined from "@mui/icons-material/ListOutlined";
import MenuBookOutlined from "@mui/icons-material/MenuBookOutlined";
import { IconButton } from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type React from "react";
import { useEffect, useMemo, useState } from "react";
import toast from "react-hot-toast";
import {
    invalidateAllNodeQuotations,
    NoteFormModal,
    useUnsaveQuotation,
} from "#/modules/quotation";
import { useReaderTour } from "#/modules/tour";
import { useListBooks } from "../../../api/books/books";
import { FetchError } from "../../../api/fetcher";
import type {
    FootnoteSentenceResponse,
    NoteResponse,
    ResourceResponse,
    SentenceResponse,
    TocNodeResponse,
} from "../../../api/model";
import {
    getListAllQuotationsQueryKey,
    useCreateQuotation,
    useListQuotations,
} from "../../../api/quotations/quotations";
import { useListResources } from "../../../api/resources/resources";
import { useGetToc } from "../../../api/toc/toc";
import { useAuth } from "../../../hooks/useAuth";
import { useFeedback } from "../../feedback";
import { AboutThisTextView } from "./AboutThisTextView";
import { CommentaryView, getSentenceRange } from "./CommentaryView";
import { NotesView } from "./NotesView";
import { PanelToc } from "./PanelToc";
import { ResourceFormModal } from "./ResourceFormModal";
import { SentenceDetail } from "./SentenceDetail";

type ViewKind =
    | "about"
    | "toc"
    | "compare"
    | "verbatim"
    | "paraphrase"
    | "allusion"
    | "sentence"
    | "notes";

interface ResourcesPanelProps {
    toc: TocNodeResponse[] | undefined;
    bookSlug: string;
    activeNodeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
    selectedSentence:
        | SentenceResponse
        | FootnoteSentenceResponse
        | (SentenceResponse | FootnoteSentenceResponse)[]
        | undefined;
    selectedSentenceId: string | undefined;
    onClose: () => void;
    activeView: string | undefined;
    onViewChange: (view: string | undefined) => void;
}

export function ResourcesPanel({
    toc,
    bookSlug,
    activeNodeSlug,
    onNavigate,
    onAddComparisonPanel,
    canAddPanel,
    selectedSentence,
    selectedSentenceId,
    onClose,
    activeView,
    onViewChange,
}: ResourcesPanelProps) {
    const { user } = useAuth();
    const { startReaderTour } = useReaderTour();
    const { openModal: openFeedbackModal } = useFeedback();
    const isEditor =
        user?.roles?.includes("editor") ||
        user?.roles?.includes("admin") ||
        false;

    // Fetch resource counts for menu badges
    const sentenceRange = useMemo(
        () => getSentenceRange(selectedSentence),
        [selectedSentence],
    );
    const { data: resourcesData } = useListResources(
        bookSlug,
        {
            start: sentenceRange?.start ?? 0,
            end: sentenceRange?.end ?? 0,
            kind: sentenceRange?.kind ?? "body",
        },
        { query: { enabled: !!sentenceRange } },
    );
    const commentaryCounts = useMemo(() => {
        const all = resourcesData?.data?.resources ?? [];
        return {
            verbatim: all.filter((r) => r.resource_type === "verbatim").length,
            paraphrase: all.filter((r) => r.resource_type === "paraphrase")
                .length,
            allusion: all.filter((r) => r.resource_type === "allusion").length,
        };
    }, [resourcesData]);

    // Modal state for resource create/edit (used by ResourceFormModal)
    const [resourceModalOpen, setResourceModalOpen] = useState(false);
    const [editingResource, setEditingResource] = useState<
        ResourceResponse | undefined
    >();
    const [modalDefaults, setModalDefaults] = useState<{
        type: "verbatim" | "paraphrase" | "allusion";
        start: number;
        end: number | undefined;
        kind: string;
    } | null>(null);

    const handleAddResource = (
        type: "verbatim" | "paraphrase" | "allusion",
        start: number,
        end: number | undefined,
        kind: string,
    ) => {
        setEditingResource(undefined);
        setModalDefaults({ type, start, end, kind });
        setResourceModalOpen(true);
    };

    const handleEditResource = (resource: ResourceResponse) => {
        setEditingResource(resource);
        setModalDefaults(null);
        setResourceModalOpen(true);
    };

    // Resolve active node ID from toc
    const activeNodeId = useMemo(() => {
        if (!activeNodeSlug || !toc) return undefined;
        const find = (nodes: TocNodeResponse[]): string | undefined => {
            for (const n of nodes) {
                if (n.slug === activeNodeSlug) return n.id;
                const found = find(n.children);
                if (found) return found;
            }
        };
        return find(toc);
    }, [activeNodeSlug, toc]);

    // Fetch quotation note count for menu badge
    const { data: quotationsData } = useListQuotations(
        bookSlug,
        { node_id: activeNodeId ?? "" },
        { query: { enabled: !!activeNodeId && !!user } },
    );
    const noteCount = useMemo(() => {
        if (!quotationsData?.data?.quotations || !sentenceRange) return 0;
        return quotationsData.data.quotations
            .filter((q) => {
                if (q.sentence_kind !== sentenceRange.kind) return false;
                const qStart = q.anchor_sentence_start_number;
                const qEnd = q.anchor_sentence_end_number ?? qStart;
                return (
                    qStart <= sentenceRange.end && qEnd >= sentenceRange.start
                );
            })
            .reduce((sum, q) => sum + q.note_count, 0);
    }, [quotationsData, sentenceRange]);

    // Check if current selection has an exact-match saved quotation
    const exactQuotation = useMemo(() => {
        if (!sentenceRange || !quotationsData?.data?.quotations)
            return undefined;
        return quotationsData.data.quotations.find((q) => {
            const startMatch =
                q.anchor_sentence_start_number === sentenceRange.start;
            const endMatch =
                sentenceRange.start === sentenceRange.end
                    ? q.anchor_sentence_end_number == null ||
                      q.anchor_sentence_end_number === sentenceRange.start
                    : q.anchor_sentence_end_number === sentenceRange.end;
            const kindMatch = q.sentence_kind === sentenceRange.kind;
            return startMatch && endMatch && kindMatch;
        });
    }, [sentenceRange, quotationsData]);

    const queryClient = useQueryClient();

    const createQuotation = useCreateQuotation({
        mutation: {
            onSuccess: () => {
                toast.success("Quotation saved");
                // Verse-level marker projection (PLAN_BIG_BOOKS.md Q7)
                // means a save in WEB also affects KJV's marker render.
                // Wider-than-current-book invalidation prevents the
                // "had to hard refresh to see the marker" bug.
                invalidateAllNodeQuotations(queryClient);
                // Also refresh the "My Quotations" account list, which is
                // cached independently of the reader's node markers.
                queryClient.invalidateQueries({
                    queryKey: getListAllQuotationsQueryKey(),
                });
            },
            onError: (err: unknown) => {
                const message =
                    err instanceof FetchError && err.message
                        ? err.message
                        : "Failed to save quotation";
                toast.error(message);
            },
        },
    });

    const {
        requestUnsave,
        UnsaveDialog,
        isPending: unsavePending,
    } = useUnsaveQuotation({
        bookSlug,
    });

    const handleToggleSaveQuotation = () => {
        if (!sentenceRange) return;
        if (exactQuotation) {
            requestUnsave(exactQuotation);
        } else {
            createQuotation.mutate({
                slug: bookSlug,
                data: {
                    sentence_start: sentenceRange.start,
                    sentence_end:
                        sentenceRange.start !== sentenceRange.end
                            ? sentenceRange.end
                            : undefined,
                    sentence_kind: sentenceRange.kind,
                },
            });
        }
    };

    // Modal state for note create/edit
    const [noteModalOpen, setNoteModalOpen] = useState(false);
    const [noteModalQuotationId, setNoteModalQuotationId] =
        useState<string>("");
    const [editingNote, setEditingNote] = useState<NoteResponse | undefined>();

    const handleOpenNoteModal = (quotationId: string, note?: NoteResponse) => {
        setNoteModalQuotationId(quotationId);
        setEditingNote(note);
        setNoteModalOpen(true);
    };

    // Build sentence context string for note modal
    const sentenceContextStr = useMemo(() => {
        if (!sentenceRange) return undefined;
        const { start, end } = sentenceRange;
        const label =
            start === end ? `Sentence ${start}` : `Sentences ${start}–${end}`;
        // Try to get a text snippet from selected sentences
        if (!selectedSentence) return label;
        const sentences = Array.isArray(selectedSentence)
            ? selectedSentence
            : [selectedSentence];
        if (sentences.length === 0) return label;
        const firstText = sentences[0].text;
        const snippet =
            firstText.length > 60 ? `${firstText.slice(0, 60)}...` : firstText;
        if (sentences.length === 1) return `${label}: "${snippet}"`;
        const lastText = sentences[sentences.length - 1].text;
        const endSnippet =
            lastText.length > 40 ? `...${lastText.slice(-40)}` : lastText;
        return `${label}: "${snippet}" ... "${endSnippet}"`;
    }, [sentenceRange, selectedSentence]);

    const viewKind = activeView as ViewKind | undefined;
    const isMenu = !viewKind;

    // For compare sub-navigation (picker vs compare-toc), keep internal state
    const [compareBookSlug, setCompareBookSlug] = useState<
        string | undefined
    >();

    // Reset compare sub-state when leaving compare view
    useEffect(() => {
        if (viewKind !== "compare") {
            setCompareBookSlug(undefined);
        }
    }, [viewKind]);

    return (
        <aside
            data-tour="resources-panel"
            className="w-80 border-l border-stone-200 bg-white shrink-0 flex flex-col"
        >
            {/* Header - matches TextPanel toolbar height */}
            <div className="border-b border-stone-200 bg-stone-50 shrink-0 py-2 flex items-center px-3">
                <IconButton
                    size="small"
                    onClick={() => onViewChange(undefined)}
                    title="Back to menu"
                    tabIndex={isMenu ? -1 : undefined}
                    sx={{ visibility: isMenu ? "hidden" : "visible", mr: 0.5 }}
                >
                    <ArrowBackOutlined fontSize="small" />
                </IconButton>
                <div className="flex-1 min-w-0">
                    <div className="text-sm text-stone-800 truncate">
                        Resources
                    </div>
                    <div className="text-xs text-stone-400 truncate">
                        {viewKind === "about"
                            ? "About this text"
                            : viewKind === "toc"
                              ? "Table of Contents"
                              : viewKind === "sentence"
                                ? "Sentence Details"
                                : viewKind === "compare"
                                  ? "Compare Text"
                                  : viewKind === "verbatim"
                                    ? "Verbatim Quotations"
                                    : viewKind === "paraphrase"
                                      ? "Paraphrases"
                                      : viewKind === "allusion"
                                        ? "Allusions"
                                        : viewKind === "notes"
                                          ? "Notes"
                                          : "\u00A0"}
                    </div>
                </div>
                <IconButton size="small" onClick={onClose} title="Close">
                    <CloseOutlined fontSize="small" />
                </IconButton>
            </div>

            {/* Menu */}
            {isMenu && (
                <div className="flex-1 overflow-y-auto">
                    <nav className="p-2 space-y-1">
                        <MenuButton
                            onClick={() => onViewChange("about")}
                            label="About this text"
                            icon={<InfoOutlined fontSize="small" />}
                        />
                        <MenuButton
                            onClick={() => onViewChange("toc")}
                            label="Table of Contents"
                            dataTour="toc"
                            icon={<ListOutlined fontSize="small" />}
                        />
                        <MenuButton
                            onClick={() => onViewChange("sentence")}
                            label="Sentence Details"
                            disabled={!selectedSentence}
                            icon={<CommitOutlined fontSize="small" />}
                        />
                        {canAddPanel && (
                            <MenuButton
                                onClick={() => onViewChange("compare")}
                                label="Compare Text"
                                dataTour="compare"
                                icon={<CompareOutlined fontSize="small" />}
                            />
                        )}
                        <MenuButton
                            onClick={() => void startReaderTour()}
                            label="Take a Tour"
                            icon={<ExploreOutlined fontSize="small" />}
                        />
                        <div className="text-[11px] uppercase tracking-wider text-stone-400 font-medium px-3 pt-3 pb-1">
                            Commentary
                        </div>
                        <MenuButton
                            onClick={() => onViewChange("verbatim")}
                            label={`Verbatim${commentaryCounts.verbatim ? ` (${commentaryCounts.verbatim})` : ""}`}
                            dataTour="commentary"
                            disabled={!selectedSentence}
                            icon={
                                <MenuBookOutlined
                                    fontSize="small"
                                    sx={{ color: "#722f37" }}
                                />
                            }
                        />
                        <MenuButton
                            onClick={() => onViewChange("paraphrase")}
                            label={`Paraphrase${commentaryCounts.paraphrase ? ` (${commentaryCounts.paraphrase})` : ""}`}
                            disabled={!selectedSentence}
                            icon={
                                <MenuBookOutlined
                                    fontSize="small"
                                    sx={{ color: "#5c6b8b" }}
                                />
                            }
                        />
                        <MenuButton
                            onClick={() => onViewChange("allusion")}
                            label={`Allusion${commentaryCounts.allusion ? ` (${commentaryCounts.allusion})` : ""}`}
                            disabled={!selectedSentence}
                            icon={
                                <MenuBookOutlined
                                    fontSize="small"
                                    sx={{ color: "#5c7a5c" }}
                                />
                            }
                        />
                        <div
                            data-tour="tools"
                            className="text-[11px] uppercase tracking-wider text-stone-400 font-medium px-3 pt-3 pb-1"
                        >
                            Tools
                        </div>
                        {user ? (
                            <>
                                <MenuButton
                                    onClick={() => onViewChange("notes")}
                                    label={`Notes${noteCount ? ` (${noteCount})` : ""}`}
                                    disabled={!selectedSentence}
                                    icon={
                                        <EditNoteOutlined
                                            fontSize="small"
                                            sx={{ color: "#6b5b73" }}
                                        />
                                    }
                                />
                                <MenuButton
                                    onClick={handleToggleSaveQuotation}
                                    label={
                                        exactQuotation
                                            ? "Unsave Quotation"
                                            : "Save Quotation"
                                    }
                                    disabled={
                                        !selectedSentence ||
                                        createQuotation.isPending ||
                                        unsavePending
                                    }
                                    icon={
                                        exactQuotation ? (
                                            <FavoriteOutlined
                                                fontSize="small"
                                                sx={{ color: "#b45264" }}
                                            />
                                        ) : (
                                            <FavoriteBorderOutlined
                                                fontSize="small"
                                                sx={{ color: "#b45264" }}
                                            />
                                        )
                                    }
                                />
                                <MenuButton
                                    onClick={openFeedbackModal}
                                    label="Send Feedback"
                                    icon={
                                        <FeedbackOutlined
                                            fontSize="small"
                                            sx={{ color: "#5c6b8b" }}
                                        />
                                    }
                                />
                            </>
                        ) : (
                            <p className="px-3 py-2 text-sm text-stone-400">
                                <Link
                                    to="/login"
                                    className="text-stone-600 underline hover:text-stone-900"
                                >
                                    Log in or create an account
                                </Link>{" "}
                                to start saving quotations, writig notes and
                                articles.
                            </p>
                        )}
                    </nav>
                </div>
            )}

            {/* About this text view */}
            {viewKind === "about" && (
                <AboutThisTextView
                    bookSlug={bookSlug}
                    activeNodeSlug={activeNodeSlug}
                />
            )}

            {/* Table of Contents view */}
            {viewKind === "toc" &&
                (toc ? (
                    <PanelToc
                        toc={toc}
                        bookSlug={bookSlug}
                        activeNodeSlug={activeNodeSlug}
                        onNavigate={onNavigate}
                    />
                ) : (
                    <div className="p-4 text-sm text-stone-400">Loading...</div>
                ))}

            {/* Sentence Details view */}
            {viewKind === "sentence" &&
                (selectedSentence ? (
                    <SentenceDetail sentence={selectedSentence} />
                ) : (
                    <div className="p-4 text-sm text-stone-400">
                        Click a sentence to see its details.
                    </div>
                ))}

            {/* Commentary views */}
            {(viewKind === "verbatim" ||
                viewKind === "paraphrase" ||
                viewKind === "allusion") && (
                <CommentaryView
                    bookSlug={bookSlug}
                    resourceType={viewKind}
                    selectedSentence={selectedSentence}
                    isEditor={isEditor}
                    onAdd={handleAddResource}
                    onEdit={handleEditResource}
                />
            )}

            {/* Compare Text view */}
            {viewKind === "compare" &&
                (!compareBookSlug ? (
                    <BookPickerView onPickBook={setCompareBookSlug} />
                ) : (
                    <CompareTocView
                        compareBookSlug={compareBookSlug}
                        onSelectNode={(nodeSlug) =>
                            onAddComparisonPanel(compareBookSlug, nodeSlug)
                        }
                        onBack={() => setCompareBookSlug(undefined)}
                    />
                ))}

            {/* Notes view */}
            {viewKind === "notes" && (
                <NotesView
                    bookSlug={bookSlug}
                    activeNodeId={activeNodeId}
                    selectedSentence={selectedSentence}
                    selectedSentenceId={selectedSentenceId}
                    onOpenNoteModal={handleOpenNoteModal}
                />
            )}

            {/* Resource create/edit modal */}
            <ResourceFormModal
                key={
                    editingResource?.id ??
                    `${modalDefaults?.type}-${modalDefaults?.start}-${modalDefaults?.end}-${modalDefaults?.kind}`
                }
                open={resourceModalOpen}
                onClose={() => setResourceModalOpen(false)}
                bookSlug={bookSlug}
                mode={editingResource ? "edit" : "create"}
                initialData={editingResource}
                defaultType={modalDefaults?.type}
                defaultSentenceStart={modalDefaults?.start}
                defaultSentenceEnd={modalDefaults?.end}
                defaultSentenceKind={modalDefaults?.kind}
                isAdmin={user?.roles?.includes("admin") ?? false}
            />

            {/* Note create/edit modal */}
            {noteModalOpen && (
                <NoteFormModal
                    key={editingNote?.id ?? `new-note-${noteModalQuotationId}`}
                    open={noteModalOpen}
                    onClose={() => setNoteModalOpen(false)}
                    bookSlug={bookSlug}
                    quotationId={noteModalQuotationId}
                    mode={editingNote ? "edit" : "create"}
                    initialData={editingNote}
                    sentenceContext={sentenceContextStr}
                />
            )}

            {UnsaveDialog}
        </aside>
    );
}

function MenuButton({
    onClick,
    label,
    disabled,
    icon,
    dataTour,
}: {
    onClick: () => void;
    label: string;
    disabled?: boolean;
    icon: React.ReactNode;
    dataTour?: string;
}) {
    return (
        <button
            onClick={onClick}
            disabled={disabled}
            data-tour={dataTour}
            className="w-full text-left text-sm px-3 py-2 rounded hover:bg-stone-100 text-stone-700 transition-colors disabled:text-stone-300 disabled:hover:bg-transparent disabled:cursor-default flex items-center gap-2"
        >
            <span className="text-stone-400">{icon}</span>
            {label}
        </button>
    );
}

function CompareTocView({
    compareBookSlug,
    onSelectNode,
    onBack,
}: {
    compareBookSlug: string;
    onSelectNode: (nodeSlug: string) => void;
    onBack: () => void;
}) {
    const { data: tocData } = useGetToc(compareBookSlug);
    const toc = tocData?.data;

    if (!toc) {
        return <div className="p-4 text-sm text-stone-400">Loading...</div>;
    }

    return (
        <>
            <div className="px-3 py-1.5 border-b border-stone-100 flex items-center">
                <button
                    onClick={onBack}
                    className="text-stone-400 hover:text-stone-600 mr-2 text-xs"
                >
                    &larr;
                </button>
                <span className="text-xs text-stone-500">Select section</span>
            </div>
            <PanelToc
                toc={toc}
                bookSlug={compareBookSlug}
                activeNodeSlug={undefined}
                onNavigate={onSelectNode}
            />
        </>
    );
}

function BookPickerView({
    onPickBook,
}: {
    onPickBook: (bookSlug: string) => void;
}) {
    const { data, isLoading, error } = useListBooks();
    const books = data?.data;

    return (
        <div className="flex-1 overflow-y-auto p-2">
            {isLoading && (
                <p className="text-stone-400 text-sm p-2">Loading...</p>
            )}
            {error ? (
                <p className="text-red-500 text-sm p-2">
                    Failed to load books.
                </p>
            ) : null}
            {books && (
                <ul className="space-y-1">
                    {books.map((book) => (
                        <li key={book.id}>
                            <button
                                onClick={() => onPickBook(book.slug)}
                                className="block w-full text-left px-2 py-1.5 rounded hover:bg-stone-100 transition-colors"
                            >
                                <div className="text-sm text-stone-900">
                                    {book.title}
                                </div>
                                <div className="text-xs text-stone-500">
                                    {book.author}
                                </div>
                            </button>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
