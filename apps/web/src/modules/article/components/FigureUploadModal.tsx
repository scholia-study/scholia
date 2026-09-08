import {
    Alert,
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    TextField,
} from "@mui/material";
import { useCallback, useEffect, useRef, useState } from "react";
import { FetchError } from "../../../api/fetcher";
import { uploadArticleImage } from "../api/uploadArticleImage";
import {
    ACCEPTED_IMAGE_TYPES,
    imageFileFrom,
    MAX_UPLOAD_BYTES,
} from "./imageFile";
import type { FigureInsert } from "./MdxEditor";

interface FigureUploadModalProps {
    open: boolean;
    onClose: () => void;
    onInsert: (figure: FigureInsert) => void;
    /** Pre-selected file (from a drop or paste on the editor surface). */
    initialFile?: File | null;
}

export function FigureUploadModal({
    open,
    onClose,
    onInsert,
    initialFile,
}: FigureUploadModalProps) {
    const fileInputRef = useRef<HTMLInputElement>(null);
    const [file, setFile] = useState<File | null>(null);
    const [previewUrl, setPreviewUrl] = useState<string | null>(null);
    const [alt, setAlt] = useState("");
    const [caption, setCaption] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [uploading, setUploading] = useState(false);

    const selectFile = useCallback((selected: File) => {
        if (!ACCEPTED_IMAGE_TYPES.includes(selected.type)) {
            setError("Only JPEG, PNG, and WebP images are accepted.");
            return;
        }
        if (selected.size > MAX_UPLOAD_BYTES) {
            setError("Image exceeds the 5 MB upload limit.");
            return;
        }
        setError(null);
        setFile(selected);
    }, []);

    useEffect(() => {
        if (!open) {
            setFile(null);
            setAlt("");
            setCaption("");
            setError(null);
            setUploading(false);
        } else if (initialFile) {
            selectFile(initialFile);
        }
    }, [open, initialFile, selectFile]);

    useEffect(() => {
        if (!file) {
            setPreviewUrl(null);
            return;
        }
        const url = URL.createObjectURL(file);
        setPreviewUrl(url);
        return () => URL.revokeObjectURL(url);
    }, [file]);

    // Pasting an image while the modal is open selects it.
    useEffect(() => {
        if (!open) return;
        const handler = (e: ClipboardEvent) => {
            const pasted = imageFileFrom(e.clipboardData);
            if (pasted) {
                e.preventDefault();
                selectFile(pasted);
            }
        };
        window.addEventListener("paste", handler);
        return () => window.removeEventListener("paste", handler);
    }, [open, selectFile]);

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const selected = e.target.files?.[0] ?? null;
        if (selected) selectFile(selected);
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        const dropped = imageFileFrom(e.dataTransfer);
        if (dropped) selectFile(dropped);
    };

    const handleDragOver = (e: React.DragEvent) => {
        if (e.dataTransfer.types.includes("Files")) e.preventDefault();
    };

    const handleInsert = async () => {
        if (!file) return;
        setUploading(true);
        setError(null);
        try {
            const uploaded = await uploadArticleImage(file);
            onInsert({
                src: uploaded.url,
                alt: alt.trim() || undefined,
                caption: caption.trim() || undefined,
                width: uploaded.width,
                height: uploaded.height,
            });
            onClose();
        } catch (err) {
            setError(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Upload failed. Please try again.",
            );
        } finally {
            setUploading(false);
        }
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>Insert figure</DialogTitle>
            <DialogContent>
                <div className="space-y-4 pt-2">
                    <input
                        ref={fileInputRef}
                        type="file"
                        accept={ACCEPTED_IMAGE_TYPES.join(",")}
                        onChange={handleFileChange}
                        className="hidden"
                    />
                    {previewUrl ? (
                        <button
                            type="button"
                            onClick={() => fileInputRef.current?.click()}
                            onDrop={handleDrop}
                            onDragOver={handleDragOver}
                            title="Choose a different image"
                            className="block w-full cursor-pointer rounded border border-stone-200 p-2"
                        >
                            <img
                                src={previewUrl}
                                alt="Upload preview"
                                className="mx-auto max-h-72 max-w-full"
                            />
                        </button>
                    ) : (
                        <button
                            type="button"
                            onClick={() => fileInputRef.current?.click()}
                            onDrop={handleDrop}
                            onDragOver={handleDragOver}
                            className="block w-full cursor-pointer rounded border-2 border-dashed border-stone-300 p-10 text-center text-sm text-stone-500 hover:border-stone-400 hover:text-stone-600"
                        >
                            Choose, drop, or paste an image (JPEG, PNG, or WebP
                            — max 5 MB). It will be re-encoded and displayed
                            inside a figure with your caption.
                        </button>
                    )}
                    <TextField
                        label="Alt text"
                        size="small"
                        fullWidth
                        value={alt}
                        onChange={(e) => setAlt(e.target.value)}
                        helperText="Describes the image for screen readers"
                    />
                    <TextField
                        label="Caption"
                        size="small"
                        fullWidth
                        value={caption}
                        onChange={(e) => setCaption(e.target.value)}
                    />
                    {error && <Alert severity="error">{error}</Alert>}
                </div>
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    variant="contained"
                    onClick={handleInsert}
                    disabled={!file || uploading}
                >
                    {uploading ? "Uploading…" : "Insert"}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
