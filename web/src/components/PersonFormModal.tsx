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
import type { PersonResponse } from "../api/model";
import { useCreatePerson } from "../api/persons/persons";

interface PersonFormModalProps {
    open: boolean;
    onClose: () => void;
    onCreated: (person: PersonResponse) => void;
}

export function PersonFormModal({
    open,
    onClose,
    onCreated,
}: PersonFormModalProps) {
    const [name, setName] = useState("");
    const [sortName, setSortName] = useState("");

    const createMutation = useCreatePerson({
        mutation: {
            onSuccess: (response) => {
                if (!response?.data) return;
                const person = response.data;
                toast.success(`Person "${person.name}" created`);
                onCreated(person);
                resetAndClose();
            },
            onError: () => {
                toast.error("Failed to create person");
            },
        },
    });

    const resetAndClose = () => {
        setName("");
        setSortName("");
        onClose();
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
            <DialogContent sx={{ display: "flex", flexDirection: "column", gap: 2, pt: "8px !important" }}>
                <TextField
                    label="Name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    size="small"
                    required
                    autoFocus
                />
                <TextField
                    label="Sort Name"
                    value={sortName}
                    onChange={(e) => setSortName(e.target.value)}
                    size="small"
                    placeholder="e.g. Vaughan, Alden T."
                    helperText="Optional. Used for bibliography ordering."
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
