import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    TextField,
} from "@mui/material";
import { useState } from "react";
import toast from "react-hot-toast";
import { FetchError } from "../../api/fetcher";
import type { PersonResponse } from "../../api/model";
import { useCreatePerson } from "../../api/persons/persons";

interface PersonFormModalProps {
    open: boolean;
    onClose: () => void;
    onCreated: (person: PersonResponse) => void;
}

function deriveSortName(fullName: string): string {
    const trimmed = fullName.trim();
    if (!trimmed) return "";
    const parts = trimmed.split(/\s+/);
    if (parts.length === 1) return trimmed;
    const surname = parts[parts.length - 1];
    const rest = parts.slice(0, -1).join(" ");
    return `${surname}, ${rest}`;
}

export function PersonFormModal({
    open,
    onClose,
    onCreated,
}: PersonFormModalProps) {
    const [name, setName] = useState("");
    const [sortName, setSortName] = useState("");
    const [sortNameManual, setSortNameManual] = useState(false);

    const createMutation = useCreatePerson({
        mutation: {
            onSuccess: (response) => {
                if (!response?.data) return;
                const person = response.data;
                toast.success(`Person "${person.name}" created`);
                onCreated(person);
                resetAndClose();
            },
            onError: (err: unknown) => {
                const message =
                    err instanceof FetchError && err.message
                        ? err.message
                        : "Failed to create person";
                toast.error(message);
            },
        },
    });

    const resetAndClose = () => {
        setName("");
        setSortName("");
        setSortNameManual(false);
        onClose();
    };

    const handleNameChange = (value: string) => {
        setName(value);
        if (!sortNameManual) {
            setSortName(deriveSortName(value));
        }
    };

    const handleSortNameChange = (value: string) => {
        setSortName(value);
        setSortNameManual(true);
    };

    const handleSubmit = () => {
        if (!name.trim()) {
            toast.error("Name is required");
            return;
        }
        createMutation.mutate({
            data: {
                name: name.trim(),
                sort_name: sortName.trim() || undefined,
            },
        });
    };

    return (
        <Dialog open={open} onClose={resetAndClose} maxWidth="xs" fullWidth>
            <DialogTitle sx={{ fontSize: 16 }}>New Person</DialogTitle>
            <DialogContent
                sx={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 2,
                    pt: "8px !important",
                }}
            >
                <TextField
                    label="Name"
                    value={name}
                    onChange={(e) => handleNameChange(e.target.value)}
                    size="small"
                    required
                    autoFocus
                />
                <TextField
                    label="Sort Name"
                    value={sortName}
                    onChange={(e) => handleSortNameChange(e.target.value)}
                    size="small"
                    placeholder="e.g. Vaughan, Alden T."
                    helperText="Ensure that the sort name is correct (surname, given names)"
                />
            </DialogContent>
            <DialogActions>
                <Button onClick={resetAndClose} size="small">
                    Cancel
                </Button>
                <Button
                    onClick={handleSubmit}
                    variant="contained"
                    size="small"
                    disabled={createMutation.isPending}
                >
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}
